//! Implementation of the SRC booking menu
//!
#![allow(unused)]

use std::error::Error;

use async_trait::async_trait;
use ntu_src::SRC_FACILITIES;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use super::HandleCallback;

/// The SRC booking menu
///
/// Flattened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Src {
    /// Select a facility returning the facility ID
    FacilitySelect(String),

    /// Select a date, returning the facility ID and date
    DateSelect(String, Date),

    /// Change the month in the inline markup calendar
    MonthSelect(String, Date),

    /// Send a request to the cache for a refresh
    Refresh(String, Date),

    /// Close the menu
    Close,
}

/// Date struct for [Src::DateSelect]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[async_trait]
impl HandleCallback for Src {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self {
            Src::FacilitySelect(id) => facility_select(&id, bot, query).await,
            Src::DateSelect(id, date) => Ok(()),
            Src::MonthSelect(id, date) => todo!(),
            Src::Refresh(id, date) => todo!(),
            Src::Close => todo!(),
        }
    }
}

/// Facility selection.
/// Adds calendar selection in keyboard
async fn facility_select(
    facil_id: &str,
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // let facil = SRC_FACILITIES.iter().find(|elem| elem.code_name == facil_id).unwrap();

    todo!()
    // Ok(())
}
