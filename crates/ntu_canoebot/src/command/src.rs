//! The "src" entry command lives here.
//! The rest of src lives in [crate::callback::src].

use std::error::Error;
use std::str::FromStr;

use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::Me;
// use teloxide::utils::command::BotCommands;

use crate::callback::src::src_menu_create;

use super::HandleCommand;

#[derive(Clone)]
pub struct Src {}

impl FromStr for Src {
    type Err = String;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Src {})
    }
}

#[async_trait]
impl HandleCommand for Src {
    async fn handle_command(
        &self,
        bot: Bot,
        msg: Message,
        _me: Me,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // send a message, do some processing, etc.

        let (text, keyboard) = src_menu_create();

        bot.send_message(msg.chat.id, text)
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
