//! Special events go here

use std::error::Error;

use chrono::Duration;
use lazy_static::lazy_static;
use teloxide::prelude::*;
use teloxide::types::{Chat, Message, MessageId};

use ntu_canoebot_config as config;

use crate::callback::Callback;
use crate::frame::construct_keyboard_tuple;

lazy_static! {
    /// The exco chat id, parsed as an option
    pub static ref EXCO_CHAT_ID: Option<i64> = {
        if config::CANOEBOT_EXCO_CHAT == 0 {
            None
        } else {
            Some(config::CANOEBOT_EXCO_CHAT)
        }
    };
}

/// The only thing that's valid here is the `chat.id`.
#[allow(unused)]
fn message_from_chat_id(chat_id: i64) -> Message {
    let chat = Chat {
        id: ChatId(chat_id),
        kind: teloxide::types::ChatKind::Private(teloxide::types::ChatPrivate {
            username: None,
            first_name: None,
            last_name: None,
            emoji_status_custom_emoji_id: None,
            bio: None,
            has_private_forwards: None,
            has_restricted_voice_and_video_messages: None,
        }),
        photo: None,
        pinned_message: None,
        message_auto_delete_time: None,
        has_hidden_members: false,
        has_aggressive_anti_spam_enabled: false,
    };

    Message {
        id: MessageId(0),
        thread_id: None,
        date: chrono::Utc::now(),
        chat,
        via_bot: None,
        kind: teloxide::types::MessageKind::ChannelChatCreated(
            teloxide::types::MessageChannelChatCreated {
                channel_chat_created: teloxide::types::True,
            },
        ),
    }
}

/// Send the logsheet to exco chat
pub async fn logsheet_prompt(bot: Bot) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(chat_id) = *EXCO_CHAT_ID {
        log::info!("logsheet prompt");

        let read_lock = ntu_canoebot_attd::logsheet::SUBMIT_LOCK.read().await;
        if read_lock.0 >= chrono::Local::now().date_naive() {
            log::info!("logsheet sent before event");
            return Ok(());
        }

        let now = chrono::Local::now().date_naive();
        let keyboard = construct_keyboard_tuple([[(
            "logsheet",
            Callback::LogSheet(crate::callback::LogSheet::Start { date: now.into() }),
        )]]);

        bot.send_message(ChatId(chat_id), "logsheet")
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

pub async fn attendance_prompt(bot: Bot) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(chat_id) = *EXCO_CHAT_ID {
        log::info!("attendance prompt");

        let now = chrono::Local::now().date_naive();
        let keyboard = construct_keyboard_tuple([[(
            "paddling",
            Callback::Paddling(crate::callback::Paddling::Get {
                date: (now + Duration::days(1)).into(),
                time_slot: false,
                freshies: false,
                deconflict: true,
                refresh: false,
                excluded_fields: u64::MAX,
                show_blanks: true,
            }),
        )]]);

        bot.send_message(ChatId(chat_id), "paddling")
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

pub async fn breakdown_prompt(bot: Bot) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(chat_id) = *EXCO_CHAT_ID {
        log::info!("breakdown prompt");

        let now = chrono::Local::now().date_naive() + Duration::days(7);
        let keyboard = construct_keyboard_tuple([[(
            "breakdown",
            Callback::Breakdown(crate::callback::Breakdown::Get {
                date: now.into(),
                time_slot: false,
                refresh: false,
            }),
        )]]);

        bot.send_message(ChatId(chat_id), "breakdown")
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}
