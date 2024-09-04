//! Individual callback handlers / family of callback handlers are defined here.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use super::HandleCallback;

/// Blank callback (empty/unused buttons)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Empty {}

#[async_trait]
impl HandleCallback for Empty {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(msg) = query.message {
            bot.edit_message_text(msg.chat().id, msg.id(), "button pressed")
                .await?;
        }
        Ok(())
    }
}
