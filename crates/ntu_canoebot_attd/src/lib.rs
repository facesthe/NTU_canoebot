//! Attendance fetch and formatting crate
#![allow(unused)]

use std::{collections::HashMap, default, error::Error};

use chrono::{Datelike, Duration, NaiveDate};
use lazy_static::lazy_static;

use ntu_canoebot_config as config;
use polars::{
    error::get_warning_function,
    export::ahash::HashSet,
    prelude::{AnyValue, DataFrame},
};
use tokio::sync::RwLock;

// most of these globals are initialized in init().
lazy_static! {
    /// Attendance sheet lookup
    static ref ATTENDANCE_SHEETS: [&'static str; 2] = [
        &*config::SHEETSCRAPER_OLD_ATTENDANCE_SHEET,
        &*config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET,
    ];

    /// Program sheet lookup
    static ref PROGRAM_SHEETS: [&'static str; 2] = [
        &*config::SHEETSCRAPER_OLD_PROGRAM_SHEET,
        &*config::SHEETSCRAPER_NEW_PROGRAM_SHEET,
    ];

    /// Lookup table of names and their 1-star certificate status.
    /// Those marked as true have passed 1-star.
    static ref NAMES_CERTS: [RwLock<HashMap<String, bool>>; 2] = Default::default();

    /// Set of all valid boats
    static ref BOATS: [RwLock<HashSet<String>>; 2] = Default::default();

    /// Boat allocations hashmap
    /// Name -> Boat
    static ref BOAT_ALLOCATIONS: [RwLock<HashMap<String, (Option<String>, Option<String>)>>; 2] = Default::default();

    /// Hashmap of long names -> short names
    static ref SHORTENED_NAMES: [RwLock<HashMap<String, String>>; 2] = Default::default();
}

/// For switching between configs
#[derive(Clone, Copy)]
#[repr(usize)]
enum Config {
    Old = 0,
    New = 1,
}

impl From<usize> for Config {
    fn from(value: usize) -> Self {
        if value == 0 {
            Config::Old
        } else {
            Config::New
        }
    }
}

/// Calculate the sheet name from some rules.
///
/// They are:
/// - A sheet always starts on Monday and ends on Sunday
/// - The sheet must end on the last Sunday of a month
/// - A sheet is named MMM-YYYY
fn calculate_sheet_name(date: NaiveDate) -> String {
    let last_day = {
        let next_month;
        let next_year;

        if date.month() == 12 {
            next_month = 1;
            next_year = date.year() + 1;
        } else {
            next_month = date.month() + 1;
            next_year = date.year();
        }

        NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap() - Duration::days(1)
    };
    let day = last_day.weekday();
    let days_after = day.number_from_monday() % 7;

    let last_sunday = last_day.day() - days_after;

    let actual_sheet_date = if date.day() > last_sunday {
        last_day + Duration::days(1)
    } else {
        last_day
    };

    actual_sheet_date.format("%b-%Y").to_string()
}

/// Returns the attendance sheet, given a date
async fn get_attendance_sheet(date: NaiveDate) -> Option<DataFrame> {
    let change = {
        let d = config::SHEETSCRAPER_CHANGEOVER_DATE.date.unwrap();
        NaiveDate::from_ymd_opt(d.year as i32, d.month as u32, d.day as u32).unwrap()
    };

    let sheet = if date >= change {
        Config::New
    } else {
        Config::Old
    };

    let sheet_name = calculate_sheet_name(date);

    let df = g_sheets::get_as_dataframe(ATTENDANCE_SHEETS[sheet as usize], Some(sheet_name)).await;

    Some(df)
}

fn dataframe_cell_to_string(cell: AnyValue) -> String {
    cell.to_string().trim_matches('\"').to_string()
}

/// Performs lookup and stuff and updates lazy-static globals
async fn update_config_from_df(
    df: &DataFrame,
    config: Config,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // SHORTENED_NAMES
    let mut names_lookup = df
        .columns([
            &*config::SHEETSCRAPER_COLUMNS_NAME,
            &*config::SHEETSCRAPER_COLUMNS_SHORT_NAME,
        ])
        .unwrap();

    let filtered = names_lookup
        .chunks(2)
        .map(|row_slice| {
            let left = row_slice[0];
            let right = row_slice[1];

            let res: Vec<(String, String)> = left
                .iter()
                .zip(right.iter())
                .filter_map(|(l, r)| {
                    let lft = dataframe_cell_to_string(l);
                    let rht = dataframe_cell_to_string(r);

                    if lft.len() == 0 || rht.len() == 0 {
                        None
                    } else {
                        Some((lft, rht))
                    }
                })
                .collect();

            res
        })
        .collect::<Vec<Vec<(String, String)>>>()
        .concat();

    let map: HashMap<String, String> = filtered.into_iter().collect();
    // println!("names lookup: {:#?}", map);

    let mut lock = SHORTENED_NAMES[config as usize].write().await;
    lock.clear();
    lock.extend(map);

    // BOATS
    let boat_list = df
        .columns([
            &*config::SHEETSCRAPER_COLUMNS_BOAT_PRIMARY,
            &*config::SHEETSCRAPER_COLUMNS_BOAT_ALTERNATE,
        ])
        .unwrap();

    let mut set: HashSet<String> = Default::default();

    for list in &boat_list {
        let filtered = list
            .iter()
            .filter_map(|cell| {
                let name = dataframe_cell_to_string(cell);
                if name.len() != 0 {
                    Some(name)
                } else {
                    None
                }
            })
            .collect::<HashSet<String>>();

        set.extend(filtered);
    }

    // println!("boat set: {:?}", set);

    let mut lock = BOATS[config as usize].write().await;
    lock.clear();
    lock.extend(set);

    // NAMES_CERTS
    let names_and_certs = df
        .columns([
            &*config::SHEETSCRAPER_COLUMNS_NAME,
            &*config::SHEETSCRAPER_COLUMNS_CERTIFICATION,
        ])
        .unwrap();

    let names = names_and_certs.get(0).unwrap();
    let certs = names_and_certs.get(1).unwrap();

    let filtered = names
        .iter()
        .zip(certs.iter())
        .filter_map(|(n, c)| {
            let name = dataframe_cell_to_string(n);
            let cert: Result<u8, std::num::ParseIntError> =
                dataframe_cell_to_string(c).parse::<u8>();
            let status: bool;
            if name.len() == 0 {
                return None;
            }
            match cert {
                Ok(_s) => status = _s != 0, // false if 0, true if otherwise
                Err(_) => return None,
            }

            Some((name, status))
        })
        .collect::<HashMap<String, bool>>();

    // println!("certificate status: {:#?}", filtered);

    let mut lock = NAMES_CERTS[config as usize].write().await;
    lock.clear();
    lock.extend(filtered);

    // BOAT_ALLOCATIONS
    let primary = boat_list[0];
    let alternate = boat_list[1];
    let allocations = names
        .iter()
        .zip(primary.iter())
        .zip(alternate.iter())
        .filter_map(|((name, pri), alt)| {


        let name = dataframe_cell_to_string(name);
        let pri = dataframe_cell_to_string(pri);
        let alt = dataframe_cell_to_string(alt);

        if name.len() == 0 {
            return None;
        }
        let pri_boat = if pri.len() == 0 {
            None
        } else {
            Some(pri)
        };

        let alt_boat = if alt.len() == 0 {
            None
        } else {
            Some(alt)
        };

        Some((name, (pri_boat, alt_boat)))
    }).collect::<HashMap<String, (Option<String>, Option<String>)>>();

    // println!("boat allocations: {:#?}", allocations);
    let mut lock = BOAT_ALLOCATIONS[config as usize].write().await;
    lock.clear();
    lock.extend(allocations);

    Ok(())
}

/// Initialize/reload from the configs sheet
pub async fn init() {
    for (idx, sheet_id) in ATTENDANCE_SHEETS.iter().enumerate() {
        let df = g_sheets::get_as_dataframe(sheet_id, Some(*config::SHEETSCRAPER_CONFIGURATION_SHEET)).await;
        update_config_from_df(&df, idx.into()).await.unwrap()
    }
}

pub async fn name_list() {}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn asd() {
        let sheet = g_sheets::get_as_dataframe(
            ATTENDANCE_SHEETS[Config::Old as usize],
            Some(&*config::SHEETSCRAPER_CONFIGURATION_SHEET),
        )
        .await;
        println!("{:?}", sheet.dtypes());

        let cols = sheet.get_column_names();
        println!("all cols: {:#?}", cols);

        let col = sheet
            .column(&*config::SHEETSCRAPER_COLUMNS_CERTIFICATION)
            .unwrap();
        println!("{}", col);

        update_config_from_df(&sheet, Config::Old).await;
    }

    #[tokio::test]
    async fn test_reloading_configs() {
        init().await;

        let new_allocations = BOAT_ALLOCATIONS[0].read().await;
        println!("{:#?}", new_allocations);

        let new_boats = BOATS[0].read().await;
        println!("{:#?}", new_boats);

        let new_certs = NAMES_CERTS[0].read().await;
        println!("{:#?}", new_certs);

        let new_short_names = SHORTENED_NAMES[0].read().await;
        println!("{:#?}", new_short_names);
    }

    #[test]
    fn test_date_calculation() {
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2022, 12, 28).unwrap()),
            "Jan-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 01, 28).unwrap()),
            "Jan-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 02, 28).unwrap()),
            "Mar-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 03, 28).unwrap()),
            "Apr-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 04, 28).unwrap()),
            "Apr-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 05, 28).unwrap()),
            "May-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 06, 28).unwrap()),
            "Jul-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 07, 28).unwrap()),
            "Jul-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 08, 28).unwrap()),
            "Sep-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 09, 28).unwrap()),
            "Oct-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 10, 28).unwrap()),
            "Oct-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 11, 28).unwrap()),
            "Dec-2023"
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 12, 28).unwrap()),
            "Dec-2023"
        );
    }
}
