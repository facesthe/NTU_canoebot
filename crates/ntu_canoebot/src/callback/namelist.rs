//! Namelist callbacks
//!

use std::error::Error;

use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use ntu_canoebot_util::debug_println;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::{
    callback::message_from_callback_query,
    frame::{calendar_month_gen, calendar_year_gen, common_buttons::BLANK, date_am_pm_navigation},
};

use super::{replace_with_whitespace, Callback, Date, HandleCallback};

/// Callbacks for /namelist
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NameList {
    /// Get the namelist for a particular date
    Get {
        date: Date,
        time_slot: bool,
        refresh: bool,
    },

    MonthSelect {
        date: Date,
    },

    YearSelect {
        date: Date,
    },
}

#[async_trait]
impl HandleCallback for NameList {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        debug_println!("namelist callback triggered");

        let msg = message_from_callback_query(&query)?;

        match self {
            NameList::Get {
                date,
                time_slot,
                refresh,
            } => {
                // let msg_cloned = msg.clone();
                // tokio::spawn(replace_with_whitespace(bot.clone(), &msg_cloned, 2));
                replace_with_whitespace(bot.clone(), &msg, 2).await?;
                namelist_get(date.to_owned(), *time_slot, *refresh, bot, msg, true).await?
            }
            NameList::MonthSelect { date } => {
                // replace_with_whitespace(bot.clone(), &msg, 2).await?;
                let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();
                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|d| {
                        Callback::NameList(NameList::Get {
                            date: (start + Duration::days(d as i64)).into(),
                            time_slot: false,
                            refresh: false,
                        })
                    })
                    .collect();

                let year = Callback::NameList(NameList::YearSelect {
                    date: date.to_owned(),
                });
                let prev = Callback::NameList(NameList::MonthSelect {
                    date: (start - Duration::days(1)).into(),
                });
                let next = Callback::NameList(NameList::MonthSelect {
                    date: (start + Duration::days(33)).into(),
                });

                let keyboard = calendar_month_gen((*date).into(), &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "namelist")
                    .reply_markup(keyboard)
                    .await?;
            }
            NameList::YearSelect { date } => {
                // replace_with_whitespace(bot.clone(), &msg, 2).await?;
                let months: Vec<Callback> = (0..12)
                    .into_iter()
                    .map(|m| {
                        let month = Date {
                            year: date.year,
                            month: 1 + m,
                            day: 1,
                        };

                        Callback::NameList(NameList::MonthSelect { date: month })
                    })
                    .collect();

                let next = Callback::NameList(NameList::YearSelect {
                    date: Date {
                        year: date.year + 1,
                        month: 1,
                        day: 1,
                    },
                });
                let prev = Callback::NameList(NameList::YearSelect {
                    date: Date {
                        year: date.year - 1,
                        month: 1,
                        day: 1,
                    },
                });

                let keyboard = calendar_year_gen((*date).into(), &months, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, msg.text().unwrap_or(BLANK))
                    .reply_markup(keyboard)
                    .await?;
            }
        }

        Ok(())
    }
}

/// Perform a namelist get operation.
/// If an entry exists in cache and refresh is not triggered, it will pull data from the cache.
pub async fn namelist_get(
    date: Date,
    time_slot: bool,
    refresh: bool,
    bot: Bot,
    msg: &Message,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let date: NaiveDate = date.into();

    if refresh {
        ntu_canoebot_attd::refresh_attd_sheet_cache(true)
            .await
            .unwrap();
    }

    let list = ntu_canoebot_attd::namelist(date, time_slot)
        .await
        .unwrap_or(ntu_canoebot_attd::NameList::from_date_time(date, time_slot));

    // generate keyboard
    let prev = Callback::NameList(NameList::Get {
        date: {
            let d: NaiveDate = date.into();
            (d - Duration::days(1)).into()
        },
        time_slot,
        refresh: false,
    });

    let next = Callback::NameList(NameList::Get {
        date: {
            let d: NaiveDate = date.into();
            (d + Duration::days(1)).into()
        },
        time_slot,
        refresh: false,
    });

    let refresh = Callback::NameList(NameList::Get {
        date: date.into(),
        time_slot,
        refresh: true,
    });

    let time = Callback::NameList(NameList::Get {
        date: date.into(),
        time_slot: !time_slot,
        refresh: false,
    });

    let calendar = Callback::NameList(NameList::MonthSelect { date: date.into() });
    let keyboard = date_am_pm_navigation(date, refresh, next, prev, time, calendar, !time_slot);

    let contents = format!("```\n{}```", list);

    match is_callback {
        true => {
            bot.edit_message_text(msg.chat.id, msg.id, contents)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
        false => {
            bot.send_message(msg.chat.id, contents)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
    }

    Ok(())
}
