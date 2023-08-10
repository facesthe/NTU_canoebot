#![allow(unused)]

use std::error::Error;

use async_trait::async_trait;
use teloxide::prelude::*;

use super::{HandleCallback, Date};


pub enum Paddling {
    /// Perform a lookup, cached
    /// Date, time_slot, refresh
    Get(Date, bool, bool),

    MonthSelect(Date),

    YearSelect(Date)
}

#[async_trait]
impl HandleCallback for Paddling {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
