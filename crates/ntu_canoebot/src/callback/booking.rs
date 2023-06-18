//! The booking menu

use std::error::Error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use super::{Callback, HandleCallback};
use crate::frame::{self, common_buttons, common_descriptions};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BTP {}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BTE {}

/// The booking sub-menu
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Booking {
    BTP(BTP),
    BTE(BTE),
    // RTP(Credentials),
    // RTE(Credentials),
    // Practical(Credentials),
    // RC(Credentials),
    // RR(Credentials),
    /// The `Here` variant of an enum means that we want to stop diving into
    /// enums and process the current one. In this case, it's the Booking enum,
    /// so we want to handle showing the booking menu.
    Here,
}

#[async_trait]
impl HandleCallback for Booking {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self {
            Booking::BTP(btp) => btp.handle_callback(bot, query).await?,
            Booking::BTE(bte) => bte.handle_callback(bot, query).await?,
            Booking::Here => {
                if let Some(msg) = query.message {
                    bot.edit_message_text(msg.chat.id, msg.id, common_descriptions::BOOKING)
                        .reply_markup(frame::booking_menu(0))
                        .await?;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl HandleCallback for BTP {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(msg) = query.message {
            bot.edit_message_text(msg.chat.id, msg.id, "in the BTP booking section")
                .reply_markup(frame::construct_keyboard(
                    [[common_buttons::BACK]],
                    [[Callback::Menu(super::Menu::Booking(Booking::Here))]],
                ))
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl HandleCallback for BTE {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(msg) = query.message {
            bot.edit_message_text(msg.chat.id, msg.id, "in the BTE booking section")
                .reply_markup(frame::construct_keyboard(
                    [[common_buttons::BACK]],
                    [[Callback::Menu(super::Menu::Booking(Booking::Here))]],
                ))
                .await?;
        }
        Ok(())
    }
}
