//! Logsheet logic goes here
#![allow(unused)]

use std::error::Error;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::NaiveDate;
use ntu_canoebot_attd::SUBMIT_LOCK;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use ntu_canoebot_config as config;

use crate::frame::{common_buttons::REFRESH, construct_keyboard_tuple};

use super::{message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogSheet {
    /// Also forces a cache refresh
    Start(Date),

    /// Date, time selection and force cache refresh
    StartTime(Date, bool, bool),

    /// Send
    Send(Date, bool),

    /// Cancel send
    Cancel(Date, bool),
}

#[async_trait]
impl HandleCallback for LogSheet {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            LogSheet::Start(date) => {
                logsheet_start(chrono::Local::now().date_naive(), bot, msg, true).await?
            }

            LogSheet::StartTime(date, time_slot, refresh) => {
                replace_with_whitespace(bot.clone(), &msg, 1).await;

                if *refresh {
                    ntu_canoebot_attd::refresh_attd_sheet_cache(true).await;
                }

                let name_list = ntu_canoebot_attd::namelist((*date).into(), *time_slot)
                    .await
                    .ok_or(anyhow!("no namelist found"))?;

                let num_paddlers = name_list.names.len();

                let (start, end) = match time_slot {
                    false => {
                        let s = (*config::FORMFILLER_TIMES_AM_START).time.unwrap();
                        let e = (*config::FORMFILLER_TIMES_AM_END).time.unwrap();
                        (s, e)
                    }
                    true => {
                        let s = (*config::FORMFILLER_TIMES_PM_START).time.unwrap();
                        let e = (*config::FORMFILLER_TIMES_PM_END).time.unwrap();
                        (s, e)
                    }
                };

                let text = format!(
                    "Date: {}\nTime: {} to {}\nPaddlers: {}",
                    NaiveDate::from(*date),
                    start,
                    end,
                    num_paddlers
                );

                let send = Callback::LogSheet(LogSheet::Send(*date, *time_slot));
                let refresh = Callback::LogSheet(LogSheet::StartTime(*date, *time_slot, true));
                let cancel = Callback::LogSheet(LogSheet::Cancel(*date, *time_slot));

                let keyboard = construct_keyboard_tuple([[
                    ("send", send),
                    (REFRESH, refresh),
                    ("cancel", cancel),
                ]]);

                bot.edit_message_text(msg.chat.id, msg.id, text)
                    .reply_markup(keyboard)
                    .await?;
            }
            LogSheet::Send(date, time_slot) => {
                bot.edit_message_text(msg.chat.id, msg.id, "sending logsheet").await;

                let mut lock = SUBMIT_LOCK.write().await;

                let prev = match time_slot {
                    false => &mut lock.0,
                    true => &mut lock.1,
                };

                let curr: NaiveDate = (*date).into();

                let time = match time_slot {
                    false => "AM",
                    true => "PM",
                };

                // common message header used for responses below
                let header = format!("Logsheet: {} {}", curr, time);

                if &curr > prev {
                    *prev = curr;

                    match ntu_canoebot_attd::logsheet::send((*date).into(), *time_slot).await {
                        Ok(response) => {
                            if StatusCode::is_success(&response.status()) {
                                bot.edit_message_text(
                                    msg.chat.id,
                                    msg.id,
                                    format!("{} sent", header),
                                )
                                .await?;
                            } else {
                                bot.edit_message_text(
                                    msg.chat.id,
                                    msg.id,
                                    format!(
                                        "{} sent with error, status code: {}",
                                        header,
                                        response.status()
                                    ),
                                )
                                .await?;
                                Err(anyhow!("logsheet sent but failed to submit correctly"))?;
                            }
                        }
                        Err(_) => {
                            bot.edit_message_text(
                                msg.chat.id,
                                msg.id,
                                format!(
                                    "{} unable to send. check if the sheet has changed",
                                    header
                                ),
                            )
                            .await?;
                            Err(anyhow!("logsheet failed before send"))?;
                        }
                    }
                } else {
                    bot.edit_message_text(msg.chat.id, msg.id, format!("{} sent before", header))
                        .await?;
                }
            }
            LogSheet::Cancel(date, time_slot) => {
                let time = match time_slot {
                    false => "AM",
                    true => "PM",
                };

                let text = format!("Logsheet: {} {} cancelled", NaiveDate::from(*date), time);
                bot.edit_message_text(msg.chat.id, msg.id, text).await?;
            }
        }

        Ok(())
    }
}

pub async fn logsheet_start(
    date: NaiveDate,
    bot: Bot,
    msg: &Message,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::spawn(ntu_canoebot_attd::refresh_attd_sheet_cache(true));

    let d: Date = date.into();
    let am = Callback::LogSheet(LogSheet::StartTime(d, false, false));
    let pm = Callback::LogSheet(LogSheet::StartTime(d, true, false));

    let keyboard = construct_keyboard_tuple([[("AM", am), ("PM", pm)]]);

    let text = format!("Logsheet: {}", date);
    match is_callback {
        true => {
            bot.edit_message_text(msg.chat.id, msg.id, text)
                .reply_markup(keyboard)
                .await?;
        }
        false => {
            bot.send_message(msg.chat.id, text)
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}
