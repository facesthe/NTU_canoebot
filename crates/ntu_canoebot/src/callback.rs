mod breakdown;
pub mod callbacks;
mod land;
mod logsheet;
mod namelist;
mod paddling;
mod ping;
// pub mod src;
mod training;
mod whatactually;

use std::str::FromStr;
use std::{error::Error, time::Duration};

use anyhow::anyhow;
use async_trait::async_trait;
use base64::engine::GeneralPurpose;
use base64::Engine;
use bincode::ErrorKind;
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use ntu_canoebot_traits::{DeriveEnumParent, EnumParent};
use ntu_canoebot_util::debug_println;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
const BASE64_ENGINE: GeneralPurpose = base64::engine::general_purpose::STANDARD;

pub use breakdown::{breakdown_get, Breakdown};
pub use land::land_get;
pub use logsheet::{logsheet_start, LogSheet};
pub use namelist::namelist_get;
pub use paddling::{paddling_get, Paddling};
pub use ping::ping_start;
use teloxide::types::MaybeInaccessibleMessage;
pub use training::training_get;
pub use whatactually::whatactually_get;

use crate::{
    frame::construct_keyboard_tuple,
    threadmonitor::{DynResult, THREAD_WATCH},
};

const BLANK_BLOCK: char = '\u{2588}';

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, DeriveEnumParent)]
#[enum_parent((_))]
pub enum Callback {
    Empty,
    // Src(src::Src),
    NameList(namelist::NameList),
    Training(training::Training),
    Paddling(paddling::Paddling),
    Land(land::Land),
    Breakdown(breakdown::Breakdown),
    LogSheet(logsheet::LogSheet),
    Ping(ping::Ping),
    WhatActually(whatactually::WhatActually),
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

// Add new callbacks to the match arm here
#[async_trait]
impl HandleCallback for Callback {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self {
            Callback::Empty => {
                bot.answer_callback_query(&query.id).await?;
                Ok(())
            }
            // Callback::Src(call) => call.handle_callback(bot, query).await,
            Callback::NameList(call) => call.handle_callback(bot, query).await,
            Callback::Training(call) => call.handle_callback(bot, query).await,
            Callback::Paddling(call) => call.handle_callback(bot, query).await,
            Callback::Land(call) => call.handle_callback(bot, query).await,
            Callback::Breakdown(call) => call.handle_callback(bot, query).await,
            Callback::LogSheet(call) => call.handle_callback(bot, query).await,
            Callback::Ping(call) => call.handle_callback(bot, query).await,
            Callback::WhatActually(call) => call.handle_callback(bot, query).await,
            // testing

            // to catch unimpl'd callbacks
            _ => {
                debug_println!("callback present but not explicitly handled in match arm");
                Ok(())
            }
        }
    }
}

// /// Trait for initializing an object from a date.
// pub trait FromDate {
//     type Output;

//     fn from_date(date: NaiveDate) -> Self::Output;
// }

/// Date struct passed inside callbacks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// Time struct passed inside callbacks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Time {
    // 24-hour time
    pub hour_mil: u32,
    // Minutes
    pub minutes: u32,
}

impl From<NaiveDate> for Date {
    fn from(value: NaiveDate) -> Self {
        Self {
            year: value.year(),
            month: value.month(),
            day: value.day(),
        }
    }
}

impl From<Date> for NaiveDate {
    fn from(value: Date) -> Self {
        NaiveDate::from_ymd_opt(value.year, value.month, value.day).unwrap()
    }
}

impl From<NaiveTime> for Time {
    fn from(value: NaiveTime) -> Self {
        Self {
            hour_mil: value.hour(),
            minutes: value.minute(),
        }
    }
}

impl From<Time> for NaiveTime {
    fn from(value: Time) -> Self {
        NaiveTime::from_hms_opt(value.hour_mil, value.minutes, 0).unwrap()
    }
}

impl Time {
    pub fn new(hours: u32, minutes: u32) -> Result<Self, ()> {
        if hours > 23 || minutes > 59 {
            Err(())
        } else {
            Ok(Self {
                hour_mil: hours,
                minutes,
            })
        }
    }
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
        debug_println!("size of callback data: {} bytes", bin_deflate.len());
        log::trace!("size of callback data: {} bytes", bin_deflate.len());

        let bin_chars = BASE64_ENGINE.encode(&bin_deflate);
        debug_println!(
            "callback data payload len: {} bytes, data: \"{}\"",
            bin_chars.len(),
            &bin_chars
        );
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

/// Get the message inside a callback query
fn message_from_callback_query(
    query: &CallbackQuery,
) -> Result<&Message, Box<dyn Error + Send + Sync>> {
    query
        .message
        .as_ref()
        .and_then(|msg| {
            if let MaybeInaccessibleMessage::Regular(m) = msg {
                Some(m)
            } else {
                None
            }
        })
        .ok_or(anyhow!("failed to get message from callback query").into())
}

/// Substitute all text in a message with blank blocks,
/// to visually mark that a callback has been triggered.
/// Set `blank_rows` to the number of blank button rows.
pub async fn replace_with_whitespace(
    bot: Bot,
    msg: &Message,
    blank_rows: usize,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let replacement: String = msg
        .text()
        .unwrap_or(" ")
        .chars()
        .map(|c| if c == '\n' { c } else { BLANK_BLOCK })
        .collect();

    let keyboard = construct_keyboard_tuple(
        (0..blank_rows)
            .into_iter()
            .map(|_| [(" ", Callback::Empty)])
            .collect::<Vec<[(&str, Callback); 1]>>(),
    );

    bot.edit_message_text(msg.chat.id, msg.id, replacement)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Main callback handler
pub async fn callback_handler(
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /// Inner async fn
    async fn inner_handler(bot: Bot, query: CallbackQuery) -> DynResult {
        // answer the callback query once at the top
        bot.answer_callback_query(&query.id).await?;

        let callback_data: Callback = {
            if let Some(data) = &query.data {
                let data_vec = data.as_bytes().to_owned();
                match (&data_vec).try_into() {
                    Ok(d) => d,
                    Err(_) => Callback::Empty,
                }
            } else {
                Callback::Empty
            }
        };

        log::info!("{:?}", callback_data);
        callback_data.handle_callback(bot, query).await
    }

    let handle = tokio::spawn(inner_handler(bot, query));

    tokio::spawn(THREAD_WATCH.push(handle, Duration::from_secs(5)));

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

    /// Tests serializing and deserializing the callback data
    #[test]
    fn test_callback_serde() {
        let callback = Callback::LogSheet(logsheet::LogSheet::StartTime {
            date: chrono::Local::now().date_naive().into(),
            time_slot: false,
            refresh: true,
            start_time: None,
            end_time: None,
            participants_offset: 0,
        });

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
