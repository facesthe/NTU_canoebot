//! Implementation of the SRC booking menu
//!

use std::error::Error;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Datelike, Duration, NaiveDate};
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
        common_buttons::{BACK, BACK_ARROW, BLANK, FORWARD_ARROW},
        common_descriptions::MENU,
        construct_keyboard_tuple, fold_buttons,
    },
};

use super::{message_from_callback_query, replace_with_whitespace, Date, HandleCallback};

/// The SRC booking menu
///
/// Flattened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Src {
    /// Show the main SRC menu
    Menu,

    /// Show the month calendar
    MonthSelect { id: String, date: Date },

    /// Show the year calendar
    YearSelect { id: String, date: Date },

    /// Send a query to the cache.
    /// Set the bool to `true` to perform a refresh
    Query {
        id: String,
        date: Date,
        refresh: bool,
    },

    // /// Send a refresh request to the cache
    // Refresh(String, Date),
    /// Close the menu
    Close,
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
            Src::MonthSelect { id, date } => src_month_select(&id, date.clone(), bot, query).await,
            Src::YearSelect { id, date } => src_year_select(id, date.clone(), bot, query).await,
            Src::Query { id, date, refresh } => {
                src_query(id, date.clone(), *refresh, bot, query).await
            }
            Src::Close => {
                let msg = message_from_callback_query(&query)?;
                bot.edit_message_text(msg.chat.id, msg.id, "/src").await?;
                Ok(())
            }
        }
    }
}

async fn src_menu(bot: Bot, query: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (text, keyboard) = src_menu_create();

    let msg = message_from_callback_query(&query)?;

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Facility selection.
/// Changes to month selection
async fn src_month_select(
    facil_id: &str,
    date: Date,
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::trace!("facility selection");

    let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();
    let facil_name = SRC_FACILITIES
        .iter()
        .find_map(|f| {
            if facil_id == f.code_name {
                Some(f.name.as_str())
            } else {
                None
            }
        })
        .ok_or(anyhow!("failed to get facility id. did the config change?"))?;

    let prev = {
        let new = start - Duration::days(1);
        Callback::Src(Src::MonthSelect {
            id: facil_id.to_string(),
            date: new.into(),
        })
    };
    let next = {
        let new = start + Duration::days(33);
        Callback::Src(Src::MonthSelect {
            id: facil_id.to_string(),
            date: new.into(),
        })
    };
    let year = Callback::Src(Src::YearSelect {
        id: facil_id.to_string(),
        date,
    });

    let days: Vec<Callback> = (0..31)
        .into_iter()
        .map(|day| {
            let date = start + Duration::days(day);

            Callback::Src(Src::Query {
                id: facil_id.to_owned(),
                date: Date {
                    year: date.year(),
                    month: date.month(),
                    day: date.day(),
                },
                refresh: false,
            })
        })
        .collect();

    let back = Some(Callback::Src(Src::Menu));

    let keyboard = calendar_month_gen(start, &days, year, next, prev, back);

    let msg = message_from_callback_query(&query)?;
    bot.edit_message_text(msg.chat.id, msg.id, facil_name)
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
        .map(|m| {
            let date = Date {
                year: date.year,
                month: m + 1,
                day: 1,
            };

            Callback::Src(Src::MonthSelect {
                id: facil_id.to_string(),
                date,
            })
        })
        .collect();

    let next = {
        let date = Date {
            year: date.year + 1,
            month: date.month,
            day: date.day,
        };

        Callback::Src(Src::YearSelect {
            id: facil_id.to_string(),
            date,
        })
    };

    let prev = {
        let date = Date {
            year: date.year - 1,
            month: date.month,
            day: date.day,
        };

        Callback::Src(Src::YearSelect {
            id: facil_id.to_string(),
            date,
        })
    };

    let back = Some(Callback::Src(Src::Menu));

    let keyboard = calendar_year_gen(date.into(), &buttons, next, prev, back);
    let msg = message_from_callback_query(&query)?;

    bot.edit_message_text(msg.chat.id, msg.id, msg.text().unwrap_or(BLANK))
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Generate navigation buttons
fn src_navigation_buttons(facil_id: &str, date: Date) -> InlineKeyboardMarkup {
    let next = Callback::Src(Src::Query {
        id: facil_id.to_owned(),
        date: {
            let d: NaiveDate = date.into();
            (d + Duration::days(1)).into()
        },
        refresh: false,
    });

    let prev = Callback::Src(Src::Query {
        id: facil_id.to_owned(),
        date: {
            let d: NaiveDate = date.into();
            (d - Duration::days(1)).into()
        },
        refresh: false,
    });

    let refresh = Callback::Src(Src::Query {
        id: facil_id.to_owned(),
        date,
        refresh: true,
    });
    let back = Callback::Src(Src::MonthSelect {
        id: facil_id.to_string(),
        date,
    });

    let buttons = vec![
        vec![
            (BACK_ARROW.to_string(), prev),
            (FORWARD_ARROW.to_string(), next),
        ],
        vec![("refresh".to_string(), refresh), (BACK.to_string(), back)],
    ];

    construct_keyboard_tuple(buttons)
}

async fn src_query(
    facil_id: &str,
    date: Date,
    refresh: bool,
    bot: Bot,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let msg = message_from_callback_query(&query)?;
    replace_with_whitespace(bot.clone(), msg, 2).await?;

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
        .get_facility(facil_idx as u8, date.into(), refresh)
        .await
        .ok_or(anyhow!("failed to retrieve from cache"))?;

    let contents = data.get_display_table(date.into()).ok_or(anyhow!(
        "failed to lookup date. check if date falls within booking entry range"
    ))?;

    let keyboard = src_navigation_buttons(facil_id, date);

    bot.edit_message_text(msg.chat.id, msg.id, format!("```\n{}```", contents))
        .reply_markup(keyboard)
        .parse_mode(ParseMode::MarkdownV2)
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

        main_buttons.push("close");
        main_buttons
    };

    let today = chrono::Local::now().date_naive();

    let button_data = {
        let mut main_data = SRC_FACILITIES
            .iter()
            .map(|facil| {
                Callback::Src(callback::src::Src::MonthSelect {
                    id: facil.code_name.to_owned(),
                    date: today.into(),
                })
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

#[cfg(test)]
mod tests {

    const ASD: char = '\u{FE2D}';

    #[test]
    fn test_underline() {
        let asd = "this is a string";

        let underlined = asd
            .chars()
            .map(|c| [c, ASD].iter().collect::<String>())
            .collect::<Vec<String>>()
            .concat();

        println!("{}", underlined);
    }
}
