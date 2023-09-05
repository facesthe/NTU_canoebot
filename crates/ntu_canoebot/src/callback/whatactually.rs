//! Callbacks for the urban dictionary

use std::error::Error;

use async_trait::async_trait;
use ntu_canoebot_util::HiddenString;

use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::dictionaries;

use super::{message_from_callback_query, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhatActually {
    Get { query: HiddenString },
}

#[async_trait]
impl HandleCallback for WhatActually {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            WhatActually::Get { query } => {
                let res = dictionaries::urbandictonary::query(query.as_str()).await;

                if let Some(result) = res {

                }
            }
        }
        Ok(())
    }
}
