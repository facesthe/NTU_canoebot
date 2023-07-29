pub mod booking;
pub mod callbacks;
pub mod menu;
pub mod src;

use std::error::Error;
use std::str::FromStr;

use async_trait::async_trait;
use base64::engine::GeneralPurpose;
use base64::Engine;
use bincode::ErrorKind;
pub use booking::Booking;
pub use menu::Menu;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
const BASE64_ENGINE: GeneralPurpose = base64::engine::general_purpose::STANDARD;

/// Callback data type.
/// All callback subtypes **must** be reachable through this type.
/// That means that this enum must contain all possible callback variants.
///
/// Enums can be nested ad infinitum, as long as they and their structs derive:
/// ```no-run
/// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// ```
///
/// This type contains callback data that can be attached to any
/// inline markup button.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Callback {
    BigData(callbacks::BigData),
    Empty,
    Menu(menu::Menu),
    Src(src::Src),
    /// Custom callback handlers that might not be linked
    /// to a particular command.
    Custom,
}

/// Handle a callback.
///
/// Each callback variant must contain a struct (unit struct or otherwise).
///
/// ```no_run
/// use std::error::Error;
///
/// use async_trait::async_trait;
/// use teloxide::prelude::*;
///
/// /// All structs nested inside this one must derive these traits
/// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// pub enum Callback {
///     Button(ButtonCallback),
/// }
///
/// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// /// ButtonCallback is the struct enclosed by the `Button` callback variant.
/// pub struct ButtonCallback {}
///
/// #[async_trait]
/// impl HandleCallback for ButtonCallback {
///     async fn handle_callback(
///         &self,
///         bot: Bot,
///         query: CallbackQuery,
///     ) -> Result<(), Box<dyn Error + Send + Sync>> {
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
trait HandleCallback {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// Default inner struct for some enums
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// UUID that identifies user in database
    pub(crate) uuid: u128,
}

// Main conversion logic for stuff -> Callback
impl TryFrom<&Vec<u8>> for Callback {
    type Error = Box<dyn Error>;
    /// The `TryFrom<&Vec<u8>>` and `TryFrom<&Callback>` traits must
    /// successfully serialize and deserialize the Callback type, or inline
    /// markup buttons won't work!
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let bin_decoded = BASE64_ENGINE.decode(value)?;

        let bin_inflated = inflate::inflate_bytes(&bin_decoded)?;

        match bincode::deserialize::<Callback>(&bin_inflated) {
            Ok(_callback) => Ok(_callback),
            Err(_err) => Err(_err),
        }
    }
}

// the following impls automatically implement other conversions into Callback
impl TryFrom<Vec<u8>> for Callback {
    type Error = Box<dyn Error>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let val_ref = &value;
        val_ref.try_into()
    }
}

impl FromStr for Callback {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<&str> for Callback {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let chunks = value.as_bytes();
        chunks.to_vec().try_into()
    }
}

impl TryFrom<String> for Callback {
    type Error = Box<dyn Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let val_ref = value.as_str();
        val_ref.try_into()
    }
}

// Main conversion logic for Callback -> stuff
impl TryFrom<&Callback> for Vec<u8> {
    type Error = Box<ErrorKind>;

    /// The `TryFrom<&Vec<u8>>` and `TryFrom<&Callback>` traits must
    /// successfully serialize and deserialize the Callback type, or inline
    /// markup buttons won't work!
    fn try_from(value: &Callback) -> Result<Self, Self::Error> {
        // let val_borrow: &Callback = val.borrow();

        let bin_data = {
            match bincode::serialize(&value) {
                Ok(_bin) => _bin,
                Err(_err) => return Err(_err),
            }
        };

        let bin_deflate = deflate::deflate_bytes(&bin_data);
        log::trace!("size of callback data: {} bytes", bin_deflate.len());

        let bin_chars = BASE64_ENGINE.encode(&bin_deflate);

        Ok(bin_chars.as_bytes().to_owned())
    }
}

// the following impls automatically implement other conversions from Callback
impl TryFrom<Callback> for Vec<u8> {
    type Error = Box<ErrorKind>;

    fn try_from(value: Callback) -> Result<Self, Self::Error> {
        let val_ref = &value;
        val_ref.try_into()
    }
}

impl From<Callback> for String {
    fn from(val: Callback) -> Self {
        let char_vec: Vec<u8> = val.try_into().unwrap();
        std::str::from_utf8(&char_vec).unwrap().to_string()
    }
}

impl ToString for Callback {
    fn to_string(&self) -> String {
        let char_vec: Vec<u8> = self.try_into().unwrap();
        std::str::from_utf8(&char_vec).unwrap().to_string()
    }
}

#[async_trait]
impl HandleCallback for Callback {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self {
            Callback::BigData(call) => call.handle_callback(bot, query).await,
            Callback::Empty => {
                bot.answer_callback_query(&query.id).await?;
                Ok(())
            }
            Callback::Menu(call) => call.handle_callback(bot, query).await,
            Callback::Src(call) => call.handle_callback(bot, query).await,

            // to catch unimpl'd callbacks
            _ => {
                log::debug!("callback not yet specified in match arm");
                Ok(())
            }
        }
    }
}

/// Main callback handler
pub async fn callback_handler(
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // answer the callback query once at the top
    bot.answer_callback_query(&query.id).await?;

    let _callback_data: Callback = {
        if let Some(data) = &query.data {
            let data_vec = data.as_bytes().to_owned();
            (&data_vec).try_into().unwrap()
        } else {
            Callback::Empty
        }
    };

    _callback_data.handle_callback(bot, query).await?;

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

    /// Tests serializing and deserializing the callback data
    #[test]
    fn test_callback_serde() {
        let callback = Callback::BigData(callbacks::BigData { uuid: u64::MAX });

        // Callback::OtherThing {
        //     name: "asadasdasdjanskdjanskdjaksjdnkajsdkjnajksasdasdsad".to_string(),
        //     age: 13,
        // };

        let serialized: Vec<u8> = (&callback).try_into().unwrap();
        let deserialized: Callback = (&serialized).try_into().unwrap();

        // let x: Callback = serialized.bytes().try_into();
        println!("serialized size: {}, {:?}", serialized.len(), &serialized);
        println!(
            "Serialized to string: {:?}",
            std::str::from_utf8(&serialized)
        );

        assert_eq!(callback, deserialized);
    }
}
