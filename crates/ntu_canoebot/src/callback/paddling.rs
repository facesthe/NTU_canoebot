use std::error::Error;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::frame::{
    calendar_month_gen, calendar_year_gen,
    common_buttons::{BACK_ARROW, DATE, FORWARD_ARROW, REFRESH},
    construct_keyboard_tuple,
};

use super::{message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Paddling {
    /// Perform a lookup, cached.
    ///
    /// Date, time_slot, deconflict, refresh
    Get {
        date: Date,
        time_slot: bool,
        deconflict: bool,
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
impl HandleCallback for Paddling {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            Paddling::Get {
                date,
                time_slot,
                deconflict,
                refresh,
            } => {
                replace_with_whitespace(bot.clone(), msg, 3).await?;
                paddling_get(
                    (*date).into(),
                    *time_slot,
                    *deconflict,
                    *refresh,
                    bot,
                    msg,
                    true,
                )
                .await?;
            }
            Paddling::MonthSelect { date } => {
                let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();

                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|d| {
                        let day: Date = (start + Duration::days(d)).into();
                        Callback::Padddling(Paddling::Get {
                            date: day,
                            time_slot: false,
                            deconflict: true,
                            refresh: false,
                        })
                    })
                    .collect();

                let next = Callback::Padddling(Paddling::MonthSelect {
                    date: (start + Duration::days(33)).into(),
                });
                let prev = Callback::Padddling(Paddling::MonthSelect {
                    date: (start - Duration::days(1)).into(),
                });
                let year = Callback::Padddling(Paddling::YearSelect { date: date.clone() });

                let keyboard = calendar_month_gen((*date).into(), &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "paddling")
                    .reply_markup(keyboard)
                    .await?;
            }
            Paddling::YearSelect { date } => {
                let months: Vec<Callback> = (0..12)
                    .into_iter()
                    .map(|m| {
                        let month = Date {
                            year: date.year,
                            month: m + 1,
                            day: 1,
                        };

                        Callback::Padddling(Paddling::MonthSelect { date: month })
                    })
                    .collect();

                let next = Callback::Padddling(Paddling::YearSelect {
                    date: Date {
                        year: date.year + 1,
                        month: 1,
                        day: 1,
                    },
                });
                let prev = Callback::Padddling(Paddling::YearSelect {
                    date: Date {
                        year: date.year - 1,
                        month: 1,
                        day: 1,
                    },
                });

                let keyboard = calendar_year_gen((*date).into(), &months, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, msg.text().unwrap_or(""))
                    .reply_markup(keyboard)
                    .await?;
            }
        }

        Ok(())
    }
}

/// Main paddling function
pub async fn paddling_get(
    date: NaiveDate,
    time_slot: bool,
    deconflict: bool,
    refresh: bool,
    bot: Bot,
    msg: &Message,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let date_n: NaiveDate = (date).into();

    if refresh {
        let handles = [
            tokio::spawn(async move { ntu_canoebot_attd::refresh_attd_sheet_cache(true).await }),
            tokio::spawn(async move { ntu_canoebot_attd::refresh_prog_sheet_cache(true).await }),
        ];

        for handle in handles {
            let _ = handle.await.unwrap();
        }
    }

    let mut name_list = ntu_canoebot_attd::namelist(date_n, time_slot)
        .await
        .ok_or(anyhow!("failed to get namelist"))?;

    name_list.assign_boats(deconflict).await;
    name_list.paddling().await.unwrap();

    let d: Date = date.into();
    let prev = Callback::Padddling(Paddling::Get {
        date: (date_n - Duration::days(1)).into(),
        time_slot,
        deconflict,
        refresh: false,
    });
    let next = Callback::Padddling(Paddling::Get {
        date: (date_n + Duration::days(1)).into(),
        time_slot,
        deconflict,
        refresh: false,
    });

    // switch between deconf modes
    let refresh = Callback::Padddling(Paddling::Get {
        date: d,
        time_slot,
        deconflict,
        refresh: true,
    });
    let switch = Callback::Padddling(Paddling::Get {
        date: d,
        time_slot,
        deconflict: !deconflict,
        refresh: false,
    });
    let time = Callback::Padddling(Paddling::Get {
        date: d,
        time_slot: !time_slot,
        deconflict,
        refresh: false,
    });
    let month = Callback::Padddling(Paddling::MonthSelect { date: d });

    let switch_label = if deconflict { "plain" } else { "deconf" };
    let time_label = if time_slot { "AM" } else { "PM" };

    let keyboard = construct_keyboard_tuple([
        vec![
            (BACK_ARROW, prev),
            (REFRESH, refresh),
            (FORWARD_ARROW, next),
        ],
        vec![(switch_label, switch), (time_label, time)],
        vec![(DATE, month)],
    ]);

    let text = format!("```\n{}```", name_list);

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
