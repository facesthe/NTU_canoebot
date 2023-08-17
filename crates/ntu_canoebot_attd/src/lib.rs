//! Attendance fetch and formatting crate

mod deconflict;
pub mod logsheet;
mod update;

use std::{collections::HashMap, fmt::Display};

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

pub use logsheet::SUBMIT_LOCK;
pub use update::init;

const NO_ALLOCATION: &str = "NO BOAT";

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

    /// "Wandering" cache.
    pub static ref SHEET_CACHE_WANDERING: RwLock<AttdSheet> = Default::default();

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
    pub boats: Option<Vec<Option<String>>>,

    pub prog: Option<String>,

    pub fetch_time: NaiveDateTime,
}

/// Format the namelist for display
/// If prog is [Option::Some], format according to config file
impl Display for NameList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines: Vec<String> = Vec::new();
        // names + boat allocationss
        let main_list: Vec<String> = match &self.boats {
            Some(_boats) => {
                let padding = self.names.iter().map(|name| name.len()).max().unwrap_or(0);

                self.names
                    .iter()
                    .zip(_boats.iter())
                    .map(|(n, b)| {
                        format!(
                            "{:padding$}  {}",
                            n,
                            b.clone().unwrap_or(NO_ALLOCATION.to_string()),
                            padding = padding
                        )
                    })
                    .collect()
            }
            None => self.names.iter().map(|name| name.to_owned()).collect(),
        };

        let fetch = format!("fetched at {}", self.fetch_time.format("%H:%M:%S"));

        let res = match &self.prog {
            Some(prog) => {
                let template = *config::SHEETSCRAPER_PADDLING_FORMAT;
                let sub_date = *config::SHEETSCRAPER_PADDLING_SUB_DATE;
                let sub_allo = *config::SHEETSCRAPER_PADDLING_SUB_BOATALLO;
                let sub_prog = *config::SHEETSCRAPER_PADDLING_SUB_PROG;
                let sub_fetch = *config::SHEETSCRAPER_PADDLING_SUB_FETCH;

                let date = self.date.format("%A %d %b ").to_string()
                    + match self.time {
                        false => "AM",
                        true => "PM",
                    };

                let allo = main_list.join("\n");

                let res = template
                    .replace(sub_date, &date)
                    .replace(sub_allo, &allo)
                    .replace(sub_prog, &prog)
                    .replace(sub_fetch, &fetch);

                res
            }
            None => {
                lines.push(self.date.format("%d %b %y").to_string());
                lines.push(format!(
                    "{} {}",
                    self.date.format("%a"),
                    if self.time { "PM" } else { "AM" }
                ));
                lines.push(String::new());
                lines.extend(main_list);

                lines.push(String::new());
                lines.push(fetch);

                let res = lines.join("\n");
                res
            }
        };

        write!(f, "{}", res)
    }
}

impl NameList {
    /// Get namelist to fetch the prog for the day
    pub async fn paddling(&mut self) -> Result<(), ()> {
        let config = get_config_type(self.date);
        let prog_lock = PROG_CACHE.read().await;

        let prog_sheet = if prog_lock.contains_date(self.date) {
            prog_lock.clone()
        } else {
            let sheet_id = PROGRAM_SHEETS[config as usize];
            let df = g_sheets::get_as_dataframe(sheet_id, Option::<&str>::None).await;
            let sheet: ProgSheet = df.try_into()?;
            sheet
        };

        self.prog = Some(
            prog_sheet
                .get_program(self.date, false)
                .unwrap_or("".to_string()),
        );

        Ok(())
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
        let days_in_sheet = calculate_sheet_name(start).1;

        debug_println!("filtered cols: {}", filtered.get_columns().len());
        debug_println!("days in sheet: {}", days_in_sheet);

        Ok(Self {
            fetch_time: chrono::Local::now().naive_local(),
            data: filtered,
            start,
            end: start + Duration::days(days_in_sheet - 1), // temp
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
            prog: None,
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

/// Calculate the start and end of a block.
///
/// - start day is always a monday
/// - end day is always a sunday
pub fn calculate_month_start_end(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let date_block = {
        let month_last = calculate_last_day(date);
        let delta = month_last.weekday().num_days_from_sunday();

        let next_month_cutoff = month_last - Duration::days(delta as i64);
        println!("next month cutoff: {}", next_month_cutoff);

        if date > next_month_cutoff {
            month_last + Duration::days(1)
        } else {
            date
        }
    };

    // debug_println!("date {} belongs in {}", date, date_block);

    let month_end = calculate_last_day(date_block);
    let month_start = NaiveDate::from_ymd_opt(date_block.year(), date_block.month(), 1).unwrap();

    let end_delta = month_end.weekday().num_days_from_sunday();
    let start_delta = month_start.weekday().num_days_from_monday();

    (
        (month_start - Duration::days(start_delta as i64)),
        (month_end - Duration::days(end_delta as i64)),
    )
}

/// Calculate the last day of a month
pub fn calculate_last_day(date: NaiveDate) -> NaiveDate {
    let last_day = {
        let next_y;
        let next_m;

        if date.month() == 12 {
            next_m = 1;
            next_y = date.year() + 1;
        } else {
            next_m = date.month() + 1;
            next_y = date.year();
        }

        NaiveDate::from_ymd_opt(next_y, next_m, 1).unwrap() - Duration::days(1)
    };

    last_day
}

/// Calculate the sheet name from some rules.
/// Also returns the number of days for that sheet.
///
/// They are:
/// - A sheet always starts on Monday and ends on Sunday
/// - The sheet must end on the last Sunday of a month
/// - A sheet is named MMM-YYYY
pub fn calculate_sheet_name(date: NaiveDate) -> (String, i64) {
    let (start, end) = calculate_month_start_end(date);
    let num_days = (end - start).num_days() + 1;
    let sheet_name = end.format("%b-%Y").to_string();

    (sheet_name, num_days)
}

/// Convert an [AnyValue] type to a string.
fn dataframe_cell_to_string(cell: AnyValue) -> String {
    cell.to_string().trim_matches('\"').to_string()
}

/// Return the namelist struct. Accesses cache if hit.
pub async fn namelist(date: NaiveDate, time_slot: bool) -> Option<NameList> {
    let config = get_config_type(date);
    let sheet_id = match config {
        crate::Config::Old => *config::SHEETSCRAPER_OLD_ATTENDANCE_SHEET,
        crate::Config::New => *config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET,
    };

    debug_println!("date: {}\nusing {:?} config", date, config);

    let (sheet_name, _) = calculate_sheet_name(date);

    debug_println!("sheet name: {}", sheet_name);

    let sheet: AttdSheet = {
        // check if cache matches up with this sheet
        let read_lock = SHEET_CACHE.read().await;
        let in_cache = read_lock.contains_date(date.into());

        let read_lock_wandering = SHEET_CACHE_WANDERING.read().await;
        let in_wandering_cache = read_lock_wandering.contains_date(date.into());

        match (in_cache, in_wandering_cache) {
            (true, _) => read_lock.clone(),
            (false, true) => read_lock_wandering.clone(),
            (false, false) => {
                drop(read_lock_wandering);
                let df = g_sheets::get_as_dataframe(sheet_id, Some(sheet_name)).await;
                let s: AttdSheet = df.try_into().ok()?;

                let mut write_wandering = SHEET_CACHE_WANDERING.write().await;

                write_wandering.start = s.start;
                write_wandering.end = s.end;
                write_wandering.data = s.data;
                write_wandering.fetch_time = s.fetch_time;

                write_wandering.clone()
            }
        }
    };

    debug_println!("sheet from: {} to {}", sheet.start, sheet.end);

    sheet.get_names(date, time_slot).await
}

/// Refresh the cached sheet
pub async fn refresh_attd_sheet_cache(force: bool) -> Result<(), ()> {
    debug_println!(
        "refreshing attd sheet cache at: {}",
        chrono::Local::now().time()
    );

    let today = chrono::Local::now().date_naive() + Duration::days(1);
    let read_cache = SHEET_CACHE.read().await;

    // check if cache lifetime limit has exceeded
    if (chrono::Local::now().naive_local() - read_cache.fetch_time).num_minutes()
        < *config::SHEETSCRAPER_CACHE_ATTD
    {
        if !force {
            return Ok(());
        }
    }

    drop(read_cache);
    let config = get_config_type(today);
    let sheet_id = ATTENDANCE_SHEETS[config as usize];
    let (sheet_name, _) = calculate_sheet_name(today + Duration::days(1));

    let read_wandering = SHEET_CACHE_WANDERING.read().await;
    let wandering_date = read_wandering.start;

    drop(read_wandering);
    let config = get_config_type(wandering_date);
    let sheet_id_wandering = ATTENDANCE_SHEETS[config as usize];
    let (sheet_name_wandering, _) = calculate_sheet_name(wandering_date);

    let tasks = (
        tokio::spawn(g_sheets::get_as_dataframe(sheet_id, Some(sheet_name))),
        tokio::spawn(g_sheets::get_as_dataframe(
            sheet_id_wandering,
            Some(sheet_name_wandering),
        )),
    );

    let df = tasks.0.await.unwrap();
    let sheet: AttdSheet = df.try_into()?;

    let df_wandering = tasks.1.await.unwrap();
    let sheet_wandering: AttdSheet = df_wandering.try_into()?;

    debug_println!("local cache fetch at: {}", sheet.fetch_time);
    debug_println!("wandering cache fetch at: {}", sheet_wandering.fetch_time);

    let mut write_lock = SHEET_CACHE.write().await;
    write_lock.start = sheet.start;
    write_lock.end = sheet.end;
    write_lock.data = sheet.data;
    write_lock.fetch_time = sheet.fetch_time;

    let mut write_wandering = SHEET_CACHE_WANDERING.write().await;
    write_wandering.start = sheet_wandering.start;
    write_wandering.end = sheet_wandering.end;
    write_wandering.data = sheet_wandering.data;
    write_wandering.fetch_time = sheet_wandering.fetch_time;

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

    if (chrono::Local::now().naive_local() - read_lock.fetch_time).num_minutes()
        < *config::SHEETSCRAPER_CACHE_PROG
    {
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

        let (sheet_name, _) = calculate_sheet_name(today);

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

    fn create_date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_calculate_month_start_end() {
        let date = create_date(2023, 1, 28);
        let (start, end) = calculate_month_start_end(date);
        assert_eq!(start, create_date(2022, 12, 26));
        assert_eq!(end, create_date(2023, 1, 29));

        let date = create_date(2023, 2, 28);
        let (start, end) = calculate_month_start_end(date);
        assert_eq!(start, create_date(2023, 2, 27));
        assert_eq!(end, create_date(2023, 3, 26));

        let date = create_date(2023, 3, 28);
        let (start, end) = calculate_month_start_end(date);
        assert_eq!(start, create_date(2023, 3, 27));
        assert_eq!(end, create_date(2023, 4, 30));

        let date = create_date(2023, 4, 28);
        let (start, end) = calculate_month_start_end(date);
        assert_eq!(start, create_date(2023, 3, 27));
        assert_eq!(end, create_date(2023, 4, 30));

        let date = create_date(2023, 5, 28);
        let (start, end) = calculate_month_start_end(date);
        assert_eq!(start, create_date(2023, 5, 1));
        assert_eq!(end, create_date(2023, 5, 28));

        let date = create_date(2023, 6, 28);
        let (start, end) = calculate_month_start_end(date);
        assert_eq!(start, create_date(2023, 6, 26));
        assert_eq!(end, create_date(2023, 7, 30));
    }

    #[test]
    fn test_date_calculation() {
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2022, 12, 28).unwrap()),
            ("Jan-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 01, 28).unwrap()),
            ("Jan-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 01, 30).unwrap()),
            ("Feb-2023".to_string(), 28)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 02, 28).unwrap()),
            ("Mar-2023".to_string(), 28)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 03, 28).unwrap()),
            ("Apr-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 04, 28).unwrap()),
            ("Apr-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 05, 28).unwrap()),
            ("May-2023".to_string(), 28)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 05, 29).unwrap()),
            ("Jun-2023".to_string(), 28)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 06, 28).unwrap()),
            ("Jul-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 07, 28).unwrap()),
            ("Jul-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 07, 31).unwrap()),
            ("Aug-2023".to_string(), 28)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 08, 28).unwrap()),
            ("Sep-2023".to_string(), 28)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 09, 28).unwrap()),
            ("Oct-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 10, 28).unwrap()),
            ("Oct-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 10, 30).unwrap()),
            ("Nov-2023".to_string(), 28)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 11, 28).unwrap()),
            ("Dec-2023".to_string(), 35)
        );
        assert_eq!(
            calculate_sheet_name(NaiveDate::from_ymd_opt(2023, 12, 28).unwrap()),
            ("Dec-2023".to_string(), 35)
        );
    }

    /// Check that num days per sheet is always a multiple of 7
    #[test]
    fn test_num_days_calculation() {
        let start = chrono::Local::now().date_naive();

        for d in 0..365 {
            let day = start + Duration::days(d);
            let num_days = calculate_sheet_name(day).1;
            assert_eq!(num_days % 7, 0);
            assert_ne!(num_days, 0);
        }
    }
}
