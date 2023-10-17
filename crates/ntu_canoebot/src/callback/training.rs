use std::error::Error;

use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use ntu_canoebot_attd::PROG_CACHE;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::frame::{
    calendar_month_gen, calendar_year_gen, common_buttons::BLANK, date_am_pm_navigation,
};

use super::{message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Training {
    /// Query with a date and time slot.
    ///
    /// Force a cache refresh with the last bool.
    Get {
        date: Date,
        time_slot: bool,
        refresh: bool,
    },
    /// Month calendar
    MonthSelect { date: Date },
    /// Year calendar
    YearSelect { date: Date },
}

#[async_trait]
impl HandleCallback for Training {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            Training::Get {
                date,
                time_slot,
                refresh,
            } => {
                replace_with_whitespace(bot.clone(), &msg, 2).await?;
                training_get(*date, *time_slot, *refresh, bot, msg, true).await?;
            }
            Training::MonthSelect { date } => {
                let start =
                    NaiveDate::from_ymd_opt(date.year.into(), date.month.into(), 1).unwrap();

                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|idx| {
                        let day = start + Duration::days(idx);
                        Callback::Training(Training::Get {
                            date: day.into(),
                            time_slot: false,
                            refresh: false,
                        })
                    })
                    .collect();

                let year = Callback::Training(Training::YearSelect { date: *date });
                let next = Callback::Training(Training::MonthSelect {
                    date: (start + Duration::days(33)).into(),
                });
                let prev: Callback = Callback::Training(Training::MonthSelect {
                    date: (start - Duration::days(1)).into(),
                });

                let keyboard = calendar_month_gen((*date).into(), &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "training")
                    .reply_markup(keyboard)
                    .await?;
            }
            Training::YearSelect { date } => {
                let months: Vec<Callback> = (1..=12)
                    .into_iter()
                    .map(|m| {
                        Callback::Training(Training::MonthSelect {
                            date: Date {
                                year: date.year,
                                month: m,
                                day: 1,
                            },
                        })
                    })
                    .collect();

                let next = Callback::Training(Training::YearSelect {
                    date: Date {
                        year: date.year + 1,
                        month: date.month,
                        day: date.day,
                    },
                });
                let prev = Callback::Training(Training::YearSelect {
                    date: Date {
                        year: date.year - 1,
                        month: date.month,
                        day: date.day,
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

pub async fn training_get(
    date: Date,
    time_slot: bool,
    refresh: bool,
    bot: Bot,
    msg: &Message,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let date: NaiveDate = date.into();

    let cache_lock = PROG_CACHE.read().await;
    if cache_lock.contains_date(date) && refresh {
        ntu_canoebot_attd::refresh_prog_sheet_cache(true)
            .await
            .unwrap();
    }

    let sheet = ntu_canoebot_attd::training_prog(date).await;
    let prog = sheet
        .get_formatted_prog(date, time_slot)
        .unwrap_or("".to_string());

    let refresh = Callback::Training(Training::Get {
        date: date.into(),
        time_slot,
        refresh: true,
    });
    let next = Callback::Training(Training::Get {
        date: (date + Duration::days(1)).into(),
        time_slot,
        refresh: false,
    });
    let prev = Callback::Training(Training::Get {
        date: (date - Duration::days(1)).into(),
        time_slot,
        refresh: false,
    });
    let slot = Callback::Training(Training::Get {
        date: date.into(),
        time_slot: !time_slot,
        refresh: false,
    });
    let calendar = Callback::Training(Training::MonthSelect { date: date.into() });

    let keyboard = date_am_pm_navigation(date, refresh, next, prev, slot, calendar, !time_slot);

    let contents = format!("```\n{}```", prog);
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
