//! Attendance fetch and formatting crate

mod deconflict;
mod update;

use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use lazy_static::lazy_static;

use ntu_canoebot_config as config;
use ntu_canoebot_util::debug_println;
use polars::{
    export::ahash::HashSet,
    prelude::{AnyValue, CsvWriter, DataFrame, SerWriter},
    series::Series,
};
use tokio::sync::RwLock;

pub use update::init;

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

/// Attendance data for one sheet
#[derive(Clone, Debug, Default)]
#[allow(unused)]
pub struct Sheet {
    data: DataFrame,
    start: NaiveDate,
    end: NaiveDate,
}

impl From<DataFrame> for Sheet {
    fn from(mut value: DataFrame) -> Self {
        // verify start date
        let start_date = &value[1].get(0).unwrap();

        let start_date = dataframe_cell_to_string(start_date.to_owned());

        debug_println!("{}", start_date);

        // trim sides of data
        let cols_to_drop: Vec<&str> = value
            .get_column_names()
            .iter()
            .zip((0..*config::SHEEETSCRAPER_LAYOUT_FENCING_LEFT).into_iter())
            .map(|(col, _)| *col)
            .collect();

        value = value.drop_many(&cols_to_drop);

        let length = value.iter().map(|series| series.len()).max().unwrap();
        value = value.slice(*config::SHEEETSCRAPER_LAYOUT_FENCING_TOP, length);

        let name_column = &value[0];

        // remove non-data columns
        let filtered: Vec<Series> = value
            .iter()
            .enumerate()
            .filter_map(|(idx, col)| {
                let window_index =
                    idx % (14 + *config::SHEEETSCRAPER_LAYOUT_BLOCK_PRE_PADDING) as usize;

                if window_index <= *config::SHEEETSCRAPER_LAYOUT_BLOCK_PRE_PADDING as usize {
                    None
                } else {
                    Some(col.to_owned())
                }
            })
            .collect();

        let filtered = DataFrame::new([vec![name_column.to_owned()], filtered].concat()).unwrap();

        println!("{}", filtered);

        let start =
            NaiveDate::parse_from_str(&start_date, *config::SHEETSCRAPER_DATE_FORMAT).unwrap();
        let days_in_sheet = filtered.get_columns().len() / 2 + 1;

        println!("days in sheet: {}", days_in_sheet);
        Self {
            data: filtered,
            start,
            end: start + Duration::days(days_in_sheet as i64), // temp
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

/// Convert an [AnyValue] type to a string.
fn dataframe_cell_to_string(cell: AnyValue) -> String {
    cell.to_string().trim_matches('\"').to_string()
}

pub async fn name_list() {}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_dt_from_str() {
        let dt = "26-Jun-23";

        let date = NaiveDate::parse_from_str(dt, "%d-%b-%y");

        println!("{:?}", date);
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

    #[tokio::test]
    async fn test_sheet_from_dataframe() {
        let today = chrono::Local::now().date_naive();
        let sheet_name = calculate_sheet_name(today);

        println!("sheet name: {}", &sheet_name);

        let df = g_sheets::get_as_dataframe(
            *config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET,
            Some(sheet_name),
        )
        .await;

        let mut sheet: Sheet = df.into();

        println!("sheet start: {}", sheet.start);
        println!("sheet end: {}", sheet.end);

        let out_file = File::create("test.csv").unwrap();
        let csv_writer = CsvWriter::new(out_file);
        csv_writer.has_header(true).finish(&mut sheet.data).unwrap();
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
