//! Callbacks for /weeklybreakdown.

use std::error::Error;

use async_trait::async_trait;

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::frame::{
    calendar_month_gen, calendar_year_gen, common_buttons::BLANK, date_am_pm_navigation,
};

use super::{message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Breakdown {
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
impl HandleCallback for Breakdown {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            Breakdown::Get {
                date,
                time_slot,
                refresh,
            } => {
                replace_with_whitespace(bot.clone(), msg, 2).await?;
                breakdown_get((*date).into(), *time_slot, *refresh, bot.clone(), msg, true).await?;
            }
            Breakdown::MonthSelect { date } => {
                let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();

                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|d| {
                        let date = start + Duration::days(d);
                        Callback::Breakdown(Breakdown::Get {
                            date: date.into(),
                            time_slot: false,
                            refresh: false,
                        })
                    })
                    .collect();

                let next = Callback::Breakdown(Breakdown::MonthSelect {
                    date: (start + Duration::days(33)).into(),
                });
                let prev = Callback::Breakdown(Breakdown::MonthSelect {
                    date: (start - Duration::days(1)).into(),
                });
                let year = Callback::Breakdown(Breakdown::YearSelect { date: *date });

                let keyboard = calendar_month_gen(start, &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "weekly breakdown")
                    .reply_markup(keyboard)
                    .await?;
            }
            Breakdown::YearSelect { date } => {
                // let start = NaiveDate::from_ymd_opt(date.year, 1, 1).unwrap();

                let months: Vec<Callback> = (1..=12)
                    .into_iter()
                    .map(|m| {
                        let month = NaiveDate::from_ymd_opt(date.year, m, 1).unwrap();
                        Callback::Breakdown(Breakdown::MonthSelect { date: month.into() })
                    })
                    .collect();

                let next = Callback::Breakdown(Breakdown::YearSelect {
                    date: NaiveDate::from_ymd_opt(date.year + 1, 1, 1).unwrap().into(),
                });
                let prev = Callback::Breakdown(Breakdown::YearSelect {
                    date: NaiveDate::from_ymd_opt(date.year - 1, 1, 1).unwrap().into(),
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

pub async fn breakdown_get(
    date: NaiveDate,
    time_slot: bool,
    refresh: bool,
    bot: Bot,
    msg: &Message,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if refresh {
        ntu_canoebot_attd::refresh_attd_sheet_cache(true)
            .await
            .unwrap();
    }

    let bd = ntu_canoebot_attd::breakdown(date, time_slot).await;

    let prev = Callback::Breakdown(Breakdown::Get {
        date: (date - Duration::days(7)).into(),
        time_slot,
        refresh: false,
    });
    let next = Callback::Breakdown(Breakdown::Get {
        date: (date + Duration::days(7)).into(),
        time_slot,
        refresh: false,
    });
    let time = Callback::Breakdown(Breakdown::Get {
        date: date.into(),
        time_slot: !time_slot,
        refresh,
    });
    let refresh = Callback::Breakdown(Breakdown::Get {
        date: date.into(),
        time_slot,
        refresh: true,
    });
    let calendar = Callback::Breakdown(Breakdown::MonthSelect { date: date.into() });

    let keyboard = date_am_pm_navigation(date, refresh, next, prev, time, calendar, !time_slot);

    let text = format!("```\n{}```", bd);
    match is_callback {
        true => {
            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
        false => {
            bot.send_message(msg.chat.id, text)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
    }

    Ok(())
}
