//! Attendance fetch and formatting crate

mod deconflict;
pub mod logsheet;
mod update;

use std::{collections::HashMap, fmt::Display, sync::Arc};

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use lazy_static::lazy_static;

use ntu_canoebot_config as config;
use ntu_canoebot_util::debug_println;
use polars::{
    export::ahash::HashSet,
    prelude::{AnyValue, DataFrame},
    series::Series,
};
use tokio::sync::{RwLock, RwLockWriteGuard};

pub use logsheet::SUBMIT_LOCK;
pub use update::init;

const NO_ALLOCATION: &str = "NO BOAT";

// most of these globals are initialized in init().
lazy_static! {
    /// Attendance sheet lookup
    static ref ATTENDANCE_SHEETS: [Option<&'static str>; 2] = [
        match config::SHEETSCRAPER_OLD_ATTENDANCE_SHEET.len() {
            0 => None,
            _ => Some(config::SHEETSCRAPER_OLD_ATTENDANCE_SHEET)
        },
        match config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET.len() {
            0 => None,
            _ => Some(config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET)
        },
    ];

    /// Program sheet lookup
    static ref PROGRAM_SHEETS: [Option<&'static str>; 2] = [
        match config::SHEETSCRAPER_OLD_PROGRAM_SHEET.len() {
            0 => None,
            _ => Some(config::SHEETSCRAPER_OLD_PROGRAM_SHEET)
        },
        match config::SHEETSCRAPER_NEW_PROGRAM_SHEET.len() {
            0 => None,
            _ => Some(config::SHEETSCRAPER_NEW_PROGRAM_SHEET)
        },
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

    /// Set of names that are part of the EXCO
    pub static ref EXCO_NAMES: [RwLock<HashSet<String>>; 2] = Default::default();

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

/// Training session type
#[derive(Clone, Copy, Debug, Default)]
pub enum Session {
    #[default]
    Paddling,
    Land,
}

/// This struct stores the indices of names to retain/exclude.
/// The LSB corresponds to vec idx 0, with support of up to 64 names.
///
/// By default, if all boats are allocated, this would be the same as setting
/// the inner u64 to [u64::MAX].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitIndices(u64);

/// Namelist object
#[derive(Clone, Debug)]
pub struct NameList {
    /// Namelist date
    pub date: NaiveDate,

    /// Session type
    pub session: Session,

    /// Time slot
    pub time: bool,

    /// List of names for a session
    pub names: Vec<String>,

    /// Names that are excluded from boat allocations
    pub excluded_names: Vec<String>,

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

        // these are tabbed out
        let excluded_list: Vec<String> = self
            .excluded_names
            .iter()
            .map(|n| format!("\t{}", n))
            .collect();

        let fetch = format!("fetched at {}", self.fetch_time.format("%H:%M:%S"));

        let res = match &self.prog {
            Some(prog) => {
                let template = config::SHEETSCRAPER_PADDLING_FORMAT;
                let sub_session = config::SHEETSCRAPER_PADDLING_SUB_SESSION;
                let sub_date = config::SHEETSCRAPER_PADDLING_SUB_DATE;
                let sub_allo = config::SHEETSCRAPER_PADDLING_SUB_BOATALLO;
                let sub_exclude = config::SHEETSCRAPER_PADDLING_SUB_EXCLUDE;
                let sub_prog = config::SHEETSCRAPER_PADDLING_SUB_PROG;
                let sub_fetch = config::SHEETSCRAPER_PADDLING_SUB_FETCH;

                let date = self.date.format("%A %d %b ").to_string()
                    + match self.time {
                        false => "AM",
                        true => "PM",
                    };

                let allo = main_list.join("\n");
                let excl = excluded_list.join("\n");

                let res = template
                    .replace(sub_session, format!("{:?}", self.session).as_str())
                    .replace(sub_date, &date)
                    .replace(sub_allo, &allo)
                    .replace(sub_exclude, &excl)
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

/// The attendance breakdown for a particular week,
/// Monday to Sunday
///
/// Shows:
/// - dates
/// - total paddlers
/// - exco paddlers
/// - fetch time
#[derive(Clone, Debug, Default)]
pub struct Breakdown {
    start: NaiveDate,
    num_total: [u16; 7],
    num_exco: [u16; 7],
    fetch_time: NaiveDateTime,
}

impl Display for Breakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DATE: &str = "date";
        const TOTAL: &str = "total";
        const EXCO: &str = "exco";

        let dates: Vec<String> = (0..7)
            .into_iter()
            .map(|add| {
                (self.start + Duration::days(add))
                    .format("%a %d %b")
                    .to_string()
            })
            .collect();

        let (max_a, max_b, max_c) = {
            let col_a = dates.iter().map(|d| d.len()).max().unwrap();
            let col_a = col_a.max(DATE.len());

            let col_b = self
                .num_total
                .iter()
                .map(|num| num_digits(*num as i64))
                .max()
                .unwrap();
            let col_b = col_b.max(TOTAL.len());

            let col_c = self
                .num_exco
                .iter()
                .map(|num| num_digits(*num as i64))
                .max()
                .unwrap();
            let col_c = col_c.max(EXCO.len());

            (col_a, col_b, col_c)
        };

        let mut resp_vec: Vec<String> = Vec::new();
        resp_vec.push(format!(
            "{:<width_a$} {:<width_b$} {:<width_c$}",
            DATE,
            TOTAL,
            EXCO,
            width_a = max_a,
            width_b = max_b,
            width_c = max_c
        ));

        for (date, (total, exco)) in dates
            .iter()
            .zip(self.num_total.iter().zip(self.num_exco.iter()))
        {
            resp_vec.push(format!(
                "{:<width_a$} {:>width_b$} {:>width_c$}",
                date,
                total,
                exco,
                width_a = max_a,
                width_b = max_b,
                width_c = max_c
            ));
        }

        resp_vec.push(String::new());
        resp_vec.push(format!("fetched at {}", self.fetch_time.format("%H:%M:%S")));

        let res = resp_vec.join("\n");

        write!(f, "{}", res)
    }
}

fn num_digits(mut number: i64) -> usize {
    if number == 0 {
        return 1;
    }

    let mut res = 0;
    while number != 0 {
        number /= 10;
        res += 1;
    }

    res
}

impl Default for BitIndices {
    fn default() -> Self {
        Self(u64::MAX)
    }
}

impl BitIndices {
    pub fn from_u64(value: u64) -> Self {
        Self(value)
    }

    pub fn to_u64(&self) -> u64 {
        self.0
    }

    /// Create an instance of `Self` from a single index.
    ///
    ///
    /// ```
    /// use ntu_canoebot_attd::BitIndices;
    ///
    /// let indices = BitIndices::from_index(10);
    ///
    /// assert_eq!(indices.to_u64(), 0b100_0000_0000);
    /// ```
    pub fn from_index(mut index: usize) -> Self {
        index &= 0b11_1111; // bounds check
        Self(0b1 << index)
    }

    /// Create an instance of `Self` from a slice.
    ///
    /// This will only look at indices up to the bitsize (63)
    ///
    /// ```
    /// use ntu_canoebot_attd::BitIndices;
    ///
    /// let v = vec![1,2,4,6,8,11];
    ///
    /// let indices = BitIndices::from_vec(v);
    ///
    /// assert_eq!(indices.to_u64(), 0b1001_0101_0110);
    /// ```
    pub fn from_vec(mut v: Vec<usize>) -> Self {
        v.sort();

        let bit_iter = v.iter().filter(|idx| **idx < 64);

        let mut inner = 0_u64;
        for bit_idx in bit_iter {
            inner |= 0b1 << bit_idx;
        }

        Self(inner)
    }

    /// Converts the bitwise internal representation to a vector of indices.
    ///
    /// ```
    /// use ntu_canoebot_attd::BitIndices;
    ///
    /// let indices = BitIndices::from_u64(0b1111_0000);
    /// let converted = indices.to_vec();
    ///
    /// assert_eq!(converted, vec![4,5,6,7])
    /// ```
    pub fn to_vec(&self) -> Vec<usize> {
        let mut copied = self.0;
        let mut indices = Vec::new();
        let mut idx = 0;

        // short circuit if spawned from default
        if self.0 == u64::MAX {
            return (0..64).into_iter().collect();
        }

        loop {
            if copied == 0 {
                break;
            }

            let zeros = copied.trailing_zeros();
            idx += zeros;
            copied >>= zeros;

            match (copied & 0b1) != 0 {
                true => indices.push(idx as usize),
                false => (),
            }

            copied >>= 1;
            idx += 1;
        }

        indices
    }
}

impl NameList {
    pub fn from_date_time(date: NaiveDate, time_slot: bool) -> Self {
        Self {
            date,
            session: Default::default(),
            time: time_slot,
            names: Default::default(),
            excluded_names: Default::default(),
            boats: Default::default(),
            prog: Default::default(),
            fetch_time: chrono::Local::now().naive_local(),
        }
    }

    /// Get namelist to fetch the prog for the day, for a given time slot
    pub async fn fill_prog(&mut self, time_slot: bool) -> Result<(), ()> {
        let prog_sheet = training_prog(self.date).await;

        self.prog = Some(
            prog_sheet
                .get_program(self.date, time_slot)
                .unwrap_or("".to_string()),
        );

        Ok(())
    }

    /// Retain names that have an index. Move all the rest to
    /// the exclude list.
    pub fn exclude(&mut self, exclude_idx: BitIndices) {
        // filter out indices within range
        let indices: Vec<usize> = exclude_idx
            .to_vec()
            .into_iter()
            .filter(|idx| *idx < self.names.len())
            .collect();

        let filtered: Vec<_> = self
            .names
            .iter()
            .enumerate()
            .filter_map(|(idx, name)| match indices.binary_search(&idx).is_ok() {
                true => Some(name.to_owned()),
                false => None,
            })
            .collect();

        let excluded: Vec<_> = self
            .names
            .iter()
            .enumerate()
            .filter_map(|(idx, name)| match !indices.binary_search(&idx).is_ok() {
                true => Some(name.to_owned()),
                false => None,
            })
            .collect();

        self.names = filtered;
        self.excluded_names = excluded;
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
            .zip((0..config::SHEETSCRAPER_LAYOUT_ATTD_FENCING_LEFT).into_iter())
            .map(|(col, _)| *col)
            .collect();

        debug_println!("to drop cols: {:?}", cols_to_drop);

        let inter_1 = value.drop_many(&cols_to_drop);

        let length = inter_1.iter().map(|series| series.len()).max().ok_or(())?;
        let inter_2 = inter_1.slice(config::SHEETSCRAPER_LAYOUT_ATTD_FENCING_TOP, length);
        let name_column = &inter_2[0];

        // remove non-data columns
        let filtered: Vec<Series> = inter_2
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(idx, col)| {
                let window_index =
                    (idx - 1) % (14 + config::SHEETSCRAPER_LAYOUT_ATTD_BLOCK_PRE_PADDING) as usize;

                if window_index < config::SHEETSCRAPER_LAYOUT_ATTD_BLOCK_PRE_PADDING as usize {
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

        let start = NaiveDate::parse_from_str(&start_date, config::SHEETSCRAPER_DATE_FORMAT)
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
    pub fn from_date(date: NaiveDate) -> Self {
        let (start, end) = calculate_month_start_end(date);

        Self {
            fetch_time: chrono::Local::now().naive_local(),
            data: Default::default(),
            start,
            end,
        }
    }

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

        let names = &self
            .data
            .column(&self.data.get_column_names().get(0)?)
            .ok()?;
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

        debug_println!(
            "selected col with offset {} (col {}): {}",
            offset,
            // reconstruct the original column idx, inside the shared online sheet.
            AttdSheet::col_idx_to_excel_alphabetic(
                offset
                    + {
                        ((offset - 1) / 14 + 1)
                            * config::SHEETSCRAPER_LAYOUT_ATTD_BLOCK_PRE_PADDING as usize
                    }
                    + config::SHEETSCRAPER_LAYOUT_ATTD_FENCING_LEFT as usize
                    + 1
            ),
            selected
        );
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
            session: Session::Paddling,
            time: time_slot,
            names: filtered,
            excluded_names: Default::default(),
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

    /// Convert a column number to the human-readable excel column indices.
    ///
    /// 1-indexed instead of 0-indexed, so 1 maps to A, 26 maps to Z, etc.
    #[cfg(debug_assertions)]
    fn col_idx_to_excel_alphabetic(mut idx: usize) -> String {
        let mut col_idx: Vec<char> = Vec::new();

        loop {
            let pos = idx % 26;
            let lowest = (pos as u8 + 96) as char; // offset to ascii 'a'

            col_idx.push(lowest);

            idx /= 26;
            if idx == 0 {
                break;
            }
        }

        col_idx
            .iter()
            .rev()
            .map(|c| c.to_ascii_uppercase())
            .collect()
    }
}

impl TryFrom<DataFrame> for ProgSheet {
    type Error = ();

    fn try_from(value: DataFrame) -> Result<Self, Self::Error> {
        let now = chrono::Local::now().naive_local();

        let (sheet_start, sheet_end) = {
            let date_col = value
                .column(config::SHEETSCRAPER_COLUMNS_PROG_DATE)
                .map_err(|_| ())?;

            // debug_println!()

            let start = dataframe_cell_to_string(date_col.iter().next().unwrap());
            let end = dataframe_cell_to_string(date_col.iter().last().unwrap());

            debug_println!("start: {}", start);
            debug_println!("end: {}", end);

            (
                NaiveDate::parse_from_str(&start, config::SHEETSCRAPER_DATE_FORMAT_PROG)
                    .map_err(|_| ())?,
                NaiveDate::parse_from_str(&end, config::SHEETSCRAPER_DATE_FORMAT_PROG)
                    .map_err(|_| ())?,
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
    pub fn from_date(date: NaiveDate) -> Self {
        let (start, end) = calculate_month_start_end(date);

        Self {
            fetch_time: chrono::Local::now().naive_local(),
            data: Default::default(),
            start,
            end,
        }
    }

    /// Returns the training prog for a given date
    pub fn get_program(&self, date: NaiveDate, time_slot: bool) -> Option<String> {
        let delta = (date - self.start).num_days();

        let col = if time_slot {
            config::SHEETSCRAPER_COLUMNS_PROG_PM
        } else {
            config::SHEETSCRAPER_COLUMNS_PROG_AM
        };

        let col = self.data.column(col).ok()?;
        let cell = col.get(delta as usize).ok()?;

        Some(dataframe_cell_to_string(cell))
    }

    /// Returns the formatted training prog, formatted for display as a message
    pub fn get_formatted_prog(&self, date: NaiveDate, time_slot: bool) -> Option<String> {
        let prog_contents = self.get_program(date, time_slot).unwrap_or("".to_string());

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
        debug_println!("next month cutoff: {}", next_month_cutoff);

        if date > next_month_cutoff {
            month_last + Duration::days(1)
        } else {
            date
        }
    };

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

/// Calculates the sheet name for lang prog.
///
/// This is a very simple function as makes some assumptions about
/// the semester structure that NTU has.
pub fn calculate_land_sheet_name(date: NaiveDate) -> String {
    const FIRST_SEM_MONTH: u32 = 8; // AY starts in August

    let acad_year: String = {
        let year = if date.month() >= FIRST_SEM_MONTH {
            date.year()
        } else {
            date.year() - 1
        };

        format!("{}", year % 100)
    };

    let sem: String = {
        let month = date.month();

        if month >= FIRST_SEM_MONTH {
            "S1".to_string()
        } else {
            "S2".to_string()
        }
    };

    format!("gym-{}{}", acad_year, sem)
}

/// Convert an [AnyValue] type to a string.
fn dataframe_cell_to_string(cell: AnyValue) -> String {
    cell.to_string().trim_matches('\"').to_string()
}

/// Return the namelist struct. Accesses cache if hit.
pub async fn namelist(date: NaiveDate, time_slot: bool) -> Option<NameList> {
    let config = get_config_type(date);
    let sheet_id = ATTENDANCE_SHEETS[config as usize];

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

                let sheet = {
                    match sheet_id {
                        Some(id) => {
                            let df = g_sheets::get_as_dataframe(id, Some(sheet_name)).await;
                            let s: AttdSheet = df.try_into().unwrap_or(AttdSheet::from_date(date));
                            s
                        }
                        None => AttdSheet::from_date(date),
                    }
                };

                let mut write_wandering = SHEET_CACHE_WANDERING.write().await;
                update_attd_cache(sheet, &mut write_wandering);

                write_wandering.clone()
            }
        }
    };

    debug_println!("sheet from: {} to {}", sheet.start, sheet.end);

    sheet.get_names(date, time_slot).await
}

/// Finds the training program for a given date. Accesses the cache
/// if hit.
pub async fn training_prog(date: NaiveDate) -> ProgSheet {
    let config = get_config_type(date);
    let sheet_id = PROGRAM_SHEETS[config as usize];

    let read_lock = PROG_CACHE.read().await;
    let prog_sheet = if read_lock.contains_date(date) {
        read_lock.clone()
    } else {
        match sheet_id {
            Some(id) => {
                let df = g_sheets::get_as_dataframe(id, Option::<&str>::None).await;
                let sheet: ProgSheet = df.try_into().unwrap_or(ProgSheet::from_date(date));
                sheet
            }
            None => ProgSheet::from_date(date),
        }
    };

    prog_sheet
}

/// Returns the attendance breakdown for a particular week,
/// from Mon to Sun
pub async fn breakdown(date: NaiveDate, time_slot: bool) -> Breakdown {
    let cache_lock = SHEET_CACHE.read().await;
    let mut wand_lock = SHEET_CACHE_WANDERING.write().await;
    let sheet = {
        match (
            cache_lock.contains_date(date),
            wand_lock.contains_date(date),
        ) {
            (true, _) => cache_lock.clone(),
            (false, true) => wand_lock.clone(),
            (false, false) => {
                let config = get_config_type(date);
                let sheet_name = calculate_sheet_name(date).0;

                match ATTENDANCE_SHEETS[config as usize] {
                    Some(sheet) => {
                        let df = g_sheets::get_as_dataframe(sheet, Some(sheet_name)).await;
                        let sheet: AttdSheet = df.try_into().unwrap_or(AttdSheet::from_date(date));
                        update_attd_cache(sheet, &mut wand_lock);
                        wand_lock.clone()
                    }
                    None => AttdSheet::from_date(date),
                }
            }
        }
    };

    let first_day = date - Duration::days(date.weekday().num_days_from_monday() as i64);

    let sheet_ref = Arc::new(sheet);

    let jobs_vec = (0..7)
        .into_iter()
        .map(|d| {
            let sheet_clone = sheet_ref.clone();
            tokio::spawn(async move {
                let day = first_day + Duration::days(d);
                (day, sheet_clone.get_names(day, time_slot).await)
            })
        })
        .collect::<Vec<_>>();

    let mut breakdown = Breakdown::default();
    breakdown.fetch_time = sheet_ref.fetch_time;
    breakdown.start = first_day;

    let config = get_config_type(date);
    let exco_lock = EXCO_NAMES[config as usize].read().await;

    for (idx, job) in jobs_vec.into_iter().enumerate() {
        let (day, names) = job.await.unwrap();
        let nlist = match names {
            Some(list) => list,
            None => NameList::from_date_time(day, false),
        };

        let num_total = nlist.names.len();
        breakdown.num_total[idx] = num_total as u16;

        let num_exco: usize = nlist
            .names
            .iter()
            .map(|name| if exco_lock.contains(name) { 1 } else { 0 })
            .sum();

        breakdown.num_exco[idx] = num_exco as u16;
    }

    breakdown
}

/// Returns the land program, taking names from the gym sheet.
///
/// No cache for this one, it's barely used.
///
/// All data processing is performed inside here.
pub async fn land(date: NaiveDate) -> NameList {
    let config = get_config_type(date);
    let sheet_name = calculate_land_sheet_name(date);

    let df = match ATTENDANCE_SHEETS[config as usize] {
        Some(sheet_id) => g_sheets::get_as_dataframe(sheet_id, Some(sheet_name)).await,
        None => return NameList::from_date_time(date, true),
    };

    // trim sides of data
    let cols_to_drop: Vec<&str> = df
        .get_column_names()
        .iter()
        .zip((0..config::SHEETSCRAPER_LAYOUT_LAND_FENCING_LEFT).into_iter())
        .map(|(col, _)| *col)
        .collect();

    debug_println!("to drop cols: {:?}", cols_to_drop);

    let df_fenced = df.drop_many(&cols_to_drop);

    // let length = df_fenced.iter().map(|series| series.len()).max().unwrap_or(0);

    // debug_println!("{}", inter_1);
    // let df_fenced = inter_1.slice(config::SHEETSCRAPER_LAYOUT_LAND_FENCING_TOP, length);
    let name_column = &df_fenced[0];

    // debug_println!("{}", df_fenced);

    let day = date.weekday().num_days_from_monday();
    let offset = day * 2 + 1;

    let attd_column = df_fenced
        .column(df_fenced.get_column_names().get(offset as usize).unwrap())
        .unwrap();

    debug_println!("{}", attd_column);

    let read_lock = SHORTENED_NAMES[config as usize].read().await;

    let filtered: Vec<String> = attd_column
        .iter()
        .enumerate()
        .filter_map(|(idx, cell)| {
            let contents = dataframe_cell_to_string(cell);
            if contents == "Y" {
                let name = name_column.get(idx).unwrap();
                let key = dataframe_cell_to_string(name);
                debug_println!("name: {}", key);
                if read_lock.contains_key(&key) {
                    read_lock.get(&key).cloned()
                } else {
                    Some(key)
                }
            } else {
                None
            }
        })
        .collect();

    NameList {
        date,
        session: Session::Land,
        time: true,
        names: filtered,
        excluded_names: Default::default(),
        boats: None,
        prog: None,
        fetch_time: chrono::Local::now().naive_local(),
    }

    // println!("{}", df);
}

fn update_attd_cache(sheet: AttdSheet, cache_lock: &mut RwLockWriteGuard<'_, AttdSheet>) {
    cache_lock.start = sheet.start;
    cache_lock.end = sheet.end;
    cache_lock.data = sheet.data;
    cache_lock.fetch_time = sheet.fetch_time;
}

/// Refresh the main cached and wandering sheet
pub async fn refresh_attd_sheet_cache(force: bool) -> Result<(), ()> {
    debug_println!(
        "refreshing attd sheet cache at: {}",
        chrono::Local::now().time()
    );

    let today = chrono::Local::now().date_naive() + Duration::days(1);
    let read_cache = SHEET_CACHE.read().await;

    // check if cache lifetime limit has exceeded
    if (chrono::Local::now().naive_local() - read_cache.fetch_time).num_minutes()
        < config::SHEETSCRAPER_CACHE_ATTD
    {
        if !force {
            return Ok(());
        }
    }

    drop(read_cache);
    let config = get_config_type(today);
    let sheet_id = ATTENDANCE_SHEETS[config as usize];
    let (sheet_name, _) = calculate_sheet_name(today);

    let read_wandering = SHEET_CACHE_WANDERING.read().await;
    let wandering_date = read_wandering.start;

    drop(read_wandering);
    let config = get_config_type(wandering_date);
    let sheet_id_wandering = ATTENDANCE_SHEETS[config as usize];
    let (sheet_name_wandering, _) = calculate_sheet_name(wandering_date);

    let mut cache_lock = SHEET_CACHE.write().await;
    let mut cache_lock_wand = SHEET_CACHE_WANDERING.write().await;

    match (sheet_id, sheet_id_wandering) {
        (None, None) => (),
        (None, Some(wand)) => {
            let df = g_sheets::get_as_dataframe(wand, Some(sheet_name_wandering)).await;
            let sheet_wand: AttdSheet = df
                .try_into()
                .unwrap_or(AttdSheet::from_date(wandering_date));
            update_attd_cache(sheet_wand, &mut cache_lock_wand);
        }
        (Some(id), None) => {
            let df = g_sheets::get_as_dataframe(id, Some(sheet_name)).await;
            let sheet = df.try_into().unwrap_or(AttdSheet::from_date(today));
            update_attd_cache(sheet, &mut cache_lock);
        }
        (Some(id), Some(id_wand)) => {
            let tasks = (
                tokio::spawn(g_sheets::get_as_dataframe(id, Some(sheet_name))),
                tokio::spawn(g_sheets::get_as_dataframe(
                    id_wand,
                    Some(sheet_name_wandering),
                )),
            );

            let df = tasks.0.await.unwrap();
            let sheet: AttdSheet = df.try_into().unwrap_or(AttdSheet::from_date(today));

            let df_wandering = tasks.1.await.unwrap();
            let sheet_wandering: AttdSheet = df_wandering
                .try_into()
                .unwrap_or(AttdSheet::from_date(wandering_date));

            update_attd_cache(sheet, &mut cache_lock);
            update_attd_cache(sheet_wandering, &mut cache_lock_wand);
        }
    }

    Ok(())
}

/// Refresh the cached sheet
pub async fn refresh_prog_sheet_cache(force: bool) -> Result<(), ()> {
    debug_println!(
        "refreshing prog sheet cache at: {}",
        chrono::Local::now().time()
    );

    let today = chrono::Local::now().date_naive() + Duration::days(1);
    let read_lock = PROG_CACHE.read().await;

    if (chrono::Local::now().naive_local() - read_lock.fetch_time).num_minutes()
        < config::SHEETSCRAPER_CACHE_PROG
    {
        if !force {
            return Ok(());
        }
    }

    drop(read_lock);
    let config = get_config_type(today);
    let sheet_id = PROGRAM_SHEETS[config as usize];

    let sheet = {
        match sheet_id {
            Some(id) => {
                let df = g_sheets::get_as_dataframe(id, Option::<&str>::None).await;
                let sheet: ProgSheet = df.try_into().unwrap_or(ProgSheet::from_date(today));

                sheet
            }
            None => ProgSheet::from_date(today),
        }
    };

    let mut write_lock = PROG_CACHE.write().await;

    write_lock.fetch_time = sheet.fetch_time;
    write_lock.data = sheet.data;
    write_lock.start = sheet.start;
    write_lock.end = sheet.end;

    drop(write_lock);

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

    #[test]
    fn test_num_digits() {
        assert_eq!(num_digits(1), 1);
        assert_eq!(num_digits(9), 1);
        assert_eq!(num_digits(10), 2);
        assert_eq!(num_digits(99), 2);
        assert_eq!(num_digits(100), 3);
    }

    #[tokio::test]
    async fn test_breakdown() {
        init().await;

        let bd = breakdown(chrono::Local::now().date_naive(), false).await;
        println!("{}", bd);

        let bd = breakdown(chrono::Local::now().date_naive(), true).await;
        println!("{}", bd);
    }

    #[tokio::test]
    async fn get_sheet() {
        let mut df = g_sheets::get_as_dataframe(
            config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET,
            Some(config::SHEETSCRAPER_CONFIGURATION_SHEET),
        )
        .await;

        println!("{:?}", df.column("is_exco"));

        // let out_file = File::create("attd.csv").unwrap();
        // let csv_writer = CsvWriter::new(out_file);
        // csv_writer.has_header(true).finish(&mut df).unwrap();
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

        let mut df =
            g_sheets::get_as_dataframe(config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET, Some(sheet_name))
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
        csv_writer
            .include_header(true)
            .finish(&mut sheet.data)
            .unwrap();
    }

    #[tokio::test]
    async fn test_prog_from_dataframe() {
        init().await;

        let today = chrono::Local::now().date_naive();

        let mut df = g_sheets::get_as_dataframe(
            config::SHEETSCRAPER_NEW_PROGRAM_SHEET,
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

    #[test]
    fn test_calculate_land_sheet_name() {
        let sess = Session::default();
        println!("{:?}", sess);

        let year = chrono::Local::now().date_naive().year();
        for i in (1..=12).into_iter() {
            let date = NaiveDate::from_ymd_opt(year, i, 1).unwrap();

            let sheet_name = calculate_land_sheet_name(date);
            println!("{} -> {}", date, sheet_name)
        }
    }

    #[tokio::test]
    async fn test_asd() {
        init().await;
        let mut res = land(chrono::Local::now().date_naive() + Duration::days(1)).await;
        res.fill_prog(true).await.unwrap();
        println!("{}", res);
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_calculate_excel_col_idx() {
        let x: Vec<_> = (1..26).into_iter().collect();
        let res: Vec<String> = ('A'..'Z').into_iter().map(|c| c.to_string()).collect();

        let x2: Vec<_> = (27..52).into_iter().collect();
        let res2: Vec<String> = ('A'..'Z').into_iter().map(|c| format!("A{}", c)).collect();

        println!("{:?}", x);

        let y: Vec<_> = x
            .into_iter()
            .map(|item| AttdSheet::col_idx_to_excel_alphabetic(item as usize))
            .collect();

        let y2: Vec<_> = x2
            .into_iter()
            .map(|item| AttdSheet::col_idx_to_excel_alphabetic(item as usize))
            .collect();

        println!("{:?}", y);

        assert_eq!(y, res);
        assert_eq!(y2, res2);
    }
}
