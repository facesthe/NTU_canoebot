//! Implementation of the SRC booking menu
//!
#![allow(unused)]

use std::error::Error;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use ntu_src::{SRC_CACHE, SRC_FACILITIES};
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardMarkup, ParseMode},
};

use crate::{
    callback::{self, Callback},
    frame::{
        calendar_month_gen, calendar_year_gen,
        common_buttons::{BACK, BACK_ARROW, FORWARD_ARROW, MONTHS},
        common_descriptions::{CALENDAR, MENU},
        construct_keyboard_tuple, fold_buttons,
    },
};

use super::HandleCallback;

/// The SRC booking menu
///
/// Flattened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Src {
    /// Show the main SRC menu
    Menu,

    /// Show the month calendar
    MonthSelect(String, Date),

    /// Show the year calendar
    YearSelect(String, Date),

    /// Send a query to the cache
    Query(String, Date),

    /// Send a refresh request to the cache
    Refresh(String, Date),

    /// Close the menu
    Close,
}

/// Date struct for [Src::DateSelect]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl From<NaiveDate> for Date {
    fn from(value: NaiveDate) -> Self {
        Self {
            year: value.year(),
            month: value.month(),
            day: value.day(),
        }
    }
}

impl From<Date> for NaiveDate {
    fn from(value: Date) -> Self {
        NaiveDate::from_ymd_opt(value.year, value.month, value.day).unwrap()
    }
}

#[async_trait]
impl HandleCallback for Src {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self {
            Self::Menu => src_menu(bot, query).await,
            Src::MonthSelect(id, date) => src_month_select(&id, date.clone(), bot, query).await,
            Src::Query(id, date) => src_query(id, date.clone(), bot, query).await,
            Src::YearSelect(id, date) => src_year_select(id, date.clone(), bot, query).await,
            Src::Refresh(id, date) => todo!(),
            Src::Close => {
                let msg = message_from_callback_query(&query)?;
                bot.edit_message_text(msg.chat.id, msg.id, "/src").await?;
                Ok(())
            }
        }
    }
}

fn message_from_callback_query(
    query: &CallbackQuery,
) -> Result<&Message, Box<dyn Error + Send + Sync>> {
    Ok(query
        .message
        .as_ref()
        .ok_or(anyhow!("failed to get message from callback query"))?)
}

/// Facility selection.
/// Changes to month selection
async fn src_month_select(
    facil_id: &str,
    date: Date,
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // let facil = SRC_FACILITIES.iter().find(|elem| elem.code_name == facil_id).unwrap();

    log::trace!("facility selection");

    let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();

    let prev = {
        let new = start - Duration::days(1);
        Callback::Src(Src::MonthSelect(facil_id.to_string(), new.into()))
    };
    let next = {
        let new = start + Duration::days(33);
        Callback::Src(Src::MonthSelect(facil_id.to_string(), new.into()))
    };
    let year = Callback::Src(Src::YearSelect(facil_id.to_string(), date));

    let days: Vec<Callback> = (0..31)
        .into_iter()
        .map(|day| {
            let date = start + Duration::days(day);

            Callback::Src(Src::Query(
                facil_id.to_owned(),
                Date {
                    year: date.year(),
                    month: date.month(),
                    day: date.day(),
                },
            ))
        })
        .collect();

    let back = Some(Callback::Src(Src::Menu));

    let keyboard = calendar_month_gen(start, &days, year, next, prev, back);

    let msg = message_from_callback_query(&query)?;
    bot.edit_message_text(msg.chat.id, msg.id, CALENDAR)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn src_year_select(
    facil_id: &str,
    date: Date,
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let buttons: Vec<Callback> = (0..12)
        .into_iter()
        .enumerate()
        .map(|(idx, m)| {
            let date = Date {
                year: date.year,
                month: (idx + 1) as u32,
                day: 1,
            };

            Callback::Src(Src::MonthSelect(facil_id.to_string(), date))
        })
        .collect();

    let next = {
        let date = Date {
            year: date.year + 1,
            month: date.month,
            day: date.day,
        };

        Callback::Src(Src::YearSelect(facil_id.to_string(), date))
    };

    let prev = {
        let date = Date {
            year: date.year - 1,
            month: date.month,
            day: date.day,
        };

        Callback::Src(Src::YearSelect(facil_id.to_string(), date))
    };

    let back = Some(Callback::Src(Src::Menu));

    let keyboard = calendar_year_gen(date.into(), &buttons, next, prev, back);
    let msg = message_from_callback_query(&query)?;

    bot.edit_message_text(msg.chat.id, msg.id, msg.text().unwrap_or(" "))
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Date selection
async fn src_query(
    facil_id: &str,
    date: Date,
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let facil_idx = SRC_FACILITIES
        .iter()
        .enumerate()
        .find_map(|(idx, facil)| {
            if facil.code_name == facil_id {
                Some(idx)
            } else {
                None
            }
        })
        .ok_or(anyhow!("failed to lookup facility id"))?;

    let data = SRC_CACHE
        .get_facility(facil_idx as u8, date.into())
        .await
        .ok_or(anyhow!("failed to retrieve from cache"))?;

    let contents = data.get_display_table(date.into()).ok_or(anyhow!(
        "failed to lookup date. check if date falls within booking entry range"
    ))?;

    let next = Callback::Src(Src::Query(facil_id.to_owned(), {
        let d: NaiveDate = date.into();
        (d + Duration::days(1)).into()
    }));

    let prev = Callback::Src(Src::Query(facil_id.to_owned(), {
        let d: NaiveDate = date.into();
        (d - Duration::days(1)).into()
    }));

    let refresh = Callback::Src(Src::Refresh(facil_id.to_owned(), date));
    let back = Callback::Src(Src::MonthSelect(facil_id.to_string(), date));

    let buttons = vec![
        vec![
            (BACK_ARROW.to_string(), prev),
            (FORWARD_ARROW.to_string(), next),
        ],
        vec![("refresh".to_string(), refresh), (BACK.to_string(), back)],
    ];

    let keyboard = construct_keyboard_tuple(buttons);

    let msg = message_from_callback_query(&query)?;

    bot.edit_message_text(msg.chat.id, msg.id, format!("```\n{}```", contents))
        .reply_markup(keyboard)
        .parse_mode(ParseMode::MarkdownV2)
        .await;

    Ok(())
}

async fn src_menu(bot: Bot, query: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (text, keyboard) = src_menu_create();

    let msg = message_from_callback_query(&query)?;

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Create the message content and inline buttons for the SRC menu.
pub fn src_menu_create() -> (String, InlineKeyboardMarkup) {
    let button_names: Vec<&str> = {
        let mut main_buttons = SRC_FACILITIES
            .iter()
            .map(|facil| facil.short_name.as_str())
            .collect::<Vec<&str>>();

        main_buttons.push("back");
        main_buttons
    };

    let today = chrono::Local::now().date_naive();

    let button_data = {
        let mut main_data = SRC_FACILITIES
            .iter()
            .map(|facil| {
                Callback::Src(callback::src::Src::MonthSelect(
                    facil.code_name.to_owned(),
                    today.into(),
                ))
            })
            .collect::<Vec<Callback>>();
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

    (MENU.to_string(), keyboard)
}
