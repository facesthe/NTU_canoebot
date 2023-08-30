//! Land training prog

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::error::Error;
use teloxide::prelude::*;

use async_trait::async_trait;

use crate::{
    callback::Callback,
    frame::{
        calendar_month_gen, calendar_year_gen,
        common_buttons::{BACK_ARROW, BLANK, DATE, FORWARD_ARROW, REFRESH},
        construct_keyboard_tuple,
    },
};

use super::{message_from_callback_query, replace_with_whitespace, Date, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Land {
    Get { date: Date },
    MonthSelect { date: Date },
    YearSelect { date: Date },
}

#[async_trait]
impl HandleCallback for Land {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            Land::Get { date } => {
                replace_with_whitespace(bot.clone(), msg, 2).await?;
                land_get(bot, msg, (*date).into(), true).await?;
            }
            Land::MonthSelect { date } => {
                let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();
                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|d| {
                        Callback::Land(Land::Get {
                            date: (start + Duration::days(d)).into(),
                        })
                    })
                    .collect();

                let next = Callback::Land(Land::MonthSelect {
                    date: (start + Duration::days(33)).into(),
                });
                let prev = Callback::Land(Land::MonthSelect {
                    date: (start - Duration::days(1)).into(),
                });
                let year = Callback::Land(Land::YearSelect { date: *date });

                let keyboard = calendar_month_gen((*date).into(), &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "land")
                    .reply_markup(keyboard)
                    .await?;
            }

            Land::YearSelect { date } => {
                let months: Vec<Callback> = (1..=12)
                    .into_iter()
                    .map(|m| {
                        Callback::Land(Land::MonthSelect {
                            date: Date {
                                year: date.year,
                                month: m,
                                day: 1,
                            },
                        })
                    })
                    .collect();

                let next = Callback::Land(Land::YearSelect {
                    date: Date {
                        year: date.year + 1,
                        month: 1,
                        day: 1,
                    },
                });
                let prev = Callback::Land(Land::YearSelect {
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

pub async fn land_get(
    bot: Bot,
    msg: &Message,
    date: NaiveDate,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let prev = Callback::Land(Land::Get {
        date: (date - Duration::days(1)).into(),
    });
    let next = Callback::Land(Land::Get {
        date: (date + Duration::days(1)).into(),
    });
    let refresh = Callback::Land(Land::Get { date: date.into() });
    let cal = Callback::Land(Land::MonthSelect { date: date.into() });

    // let keyboard = date_am_pm_navigation(date, refresh, next, prev, time_slot, calendar: cal, time_slot_bool);

    let keyboard = construct_keyboard_tuple([
        vec![
            (BACK_ARROW, prev),
            (REFRESH, refresh),
            (FORWARD_ARROW, next),
        ],
        vec![(DATE, cal)],
    ]);

    let mut prog = ntu_canoebot_attd::land(date).await;
    prog.fill_prog(true).await.unwrap();

    let text = format!("```\n{}```", prog);

    match is_callback {
        true => {
            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(keyboard)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
        false => {
            bot.send_message(msg.chat.id, text)
                .reply_markup(keyboard)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
    }

    Ok(())
}
