//! Individual command handlers are defined in this file.

use std::error::Error;
use std::str::FromStr;

use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Me};
use teloxide::utils::command::BotCommands;

use super::{Commands, HandleCommand};
use crate::callback::Callback;

/// Unit struct to carry trait implementations.
/// This separates and simplifies writing code: each command has it's own
/// impl block.
#[derive(Clone, Debug)]
pub struct Help {}

// Mandatory implementation for main Commands enum
impl FromStr for Help {
    type Err = String;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Help {})
    }
}

// Command handler for this particular command
#[async_trait]
impl HandleCommand for Help {
    async fn handle_command(
        &self,
        bot: Bot,
        msg: Message,
        _me: Me,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        bot.send_message(msg.chat.id, Commands::descriptions().to_string())
            .await?;

        Ok(())
    }
}

/// The start command handles bot-user initialisation.
#[derive(Clone, Debug)]
pub struct Start {}

impl FromStr for Start {
    type Err = String;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Start {})
    }
}

#[async_trait]
impl HandleCommand for Start {
    async fn handle_command(
        &self,
        _bot: Bot,
        _msg: Message,
        _me: Me,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Button {}

impl FromStr for Button {
    type Err = String;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Button {})
    }
}

#[async_trait]
impl HandleCommand for Button {
    async fn handle_command(
        &self,
        bot: Bot,
        msg: Message,
        _me: Me,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        bot.send_message(msg.chat.id, "Here is your button")
            .reply_markup(single_inline_button("button", Callback::Empty))
            .await?;

        Ok(())
    }
}

fn single_inline_button(name: &str, callback: Callback) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    keyboard.push(vec![InlineKeyboardButton::callback(name, callback)]);

    InlineKeyboardMarkup::new(keyboard)
}
