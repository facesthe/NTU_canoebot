//! Logsheet logic goes here

use std::error::Error;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use ntu_canoebot_attd::{start_end_times, SUBMIT_LOCK};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::frame::{
    common_buttons::{REFRESH, TIME_AM, TIME_PM},
    construct_keyboard, construct_keyboard_tuple,
};

use super::{
    message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback, Time,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogSheet {
    /// Also forces a cache refresh
    Start { date: Date },

    /// Date, time selection and force cache refresh
    StartTime {
        date: Date,
        time_slot: bool,
        refresh: bool,
        start_time: Option<Time>,
        end_time: Option<Time>,
    },

    /// Send
    Send {
        date: Date,
        time_slot: bool,
        start_time: Option<Time>,
        end_time: Option<Time>,
    },

    /// Increment/decrement start/end time, for hours and minutes
    Modify {
        date: Date,
        time_slot: bool,
        start_time: Time,
        end_time: Time,
    },

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
                start_time,
                end_time,
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

                let (mut start, mut end) = start_end_times(*time_slot);

                if let Some(s) = start_time {
                    start = NaiveTime::from(*s)
                };

                if let Some(e) = end_time {
                    end = NaiveTime::from(*e)
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
                    start_time: *start_time,
                    end_time: *end_time,
                });
                let refresh = Callback::LogSheet(LogSheet::StartTime {
                    date: *date,
                    time_slot: *time_slot,
                    refresh: true,
                    start_time: *start_time,
                    end_time: *end_time,
                });
                let cancel = Callback::LogSheet(LogSheet::Cancel {
                    date: *date,
                    time_slot: *time_slot,
                });
                let back = Callback::LogSheet(LogSheet::Start { date: *date });
                let edit = Callback::LogSheet(LogSheet::Modify {
                    date: *date,
                    time_slot: *time_slot,
                    start_time: start_time.unwrap_or(start.into()),
                    end_time: end_time.unwrap_or(end.into()),
                });

                let keyboard = construct_keyboard_tuple([
                    vec![("send", send), (REFRESH, refresh), ("cancel", cancel)],
                    vec![("back", back), ("edit", edit)],
                ]);

                bot.edit_message_text(msg.chat.id, msg.id, text)
                    .reply_markup(keyboard)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
            }
            LogSheet::Send {
                date,
                time_slot,
                start_time,
                end_time,
            } => {
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

                // refac in prog
                if &curr > prev {
                    match ntu_canoebot_attd::logsheet::send(
                        (*date).into(),
                        *time_slot,
                        start_time.and_then(|s| Some(NaiveTime::from(s))),
                        end_time.and_then(|s| Some(NaiveTime::from(s))),
                    )
                    .await
                    {
                        Ok(response) => {
                            *prev = curr;

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
                                Err(anyhow!(
                                    "logsheet sent but failed to submit correctly, status code {}",
                                    response.status()
                                ))?;
                            }
                        }
                        Err(e) => {
                            bot.edit_message_text(
                                msg.chat.id,
                                msg.id,
                                format!("{} unable to be sent. \nError: {}", header, e),
                            )
                            .await?;
                            Err(anyhow!("logsheet failed before sending: {}", e))?;
                        }
                    }
                } else {
                    bot.edit_message_text(msg.chat.id, msg.id, format!("{} sent before", header))
                        .await?;
                }
            }
            LogSheet::Modify {
                date,
                time_slot,
                start_time,
                end_time,
            } => {
                const HOUR: chrono::Duration = chrono::Duration::hours(1);
                const QUARTER: chrono::Duration = chrono::Duration::minutes(15);

                let start_time = NaiveTime::from(*start_time);
                let end_time = NaiveTime::from(*end_time);

                let start_labels = vec!["-1h", "-15min", "start", "+15min", "+1h"];
                let end_labels = vec!["-1h", "-15min", "end", "+15min", "+1h"];

                let callback_from_start_time = |t_start: NaiveTime| -> Callback {
                    Callback::LogSheet(LogSheet::Modify {
                        date: *date,
                        time_slot: *time_slot,
                        start_time: t_start.into(),
                        end_time: end_time.into(),
                    })
                };

                let callback_from_end_time = |t_end: NaiveTime| -> Callback {
                    Callback::LogSheet(LogSheet::Modify {
                        date: *date,
                        time_slot: *time_slot,
                        start_time: start_time.into(),
                        end_time: t_end.into(),
                    })
                };

                let start_data_rows = vec![
                    callback_from_start_time(start_time - HOUR),
                    callback_from_start_time(start_time - QUARTER),
                    Callback::Empty,
                    callback_from_start_time(start_time + QUARTER),
                    callback_from_start_time(start_time + HOUR),
                ];

                let end_data_rows = vec![
                    callback_from_end_time(end_time - HOUR),
                    callback_from_end_time(end_time - QUARTER),
                    Callback::Empty,
                    callback_from_end_time(end_time + QUARTER),
                    callback_from_end_time(end_time + HOUR),
                ];

                let button_labels = vec![start_labels, end_labels, vec!["✔️"]];
                let button_data = vec![
                    start_data_rows,
                    end_data_rows,
                    vec![Callback::LogSheet(LogSheet::StartTime {
                        date: *date,
                        time_slot: *time_slot,
                        refresh: false,
                        start_time: Some(start_time.into()),
                        end_time: Some(end_time.into()),
                    })],
                ];

                let keyboard = construct_keyboard(button_labels, button_data);

                let date_naive = (*date).into();
                let name_list = ntu_canoebot_attd::namelist(date_naive, *time_slot)
                    .await
                    .unwrap_or(ntu_canoebot_attd::NameList::from_date_time(
                        date_naive, *time_slot,
                    ));

                let num_paddlers = name_list.names.len();

                let text = format!(
                    "```\nDate: {}\nTime: {} to {}\nPaddlers: {}\nFetched:  {}```",
                    NaiveDate::from(*date),
                    start_time,
                    end_time,
                    num_paddlers,
                    name_list.fetch_time.format("%H:%M:%S").to_string()
                );

                bot.edit_message_text(msg.chat.id, msg.id, text)
                    .reply_markup(keyboard)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
            }

            LogSheet::Cancel { date, time_slot } => {
                let time = match time_slot {
                    false => TIME_AM,
                    true => TIME_PM,
                };

                let text = format!("Logsheet: {} {} cancelled", NaiveDate::from(*date), time);
                bot.edit_message_text(msg.chat.id, msg.id, text).await?;
            } // _ => todo!(),
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
        start_time: None,
        end_time: None,
    });
    let pm = Callback::LogSheet(LogSheet::StartTime {
        date: d,
        time_slot: true,
        refresh: false,
        start_time: None,
        end_time: None,
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
