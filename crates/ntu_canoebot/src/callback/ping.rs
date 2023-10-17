//! Callbacks for the /ping command

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, time::Duration};
use teloxide::{prelude::*, types::MessageId};

use crate::frame::construct_keyboard_tuple;

use super::{message_from_callback_query, Callback, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Ping {
    Next { confirm: bool, msg_id: i32 },
}

#[async_trait]
impl HandleCallback for Ping {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        let (text, msg_id) = match self {
            Ping::Next { confirm, msg_id } => {
                let msg_text = match confirm {
                    true => {
                        log::warn!("{:#?}", msg);
                        "chat logged. this message will be deleted."
                    }
                    false => "this message will be deleted.",
                };

                (msg_text, *msg_id)
            }
        };
        bot.edit_message_text(msg.chat.id, msg.id, text).await?;
        tokio::time::sleep(Duration::from_secs(10)).await;
        bot.delete_message(msg.chat.id, msg.id).await?;
        let res = bot.delete_message(msg.chat.id, MessageId(msg_id)).await;
        if let Err(e) = res {
            log::info!("unable to delete user message: {}", e)
        }
        Ok(())
    }
}

pub async fn ping_start(bot: Bot, msg: &Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    let text = "/ping logs YOUR chat and user data. Proceed?";

    let keyboard = construct_keyboard_tuple([[
        (
            "yes",
            Callback::Ping(Ping::Next {
                confirm: true,
                msg_id: msg.id.0,
            }),
        ),
        (
            "no",
            Callback::Ping(Ping::Next {
                confirm: false,
                msg_id: msg.id.0,
            }),
        ),
    ]]);

    bot.send_message(msg.chat.id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
