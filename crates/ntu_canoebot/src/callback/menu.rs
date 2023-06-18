//! The main menu

use std::error::Error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use super::{booking, HandleCallback};
use crate::frame::{self, common_descriptions};

/// Start menu
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Menu {
    Booking(booking::Booking),
    Status,
    /// The `Here` variant on callback enums indicate that we want to stop and
    /// handle the current enum. In this case, it's the `Menu` enum.
    Here,
}

#[async_trait]
impl HandleCallback for Menu {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self {
            Menu::Booking(_submenu) => _submenu.handle_callback(bot, query).await?,
            Menu::Status => (),
            Menu::Here => {
                if let Some(msg) = query.message {
                    bot.edit_message_text(msg.chat.id, msg.id, common_descriptions::MENU)
                        .reply_markup(frame::main_menu())
                        .await?;
                }
            }
        }

        Ok(())
    }
}
