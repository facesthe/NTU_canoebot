//! The "src" entry command lives here.
//! The rest of src lives in [crate::callback::src].

use std::error::Error;
use std::str::FromStr;

use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::Me;
// use teloxide::utils::command::BotCommands;

use crate::{
    callback::Callback,
    frame::{construct_keyboard_tuple, fold_buttons},
};

use super::HandleCommand;
use crate::callback;

use ntu_src::SRC_FACILITIES;

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

        let button_names: Vec<&str> = {
            let mut main_buttons = SRC_FACILITIES
                .iter()
                .map(|facil| facil.short_name.as_str())
                .collect::<Vec<&str>>();

            main_buttons.push("back");
            main_buttons
        };

        let button_data = {
            let mut main_data = SRC_FACILITIES
                .iter()
                .map(|facil| {
                    Callback::Src(callback::src::Src::FacilitySelect(
                        facil.code_name.to_owned(),
                    ))
                })
                .collect::<Vec<callback::Callback>>();
            main_data.push(Callback::Src(callback::src::Src::Close));
            main_data
        };

        let buttons = button_names
            .iter()
            .zip(button_data)
            .map(|(name, data)| (name.to_string(), data))
            .collect::<Vec<(String, Callback)>>();

        let folded_buttons = fold_buttons(&buttons, 3);

        let keyboard = construct_keyboard_tuple(folded_buttons);

        bot.send_message(msg.chat.id, "choose a SRC facility: ")
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
