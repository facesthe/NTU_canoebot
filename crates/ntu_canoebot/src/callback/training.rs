use std::error::Error;

use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use ntu_canoebot_attd::{get_config_type, refresh_prog_sheet_cache, ProgSheet, PROG_CACHE};
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::frame::{calendar_month_gen, calendar_year_gen, date_am_pm_navigation};

use super::{message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback};

use ntu_canoebot_config as config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Training {
    /// Query with a date and time slot.
    ///
    /// Force a cache refresh with the last bool.
    Get(Date, bool, bool),
    /// Month calendar
    MonthSelect(Date),
    /// Year calendar
    YearSelect(Date),
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
            Training::Get(date, time_slot, refresh) => {
                replace_with_whitespace(bot.clone(), &msg, 2).await?;
                training_get(*date, *time_slot, *refresh, bot, msg, true).await?;
            }
            Training::MonthSelect(date) => {
                let start =
                    NaiveDate::from_ymd_opt(date.year.into(), date.month.into(), 1).unwrap();

                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|idx| {
                        let day = start + Duration::days(idx);
                        Callback::Training(Training::Get(day.into(), false, false))
                    })
                    .collect();

                let year = Callback::Training(Training::YearSelect(*date));
                let next =
                    Callback::Training(Training::MonthSelect((start + Duration::days(33)).into()));
                let prev: Callback =
                    Callback::Training(Training::MonthSelect((start - Duration::days(1)).into()));

                let keyboard = calendar_month_gen((*date).into(), &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "training")
                    .reply_markup(keyboard)
                    .await?;
            }
            Training::YearSelect(date) => {
                let months: Vec<Callback> = (1..=12)
                    .into_iter()
                    .map(|m| {
                        Callback::Training(Training::MonthSelect(Date {
                            year: date.year,
                            month: m,
                            day: 1,
                        }))
                    })
                    .collect();

                let next = Callback::Training(Training::YearSelect(Date {
                    year: date.year + 1,
                    month: date.month,
                    day: date.day,
                }));
                let prev = Callback::Training(Training::YearSelect(Date {
                    year: date.year - 1,
                    month: date.month,
                    day: date.day,
                }));

                let keyboard = calendar_year_gen((*date).into(), &months, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, msg.text().unwrap_or(" "))
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
    let sheet_id = match get_config_type(date) {
        ntu_canoebot_attd::Config::Old => *config::SHEETSCRAPER_OLD_PROGRAM_SHEET,
        ntu_canoebot_attd::Config::New => *config::SHEETSCRAPER_NEW_PROGRAM_SHEET,
    };

    let sheet: ProgSheet = {
        let read_lock = PROG_CACHE.read().await;
        match read_lock.contains_date(date) {
            true => {
                if refresh {
                    drop(read_lock);
                    refresh_prog_sheet_cache(true).await.unwrap();
                    PROG_CACHE.read().await.clone()
                } else {
                    read_lock.clone()
                }
            }
            false => {
                let df = g_sheets::get_as_dataframe(sheet_id, Option::<&str>::None).await;
                df.try_into().unwrap()
            }
        }
    };

    let prog = sheet.get_formatted_prog(date, time_slot).unwrap();

    let refresh = Callback::Training(Training::Get(date.into(), time_slot, true));
    let next = Callback::Training(Training::Get(
        (date + Duration::days(1)).into(),
        time_slot,
        false,
    ));
    let prev = Callback::Training(Training::Get(
        (date - Duration::days(1)).into(),
        time_slot,
        false,
    ));
    let slot = Callback::Training(Training::Get(date.into(), !time_slot, false));
    let calendar = Callback::Training(Training::MonthSelect(date.into()));

    let keyboard = date_am_pm_navigation(date, refresh, next, prev, slot, calendar);

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
