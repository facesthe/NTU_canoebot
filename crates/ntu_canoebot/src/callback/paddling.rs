use std::error::Error;
use std::ops;

use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use ntu_canoebot_attd::{BitIndices, NameList};
use ntu_canoebot_traits::{DeriveEnumParent, EnumParent};
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::frame::{
    calendar_month_gen, calendar_year_gen,
    common_buttons::{BACK_ARROW, BLANK, DATE, FORWARD_ARROW, REFRESH, TIME_AM, TIME_PM},
    construct_keyboard_tuple, convert_to_2d,
};

use super::{message_from_callback_query, replace_with_whitespace, Callback, Date, HandleCallback};

/// Method to filter names by
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub enum FilterType {
    /// Each callback adds to the main list
    /// and removes a name from the exclude list.
    Add,
    /// Each callback removes from the main list
    /// and adds a name to the exclude list.
    #[default]
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, DeriveEnumParent)]
#[enum_parent(Callback::Paddling(_))]
pub enum Paddling {
    /// Perform a lookup, cached.
    Get {
        date: Date,
        /// true == AM, false == PM
        time_slot: bool,
        freshies: bool,
        /// perform deconflict
        deconflict: bool,
        refresh: bool,
        excluded_fields: u64,
        /// Show blank blocks when updating the message
        show_blanks: bool,
    },

    MonthSelect {
        date: Date,
        freshies: bool,
    },

    YearSelect {
        date: Date,
        freshies: bool,
    },

    /// Exclude names from the boat allocation.
    ///
    /// Used when there are lots of people doing team boats.
    ExcludeSelection {
        // these fields will have their state frozen
        // until the user decides to complete the exclude
        date: Date,
        time_slot: bool,
        freshies: bool,
        deconflict: bool,
        refresh: bool,

        /// To be converted to [ntu_canoebot_attd::BitIndices]
        excluded_fields: u64,

        exclude_type: FilterType,
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
                freshies,
                deconflict,
                refresh,
                excluded_fields,
                show_blanks,
            } => {
                if *show_blanks {
                    replace_with_whitespace(bot.clone(), msg, 3).await?;
                }

                paddling_get(
                    (*date).into(),
                    *time_slot,
                    *freshies,
                    *deconflict,
                    *refresh,
                    *excluded_fields,
                    bot,
                    msg,
                    true,
                )
                .await?;
            }
            Paddling::MonthSelect { date, freshies } => {
                let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();

                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|d| {
                        let day: Date = (start + Duration::days(d)).into();
                        Self::enum_parent(Self::Get {
                            date: day,
                            time_slot: false,
                            freshies: *freshies,
                            deconflict: true,
                            refresh: false,
                            excluded_fields: u64::MAX,
                            show_blanks: true,
                        })
                    })
                    .collect();

                let next = Self::enum_parent(Self::MonthSelect {
                    date: (start + Duration::days(33)).into(),
                    freshies: *freshies,
                });
                let prev = Self::enum_parent(Self::MonthSelect {
                    date: (start - Duration::days(1)).into(),
                    freshies: *freshies,
                });
                let year = Self::enum_parent(Self::YearSelect {
                    date: date.clone(),
                    freshies: *freshies,
                });

                let keyboard = calendar_month_gen((*date).into(), &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "paddling")
                    .reply_markup(keyboard)
                    .await?;
            }
            Paddling::YearSelect { date, freshies } => {
                let months: Vec<Callback> = (0..12)
                    .into_iter()
                    .map(|m| {
                        let month = Date {
                            year: date.year,
                            month: m + 1,
                            day: 1,
                        };

                        Self::enum_parent(Self::MonthSelect {
                            date: month,
                            freshies: *freshies,
                        })
                    })
                    .collect();

                let next = Self::enum_parent(Self::YearSelect {
                    date: Date {
                        year: date.year + 1,
                        month: 1,
                        day: 1,
                    },
                    freshies: *freshies,
                });
                let prev = Self::enum_parent(Self::YearSelect {
                    date: Date {
                        year: date.year - 1,
                        month: 1,
                        day: 1,
                    },
                    freshies: *freshies,
                });

                let keyboard = calendar_year_gen((*date).into(), &months, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, msg.text().unwrap_or(BLANK))
                    .reply_markup(keyboard)
                    .await?;
            }

            Paddling::ExcludeSelection {
                date,
                time_slot,
                freshies,
                deconflict,
                refresh,
                excluded_fields,
                exclude_type,
            } => {
                let date_n = (*date).into();
                let mut name_list = ntu_canoebot_attd::namelist(date_n, *time_slot, *freshies)
                    .await
                    .unwrap_or(NameList::from_date_time(date_n, *time_slot));

                // this is the original list of ppl
                let original_names_order = name_list.names.clone();
                let num_names = original_names_order.len();

                let excluded = BitIndices::from_u64(*excluded_fields);

                name_list.exclude(excluded);
                name_list.assign_boats(*deconflict).await;
                if !freshies {
                    name_list.fill_prog(false).await.unwrap();
                }

                let mut header_buttons = vec![
                    vec![
                        (
                            "exclude all",
                            Self::enum_parent(Self::ExcludeSelection {
                                date: *date,
                                time_slot: *time_slot,
                                freshies: *freshies,
                                deconflict: *deconflict,
                                refresh: *refresh,
                                excluded_fields: u64::MIN,
                                exclude_type: FilterType::Add,
                            }),
                        ),
                        (
                            "include all",
                            Self::enum_parent(Self::ExcludeSelection {
                                date: *date,
                                time_slot: *time_slot,
                                freshies: *freshies,
                                deconflict: *deconflict,
                                refresh: *refresh,
                                excluded_fields: u64::MAX,
                                exclude_type: FilterType::Remove,
                            }),
                        ),
                    ],
                    vec![(
                        "done",
                        Self::enum_parent(Self::Get {
                            date: *date,
                            time_slot: *time_slot,
                            freshies: *freshies,
                            deconflict: *deconflict,
                            refresh: *refresh,
                            excluded_fields: *excluded_fields,
                            show_blanks: false,
                        }),
                    )],
                ];

                // when adding names we are setting one bit (bitor what was calculated)
                // when removing names we are clearing one bit (bitand the inverse of what was calculated)
                // LHS is the original exclude indices
                // RHS is the new computed index
                let (indices_of_names, merge_op): (BitIndices, fn((u64, u64)) -> u64) =
                    match exclude_type {
                        FilterType::Add => (
                            BitIndices::from_u64(!excluded_fields),
                            |(left, right): (u64, u64)| -> u64 { ops::BitOr::bitor(left, right) },
                        ),
                        FilterType::Remove => (
                            BitIndices::from_u64(*excluded_fields),
                            |(left, right): (u64, u64)| -> u64 {
                                ops::BitAnd::bitand(left, !right)
                            },
                        ),
                    };

                let indices_of_names = indices_of_names
                    .to_vec()
                    .into_iter()
                    .filter(|idx| *idx < num_names)
                    .collect::<Vec<_>>();

                // dbg!(&indices_of_names);

                // include/exclude names button construction
                let name_buttons = original_names_order
                    .iter()
                    .enumerate()
                    .filter_map(
                        |(idx, name)| match indices_of_names.binary_search(&idx).is_ok() {
                            true => Some((
                                name,
                                merge_op((*excluded_fields, BitIndices::from_index(idx).to_u64())),
                            )),
                            false => None,
                        },
                    )
                    .map(|(name, excl)| {
                        let callback = Self::enum_parent(Self::ExcludeSelection {
                            date: *date,
                            time_slot: *time_slot,
                            freshies: *freshies,
                            deconflict: *deconflict,
                            refresh: *refresh,
                            excluded_fields: excl,
                            exclude_type: *exclude_type,
                        });

                        (name.as_str(), callback)
                    })
                    .collect::<Vec<_>>();

                let names_2d = convert_to_2d(&name_buttons, 2);

                header_buttons.extend(names_2d);
                let keyboard = construct_keyboard_tuple(header_buttons);
                let text = format!("```\n{}```", name_list);

                bot.edit_message_text(msg.chat.id, msg.id, text)
                    .reply_markup(keyboard)
                    .parse_mode(ParseMode::MarkdownV2)
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
    freshies: bool,
    deconflict: bool,
    refresh: bool,
    exclude_idx: u64,
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

    let mut name_list = match ntu_canoebot_attd::namelist(date_n, time_slot, freshies).await {
        Some(nl) => nl,
        None => {
            log::error!(
                "namelist date: {} time slot: {} not found, defaulting to blank",
                date_n,
                time_slot
            );
            NameList::from_date_time(date, time_slot)
        }
    };

    let excluded = BitIndices::from_u64(exclude_idx);

    name_list.exclude(excluded);
    name_list.assign_boats(deconflict).await;
    // freshies do not follow prog
    if !freshies {
        name_list.fill_prog(false).await.unwrap();
    }

    let d: Date = date.into();
    let prev = Paddling::enum_parent(Paddling::Get {
        date: (date_n - Duration::days(1)).into(),
        time_slot,
        freshies,
        deconflict,
        refresh: false,
        excluded_fields: u64::MAX,
        show_blanks: true,
    });
    let next = Paddling::enum_parent(Paddling::Get {
        date: (date_n + Duration::days(1)).into(),
        time_slot,
        freshies,
        deconflict,
        refresh: false,
        excluded_fields: u64::MAX,
        show_blanks: true,
    });

    // switch between deconf modes
    let refresh = Paddling::enum_parent(Paddling::Get {
        date: d,
        time_slot,
        freshies,
        deconflict,
        refresh: true,
        excluded_fields: exclude_idx,
        show_blanks: true,
    });
    let switch = Paddling::enum_parent(Paddling::Get {
        date: d,
        time_slot,
        freshies,
        deconflict: !deconflict,
        refresh: false,
        excluded_fields: exclude_idx,
        show_blanks: true,
    });
    let time = Paddling::enum_parent(Paddling::Get {
        date: d,
        time_slot: !time_slot,
        freshies,
        deconflict,
        refresh: false,
        excluded_fields: u64::MAX,
        show_blanks: true,
    });
    let month = Paddling::enum_parent(Paddling::MonthSelect { date: d, freshies });

    let switch_label = if deconflict { "plain" } else { "deconf" };
    let time_label = if time_slot { TIME_AM } else { TIME_PM };

    let keyboard = construct_keyboard_tuple([
        vec![
            (BACK_ARROW, prev),
            (REFRESH, refresh),
            (FORWARD_ARROW, next),
        ],
        vec![(switch_label, switch), (time_label, time)],
        vec![
            (DATE, month),
            (
                "filter",
                Paddling::enum_parent(Paddling::ExcludeSelection {
                    date: d,
                    time_slot,
                    freshies,
                    deconflict,
                    refresh: false,
                    excluded_fields: exclude_idx,
                    exclude_type: Default::default(),
                }),
            ),
        ],
    ]);

    let text = format!("```\n{}```", name_list);

    log::info!("checkpoint");

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
