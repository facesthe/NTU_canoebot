//! Attendance fetch and formatting crate

mod deconflict;
mod update;

use std::{collections::HashMap, fmt::Display, str::FromStr};

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use lazy_static::lazy_static;

use ntu_canoebot_config as config;
use ntu_canoebot_util::debug_println;
use polars::{
    export::ahash::HashSet,
    prelude::{AnyValue, DataFrame},
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

    /// Local cache of sheet.
    pub static ref SHEET_CACHE: RwLock<AttdSheet> = Default::default();

    /// Local cache of trainig prog.
    /// Since each program sheet contains data for one entire year,
    /// this is pretty much all the data needed.
    pub static ref PROG_CACHE: RwLock<ProgSheet> = Default::default();

}

/// For switching between configs
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum Config {
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
pub struct AttdSheet {
    fetch_time: NaiveDateTime,
    data: DataFrame,
    start: NaiveDate,
    end: NaiveDate,
}

/// Training prog data for sheet
#[derive(Clone, Debug, Default)]
#[allow(unused)]
pub struct ProgSheet {
    fetch_time: NaiveDateTime,
    data: DataFrame,
    start: NaiveDate,
    end: NaiveDate,
}

/// Namelist object
#[derive(Clone, Debug)]
pub struct NameList {
    /// Namelist date
    pub date: NaiveDate,

    /// Time slot
    pub time: bool,

    /// List of names for a session
    pub names: Vec<String>,

    /// List of boats (if any) for a session
    pub boats: Option<Vec<String>>,

    pub fetch_time: NaiveDateTime,
}

/// Format the namelist for display
impl Display for NameList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines: Vec<String> = Vec::new();

        lines.push(self.date.format("%d %b %y").to_string());
        lines.push(format!(
            "{} {}",
            self.date.format("%a"),
            if self.time { "PM" } else { "AM" }
        ));
        lines.push(String::new());

        let main_list: Vec<String> = match &self.boats {
            Some(_boats) => {
                let padding = self.names.iter().map(|name| name.len()).max().unwrap();

                self.names
                    .iter()
                    .zip(_boats.iter())
                    .map(|(n, b)| format!("{:padding$}  {}", n, b, padding = padding))
                    .collect()
            }
            None => self.names.iter().map(|name| name.to_owned()).collect(),
        };

        lines.extend(main_list);

        lines.push(String::new());
        lines.push(format!("fetched at {}", self.fetch_time.format("%H:%M:%S")));

        let res = lines.join("\n");
        write!(f, "{}", res)
    }
}

impl TryFrom<DataFrame> for AttdSheet {
    type Error = ();
    fn try_from(value: DataFrame) -> Result<Self, Self::Error> {
        // verify start date
        let start_date = &value[1].get(0).ok().ok_or(())?;

        let start_date = dataframe_cell_to_string(start_date.to_owned());

        debug_println!("{}", start_date);

        // trim sides of data
        let cols_to_drop: Vec<&str> = value
            .get_column_names()
            .iter()
            .zip((0..*config::SHEEETSCRAPER_LAYOUT_FENCING_LEFT).into_iter())
            .map(|(col, _)| *col)
            .collect();

        debug_println!("to drop cols: {:?}", cols_to_drop);

        let inter_1 = value.drop_many(&cols_to_drop);

        let length = inter_1.iter().map(|series| series.len()).max().ok_or(())?;
        let inter_2 = inter_1.slice(*config::SHEEETSCRAPER_LAYOUT_FENCING_TOP, length);
        let name_column = &inter_2[0];

        // remove non-data columns
        let filtered: Vec<Series> = inter_2
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(idx, col)| {
                let window_index =
                    (idx - 1) % (14 + *config::SHEEETSCRAPER_LAYOUT_BLOCK_PRE_PADDING) as usize;

                if window_index < *config::SHEEETSCRAPER_LAYOUT_BLOCK_PRE_PADDING as usize {
                    None
                } else {
                    Some(col.to_owned())
                }
            })
            .collect();

        let filtered = DataFrame::new([vec![name_column.to_owned()], filtered].concat())
            .ok()
            .ok_or(())?;

        debug_println!("{}", filtered);

        let start = NaiveDate::parse_from_str(&start_date, *config::SHEETSCRAPER_DATE_FORMAT)
            .ok()
            .ok_or(())?;
        let days_in_sheet = filtered.get_columns().len() / 2 + 1;

        debug_println!("days in sheet: {}", days_in_sheet);

        Ok(Self {
            fetch_time: chrono::Local::now().naive_local(),
            data: filtered,
            start,
            end: start + Duration::days(days_in_sheet as i64), // temp
        })
    }
}

impl AttdSheet {
    /// Returns a list of names for a particular date and time.
    /// Returns [Option::None] if the date given is outside the sheet range.
    ///
    /// Set `time_slot` to `false` for morning sessions.
    /// Set `time_slot` to `true` for afternoon sessions.
    pub async fn get_names(&self, date: NaiveDate, time_slot: bool) -> Option<NameList> {
        if date < self.start || date > self.end {
            return None;
        }

        let delta = (date - self.start).num_days() as usize;
        let offset = if time_slot {
            delta * 2 + 2
        } else {
            delta * 2 + 1
        };

        let names = &self.data[0];
        let selected = &self
            .data
            .column(&self.data.get_column_names().get(offset)?)
            .ok()?;

        let read_lock = {
            let change_over = config::SHEETSCRAPER_CHANGEOVER_DATE.date.unwrap();
            let config = get_config_type(date);

            debug_println!(
                "changeover date is: {}.\nUsing {:?} config for date: {}.",
                change_over,
                config,
                date
            );

            SHORTENED_NAMES[config as usize].read().await
        };

        debug_println!("selected col with offset {}: {}", offset, selected);
        let filtered: Vec<String> = selected
            .iter()
            .enumerate()
            .filter_map(|(idx, name)| {
                let converted = dataframe_cell_to_string(name);
                // debug_println!("cell contains: {}", converted);
                if converted == "Y" {
                    let cell = names.get(idx).unwrap();

                    // substitute with short names (if any)
                    let key = dataframe_cell_to_string(cell);
                    if read_lock.contains_key(&key) {
                        return read_lock.get(&key).cloned();
                    } else {
                        return Some(key);
                    }
                } else {
                    None
                }
            })
            .collect();

        Some(NameList {
            date,
            time: time_slot,
            names: filtered,
            boats: None,
            fetch_time: self.fetch_time,
        })
    }

    /// Checks if the sheet contains the specified date
    pub fn contains_date(&self, date: NaiveDate) -> bool {
        if date >= self.start && date <= self.end {
            true
        } else {
            false
        }
    }
}

impl TryFrom<DataFrame> for ProgSheet {
    type Error = ();

    fn try_from(value: DataFrame) -> Result<Self, Self::Error> {
        let now = chrono::Local::now().naive_local();

        let (sheet_start, sheet_end) = {
            let date_col = value
                .column(*config::SHEETSCRAPER_COLUMNS_PROG_DATE)
                .unwrap();

            // debug_println!()

            let start = dataframe_cell_to_string(date_col.iter().next().unwrap());
            let end = dataframe_cell_to_string(date_col.iter().last().unwrap());

            debug_println!("start: {}", start);
            debug_println!("end: {}", end);

            (
                NaiveDate::parse_from_str(&start, *config::SHEETSCRAPER_DATE_FORMAT_PROG).unwrap(),
                NaiveDate::parse_from_str(&end, *config::SHEETSCRAPER_DATE_FORMAT_PROG).unwrap(),
            )
        };

        Ok(Self {
            fetch_time: now,
            data: value,
            start: sheet_start,
            end: sheet_end,
        })
    }
}

impl ProgSheet {
    /// Returns the training prog for a given date
    pub fn get_program(&self, date: NaiveDate, time_slot: bool) -> Option<String> {
        let delta = (date - self.start).num_days();

        let col = if time_slot {
            *config::SHEETSCRAPER_COLUMNS_PROG_PM
        } else {
            *config::SHEETSCRAPER_COLUMNS_PROG_AM
        };

        let col = self.data.column(col).ok()?;
        let cell = col.get(delta as usize).ok()?;

        Some(dataframe_cell_to_string(cell))
    }

    /// Returns the formatted training prog, formatted for display as a message
    pub fn get_formatted_prog(&self, date: NaiveDate, time_slot: bool) -> Option<String> {
        let prog_contents = self.get_program(date, time_slot)?;

        let mut lines = Vec::new();

        lines.push(date.format("%d %b %y").to_string());
        lines.push(format!(
            "{} {}",
            date.format("%a"),
            if time_slot { "PM" } else { "AM" }
        ));
        lines.push(String::new());
        lines.push(prog_contents);
        lines.push(String::new());
        lines.push(format!("fetched at {}", self.fetch_time.format("%H:%M:%S")));

        Some(lines.join("\n"))
    }

    pub fn contains_date(&self, date: NaiveDate) -> bool {
        if date >= self.start && date <= self.end {
            true
        } else {
            false
        }
    }
}

/// Checks which config to use, by comparing the given date and changeover date.
pub fn get_config_type(date: NaiveDate) -> Config {
    let change = config::SHEETSCRAPER_CHANGEOVER_DATE.date.unwrap();

    if date
        >= NaiveDate::from_ymd_opt(change.year.into(), change.month.into(), change.day.into())
            .unwrap()
    {
        Config::New
    } else {
        Config::Old
    }
}

/// Calculate the sheet name from some rules.
///
/// They are:
/// - A sheet always starts on Monday and ends on Sunday
/// - The sheet must end on the last Sunday of a month
/// - A sheet is named MMM-YYYY
pub fn calculate_sheet_name(date: NaiveDate) -> String {
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

/// Refresh the cached sheet
pub async fn refresh_attd_sheet_cache(force: bool) -> Result<(), ()> {
    debug_println!(
        "refreshing attd sheet cache at: {}",
        chrono::Local::now().time()
    );

    let today = chrono::Local::now().date_naive() + Duration::days(1);
    let read_lock = SHEET_CACHE.read().await;

    // check if cache lifetime limit has exceeded
    if (chrono::Local::now().naive_local() - read_lock.fetch_time).num_minutes()
        < *config::SHEETSCRAPER_CACHE_ATTD
    {
        if !force {
            return Ok(());
        }
    }

    drop(read_lock);
    let config = get_config_type(today);

    let sheet_id = ATTENDANCE_SHEETS[config as usize];
    let sheet_name = calculate_sheet_name(today + Duration::days(1));
    let df = g_sheets::get_as_dataframe(sheet_id, Some(sheet_name)).await;

    let sheet: AttdSheet = df.try_into()?;

    let mut write_lock = SHEET_CACHE.write().await;
    write_lock.start = sheet.start;
    write_lock.end = sheet.end;
    write_lock.data = sheet.data;
    write_lock.fetch_time = sheet.fetch_time;

    Ok(())
}

/// Refresh the cached sheet
pub async fn refresh_prog_sheet_cache(force: bool) -> Result<(), ()> {
    debug_println!(
        "refreshing prog sheet cache at: {}",
        chrono::Local::now().time()
    );

    let today = chrono::Local::now().naive_local() + Duration::days(1);
    let read_lock = PROG_CACHE.read().await;

    if (today - read_lock.fetch_time).num_minutes() < *config::SHEETSCRAPER_CACHE_PROG {
        if !force {
            return Ok(());
        }
    }

    drop(read_lock);
    let config = get_config_type(today.date());
    let sheet_id = PROGRAM_SHEETS[config as usize];

    let df = g_sheets::get_as_dataframe(sheet_id, Option::<&str>::None).await;

    let sheet: ProgSheet = df.try_into()?;

    let mut write_lock = PROG_CACHE.write().await;

    write_lock.fetch_time = sheet.fetch_time;
    write_lock.data = sheet.data;
    write_lock.start = sheet.start;
    write_lock.end = sheet.end;

    Ok(())
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::str::FromStr;

    use polars::prelude::{CsvWriter, SerWriter};

    use super::*;

    #[tokio::test]
    async fn get_sheet() {
        let mut df = g_sheets::get_as_dataframe(
            *config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET,
            Some("Aug-2023"),
        )
        .await;

        let out_file = File::create("attd.csv").unwrap();
        let csv_writer = CsvWriter::new(out_file);
        csv_writer.has_header(true).finish(&mut df).unwrap();
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
        init().await;

        let today = chrono::Local::now().date_naive();

        let sheet_name = calculate_sheet_name(today);

        println!("sheet name: {}", &sheet_name);

        let mut df = g_sheets::get_as_dataframe(
            *config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET,
            Some(sheet_name),
        )
        .await;

        let mut sheet: AttdSheet = df.try_into().unwrap();

        let cols = sheet.data.get_column_names();
        println!("{:?}", cols);
        let x: Vec<&&str> = cols.iter().skip(1).collect();
        println!("{:?}", x);

        println!("sheet start: {}", sheet.start);
        println!("sheet end: {}", sheet.end);
        println!("sheet time: {}", sheet.fetch_time);

        let names = sheet.get_names(today, false).await;
        // println!("{:#?}", names);

        println!("namelist:\n{}", names.unwrap());

        let out_file = File::create("raw.csv").unwrap();
        let csv_writer = CsvWriter::new(out_file);
        csv_writer.has_header(true).finish(&mut sheet.data).unwrap();
    }

    #[tokio::test]
    async fn test_prog_from_dataframe() {
        init().await;

        let today = chrono::Local::now().date_naive();

        let mut df = g_sheets::get_as_dataframe(
            *config::SHEETSCRAPER_NEW_PROGRAM_SHEET,
            Option::<&str>::None,
        )
        .await;

        debug_println!("prog sheet: {}", df);

        let sheet: ProgSheet = df.try_into().unwrap();

        let prog = sheet.get_program(today, false);

        println!("{:?}", prog);
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
