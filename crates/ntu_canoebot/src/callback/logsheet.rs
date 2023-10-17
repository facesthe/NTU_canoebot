//! Logsheet logic goes here

use std::error::Error;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::NaiveDate;
use ntu_canoebot_attd::SUBMIT_LOCK;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use ntu_canoebot_config as config;

use crate::frame::{
    common_buttons::{REFRESH, TIME_AM, TIME_PM},
    construct_keyboard_tuple,
};

use super::{message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogSheet {
    /// Also forces a cache refresh
    Start { date: Date },

    /// Date, time selection and force cache refresh
    StartTime {
        date: Date,
        time_slot: bool,
        refresh: bool,
    },

    /// Send
    Send { date: Date, time_slot: bool },

    /// Cancel send
    Cancel { date: Date, time_slot: bool },
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
            LogSheet::Start { date } => logsheet_start((*date).into(), bot, msg, true).await?,

            LogSheet::StartTime {
                date,
                time_slot,
                refresh,
            } => {
                replace_with_whitespace(bot.clone(), &msg, 2).await?;

                if *refresh {
                    ntu_canoebot_attd::refresh_attd_sheet_cache(true)
                        .await
                        .unwrap();
                }

                let date_naive = (*date).into();

                let name_list = ntu_canoebot_attd::namelist(date_naive, *time_slot)
                    .await
                    .unwrap_or(ntu_canoebot_attd::NameList::from_date_time(
                        date_naive, *time_slot,
                    ));

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
                    "```\nDate: {}\nTime: {} to {}\nPaddlers: {}\nFetched:  {}```",
                    NaiveDate::from(*date),
                    start,
                    end,
                    num_paddlers,
                    name_list.fetch_time.format("%H:%M:%S").to_string()
                );

                let send = Callback::LogSheet(LogSheet::Send {
                    date: *date,
                    time_slot: *time_slot,
                });
                let refresh = Callback::LogSheet(LogSheet::StartTime {
                    date: *date,
                    time_slot: *time_slot,
                    refresh: true,
                });
                let cancel = Callback::LogSheet(LogSheet::Cancel {
                    date: *date,
                    time_slot: *time_slot,
                });
                let back = Callback::LogSheet(LogSheet::Start { date: *date });

                let keyboard = construct_keyboard_tuple([
                    vec![("send", send), (REFRESH, refresh), ("cancel", cancel)],
                    vec![("back", back)],
                ]);

                bot.edit_message_text(msg.chat.id, msg.id, text)
                    .reply_markup(keyboard)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
            }
            LogSheet::Send { date, time_slot } => {
                bot.edit_message_text(msg.chat.id, msg.id, "sending logsheet")
                    .await?;

                let mut lock = SUBMIT_LOCK.write().await;

                let prev = match time_slot {
                    false => &mut lock.0,
                    true => &mut lock.1,
                };

                let curr: NaiveDate = (*date).into();

                let time = match time_slot {
                    false => TIME_AM,
                    true => TIME_PM,
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
            LogSheet::Cancel { date, time_slot } => {
                let time = match time_slot {
                    false => TIME_AM,
                    true => TIME_PM,
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
    let am = Callback::LogSheet(LogSheet::StartTime {
        date: d,
        time_slot: false,
        refresh: false,
    });
    let pm = Callback::LogSheet(LogSheet::StartTime {
        date: d,
        time_slot: true,
        refresh: false,
    });

    let keyboard = construct_keyboard_tuple([[(TIME_AM, am), (TIME_PM, pm)]]);

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
