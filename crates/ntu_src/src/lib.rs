//! SRC booking interface
// #![allow(unused)]

use std::{
    collections::HashMap, fs, ops::DerefMut, path::PathBuf, str::FromStr, sync::Arc, time::Instant,
};

use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use lazy_static::__Deref;

use serde_derive::Deserialize;
use tokio::sync::Mutex;

const WINDOW_DAYS: usize = 8;

lazy_static::lazy_static! {
    /// Cache for SRC booking slots. 2 - way set associative
    static ref SRC_CACHE: SrcCache = SrcCache::default();

    /// Lookup table for src facilities, fixed at runtime
    static ref SRC_FACILITIES: SrcFacilities = {

        let tomlfile: String = match SrcFacilities::find_and_read_file(".configs/srcscraper.config.toml") {
            Some(_file) => {_file},
            None => panic!()
        };

        let toml_val: HashMap<String, Vec<SrcFacility>> = toml::from_str(&tomlfile).expect("failed to parse toml");

        let inner_vec = toml_val.values().next().expect("map should have one entry");

        println!("constructed global static SRC_FACILITIES");
        SrcFacilities {
            inner: inner_vec.to_owned()
        }
    };
}

/// Internal cache struct
/// 2 way set associative -> 2 separate copies at each entry
#[derive(Clone, Debug)]
pub struct SrcCache {
    inner: Arc<Mutex<Vec<[SrcBookingEntry; 2]>>>,
}

impl __Deref for SrcCache {
    type Target = Arc<Mutex<Vec<[SrcBookingEntry; 2]>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SrcCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Contains an 8-day block of booking info
#[derive(Clone, Debug)]
pub struct SrcBookingEntry {
    /// Same as code_name in [SrcFacility]
    facility_code: String,
    /// Date of entry
    date: NaiveDate,
    /// Time at which table was fetched
    fetch_time: NaiveTime,
    /// Time taken to fetch table in milliseconds
    latency: i64,
    /// Marks if cache line is the older one (out of 2 cache lines)
    old: bool,
    /// Booking data
    data: SrcSquashedBookingData,
}

impl Default for SrcBookingEntry {
    fn default() -> Self {
        Self {
            facility_code: Default::default(),
            date: Default::default(),
            fetch_time: chrono::Local::now().naive_local().time(),
            latency: Default::default(),
            old: Default::default(),
            data: Default::default(),
        }
    }
}

/// Contains the 8-day consecutive block of booking data
/// for a particular facility
#[derive(Clone, Debug)]
struct SrcBookingData {
    /// Vector of time strings.
    /// This data is "just visiting", and is not processed.
    time_col: Vec<String>,

    /// (courts x time_slots) by 8 matrix of booking availability
    data: Vec<Vec<SrcBookingAvailability>>,
}

impl From<table_extract::Table> for SrcBookingData {
    fn from(value: table_extract::Table) -> Self {
        let mut availability_matrix: Vec<Vec<SrcBookingAvailability>> = Vec::new();
        let mut time_col: Vec<String> = Vec::new();
        // set the times
        // value.iter().skip(n)

        for row in value.iter() {
            let _row = row.as_slice();

            // get time data
            if _row.len() > WINDOW_DAYS + 1 {
                time_col.push(_row[0].clone())
            }

            // take the last 8 elements
            let actual: Vec<String> = _row
                .iter()
                .rev()
                .take(WINDOW_DAYS)
                .rev()
                .map(|elem| elem.to_owned())
                .collect();
            let parsed: Vec<SrcBookingAvailability> =
                actual.iter().map(|elem| elem.parse().unwrap()).collect();

            availability_matrix.push(parsed)
        }

        Self {
            time_col,
            data: availability_matrix,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct SrcSquashedBookingData {
    time_col: Vec<String>,
    /// Matrix of **columns** that contain **rows**
    avail_col: Vec<Vec<SrcSqashedBookingAvailability>>,
}

/// Availability data squashed for each time slot
#[derive(Clone, Debug)]
struct SrcSqashedBookingAvailability {
    /// Time slot string
    time: String,
    /// Total slots for that time period
    total: u8,
    /// Slots available
    available: u8,
    /// If all slots are booked by the same entity,
    /// it will be inside here
    unavailable: Option<String>,
}

impl From<SrcBookingData> for SrcSquashedBookingData {
    fn from(value: SrcBookingData) -> Self {
        let slots_per_time_slot: usize = value.data.len() / value.time_col.len();

        let mut squashed_matrix: Vec<Vec<SrcSqashedBookingAvailability>> = Vec::new();

        // iterate over columns
        for col_idx in 0..WINDOW_DAYS {
            let col: Vec<SrcBookingAvailability> = value
                .data
                .iter()
                .map(|row| row.get(col_idx).unwrap().to_owned())
                .collect();

            let mut squashed_col: Vec<SrcSqashedBookingAvailability> = Vec::new();

            let _: Vec<()> = col
                .chunks(slots_per_time_slot)
                .enumerate()
                .map(|(idx, block)| {
                    let avail: u8 = block
                        .iter()
                        .map(|item| {
                            if let SrcBookingAvailability::Available = item {
                                1
                            } else {
                                0
                            }
                        })
                        .sum();

                    let mut unavail: Option<String> = None;
                    // check for all matching
                    if avail == 0 {
                        let first = block[0].clone();

                        // checking if all elements are the same
                        if block.iter().all(|elem| elem == &first) {
                            if let SrcBookingAvailability::Unavailable(_who) = first {
                                unavail = Some(_who)
                            } else if let SrcBookingAvailability::Closed = first {
                                unavail = Some("CLOSED".to_string())
                            }
                        }
                    }

                    squashed_col.push(SrcSqashedBookingAvailability {
                        time: value.time_col[idx].clone(),
                        total: slots_per_time_slot as u8,
                        available: avail,
                        unavailable: unavail,
                    });
                })
                .collect();

            squashed_matrix.push(squashed_col);
        }

        SrcSquashedBookingData {
            time_col: value.time_col,
            avail_col: squashed_matrix,
        }
    }
}

impl SrcSqashedBookingAvailability {
    /// Formats the time and availability into 2 strings
    pub fn to_string_tuple(&self) -> (String, String) {
        let time = self.time.clone();
        let avail = {
            if let Some(_unavail) = &self.unavailable {
                _unavail.clone()
            } else {
                format!("{}/{}", self.available, self.total)
            }
        };

        (time, avail)
    }
}

/// Shows the booking status for one time slot
#[derive(Clone, Debug, PartialEq)]
enum SrcBookingAvailability {
    /// Available to book
    Available,
    /// Made unavailable by SRC
    Closed,
    /// Booked, contains details of person/org
    Unavailable(String),
}

impl FromStr for SrcBookingAvailability {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "avail" => Ok(Self::Available),
            "closed" => Ok(Self::Closed),
            _ => Ok(Self::Unavailable(s.to_owned())),
        }
    }
}

impl Default for SrcCache {
    fn default() -> Self {
        let mut inner: Vec<[SrcBookingEntry; 2]> = Vec::new();

        let num_entries = SRC_FACILITIES.len();

        for _idx in 0..num_entries {
            inner.push([Default::default(), Default::default()]);
        }
        println!("created SrcCache instance");
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl SrcCache {
    /// Refreshes the data inside all cache lines
    pub async fn refresh_all(&self) {
        let mut lock = self.lock().await;

        let mut src_handles = Vec::new();

        for (idx, facil) in SRC_FACILITIES.iter().enumerate() {
            let date_a = lock[idx][0].date;
            let date_b = lock[idx][1].date;

            src_handles.push(vec![
                tokio::spawn(async move {SrcBookingEntry::get_entry(facil, date_a).await}),
                tokio::spawn(async move {SrcBookingEntry::get_entry(facil, date_b).await})
            ]);
        }

        for (set, set_handle) in src_handles.into_iter().enumerate() {
            for (line, entry) in set_handle.into_iter().enumerate() {
                let booking_entry = entry.await;
                match booking_entry {
                    Ok(_entry) => lock[set][line] = _entry,
                    Err(_) => (),
                }
            }
        }
    }

    /// Repopulate the cache with fresh data.
    /// Overwrites all data previously inside.
    /// References the current day.
    ///
    /// One 8-day block is populated starting from the current day,
    /// the other 8-day block is populated with the previous 8 days.
    ///
    /// NTU might change the number of days returned for each table,
    /// but that's a problem for future me hoho
    pub async fn fill_all(&self) {
        let line_2_date = chrono::Local::now().naive_local().date();
        let line_1_date = line_2_date - Duration::days(WINDOW_DAYS as i64);

        // self.inner.lock().await.clear();

        let mut src_handles = Vec::new();

        let _self_ref = Arc::new(self);
        for facil in SRC_FACILITIES.iter() {
            let handle_a =
                tokio::spawn(async move { SrcBookingEntry::get_entry(facil, line_1_date).await });

            let handle_b =
                tokio::spawn(async move { SrcBookingEntry::get_entry(facil, line_2_date).await });

            src_handles.push(vec![handle_a, handle_b]);
        }

        let mut lock = self.lock().await;
        for (idx, handles) in src_handles.into_iter().enumerate() {
            for (cache_line, handle) in handles.into_iter().enumerate() {
                match handle.await {
                    Ok(data) => {
                        lock[idx][cache_line] = data;
                        // flip the other entry to old
                        lock[idx][1 - cache_line].old = !lock[idx][1 - cache_line].old;
                    }
                    Err(_) => (), // do not update on error
                }
            }
        }
    }

    /// Populate a single cache line with fresh data.
    /// The date param represents the start date of a consecutive 8-day block.
    /// Changes the age of other cache entry in line to "old"
    async fn fill(
        &self,
        date: NaiveDate,
        facility_num: u8,
        cache_line: bool,
    ) -> Result<(), errors::FacilityError> {
        println!("cache line {}:{}", facility_num, cache_line as u8);

        let facility = {
            let res = SRC_FACILITIES.get_index(facility_num as usize);
            match res {
                Some(_facility) => _facility,
                None => return Err(errors::FacilityError {}),
            }
        };

        let entry = SrcBookingEntry::get_entry(&facility, date).await;
        let mut lock = self.lock().await;
        lock[facility_num as usize][cache_line as usize] = entry;

        // set other as old
        let other = &mut lock[facility_num as usize][!cache_line as usize];
        other.old = true;

        Ok(())
    }

    /// Retrieves facility booking data for a particular facility.
    /// Fetches from SRC if date does not exist.
    pub async fn get_facility(&self, facility_num: u8, date: NaiveDate) -> SrcBookingEntry {
        let mut lock = self.lock().await;
        let cache_line = &mut lock[facility_num as usize];

        // used for cache replacement
        let mut old_line: usize = 0;
        let mut newer_date: NaiveDate = date;

        let mut hit: Option<usize> = None;

        for (idx, entry) in cache_line.iter().enumerate() {
            if entry.old {
                old_line = idx;
            } else {
                newer_date = entry.date;
            }

            if (date - entry.date).num_days() < WINDOW_DAYS as i64 {
                hit = Some(idx)
            } else {
                continue;
            }
        }

        match hit {
            Some(idx) => return cache_line[idx].clone(),
            None => {
                // find a non-overlapping block
                // is the new date smaller than the date in other cache line
                let is_negative: bool = !(newer_date > date);
                let diff = (newer_date - date).num_days().abs();

                // date adjustment necessary
                if diff < WINDOW_DAYS as i64 {
                    let fetch_date = newer_date
                        + Duration::days( // some quick math so that I save on branches
                            WINDOW_DAYS as i64 * !is_negative as i64
                                - WINDOW_DAYS as i64 * is_negative as i64,
                        );
                    match self.fill(fetch_date, facility_num, old_line != 0).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                } else {
                    match self.fill(date, facility_num, old_line != 0).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                }

                return cache_line[old_line].clone();
            }
        }
    }
}

impl SrcBookingEntry {
    /// Retrieve data given a facility and date
    pub async fn get_entry(facility: &SrcFacility, date: NaiveDate) -> Self {
        println!("facility: {}, date: {}", facility.code_name, date.day());
        let start_time = chrono::Local::now().naive_local().time();
        let table = facility.get_table(date).await;
        let end_time = chrono::Local::now().naive_local().time();

        let data = SrcBookingData::from(table);
        let squashed = SrcSquashedBookingData::from(data);

        Self {
            facility_code: facility.code_name.clone(),
            date,
            fetch_time: end_time,
            latency: (end_time - start_time).num_milliseconds(),
            old: false,
            data: squashed,
        }
    }

    /// Returns a formatted string to be sent to a user,
    /// given a date
    pub fn get_display_table(&self, date: NaiveDate) -> String {
        let col_no = (date - self.date).num_days();

        let col: &Vec<SrcSqashedBookingAvailability> =
            self.data.avail_col.get(col_no as usize).unwrap();

        // println!("{:#?}", col);

        let mut display_str = String::new();

        let tup_vec: Vec<(String, String)> =
            col.iter().map(|elem| elem.to_string_tuple()).collect();

        let max = *tup_vec
            .iter()
            .map(|(a, b)| (a.len(), b.len()))
            .collect::<Vec<(usize, usize)>>()
            .iter()
            .max()
            .expect("unable to iterate to find max values");

        // header
        display_str.push_str(&format!(
            "{:<width_a$}  {:>width_b$}\n",
            "time",
            "slots",
            width_a = max.0,
            width_b = max.1
        ));

        for (time, avail) in tup_vec.iter() {
            display_str.push_str(&format!(
                "{:<width_a$}  {:>width_b$}\n",
                time,
                avail,
                width_a = max.0,
                width_b = max.1
            ))
        }

        let facility_name = match SRC_FACILITIES.get_code(&self.facility_code) {
            Some(facil) => facil.name,
            None => "".to_string(),
        };


        format!(
            "{}\n{}\n\n{}\nfetched on: {}\nfetch time:    {:.2}s",
            date.format("%d %b %y, %A"),
            facility_name,
            display_str,
            self.fetch_time.format("%H:%M:%S"),
            self.latency as f32 / 1000 as f32
        )
    }
}

/// Contains information about one SRC facility
#[derive(Clone, Debug, Deserialize)]
pub struct SrcFacility {
    /// Full name as listed on SRC website
    pub name: String,
    /// Short form for display
    #[serde(rename = "shortname")]
    pub short_name: String,
    /// Code name for querying SRC
    #[serde(rename = "codename")]
    code_name: String,
    /// Number of courts, also for querying SRC
    courts: u8,
}

/// Wrapper around a vector of [SrcFacility]
pub struct SrcFacilities {
    inner: Vec<SrcFacility>,
}

impl __Deref for SrcFacilities {
    type Target = Vec<SrcFacility>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SrcFacility {
    /// Fetch the table from the SRC and perform some formatting
    pub async fn get_table(&self, date: NaiveDate) -> table_extract::Table {
        let date_string = date.format("%d-%b-%Y");

        let request_url = format!("https://wis.ntu.edu.sg/pls/webexe88/srce_smain_s.srce$sel31_v?choice=1&fcode={}&fcourt={}&ftype=2&p_date={}&p_mode=2", self.code_name, self.courts, date_string);

        let resp = reqwest::get(request_url).await.unwrap();
        let content = resp.text().await.unwrap();

        table_extract::Table::find_first(&content)
            .expect("Unable to find table inside HTML response")
    }
}

impl SrcFacilities {
    /// Attempts to find a file and read it.
    /// If it is unable to find the file, it goes up one parent
    /// and continues.
    fn find_and_read_file(path: &str) -> Option<String> {
        let path = PathBuf::from(path);

        let mut curdir = std::env::current_dir().expect("failed to get current dir");

        loop {
            println!("curdir: {:?}", &curdir);

            if curdir.join(&path).exists() {
                break;
            } else {
                match curdir.parent() {
                    Some(_path) => curdir = PathBuf::from(_path),
                    None => return None,
                }
            }
        }

        fs::read_to_string(curdir.join(path)).ok()
        // todo!()
    }

    /// Create facility table
    pub fn from_string(string: &str) -> Result<Self, toml::de::Error> {
        let res: Vec<SrcFacility> = toml::de::from_str(string)?;
        Ok(SrcFacilities { inner: res })
    }

    /// Returns the src facility given its index
    pub fn get_index(&self, idx: usize) -> Option<SrcFacility> {
        self.inner.get(idx).cloned()
    }

    /// Returns the src facility given its code name
    pub fn get_code(&self, code: &str) -> Option<SrcFacility> {

        let res = self.inner.iter().find(|elem| elem.code_name == code);

        res.cloned()
    }
}

mod errors {

    /// Facility does not exist
    pub struct FacilityError {}
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[tokio::test]
    async fn test_retrieval() {
        let facility = SrcFacility {
            name: "whatever".to_string(),
            short_name: "whatever".to_string(),
            code_name: "WG".to_string(),
            courts: 20,
        };

        let datetime = chrono::Local::now().naive_local();
        let start_time = tokio::time::Instant::now();

        let content = facility.get_table(datetime.date()).await;
        let end_time = tokio::time::Instant::now();

        let time_taken = end_time - start_time;

        // println!("{:?}", content);
        // println!("{}", content.headers());
        let header = content.headers();
        let mut header_vec: Vec<String> = vec!["".to_string(); header.len()];

        for (key, val) in header.iter() {
            header_vec[*val] = key.clone()
        }

        // header.iter().map(|(key, val)| {
        //     header_vec[*val] = key.clone();
        //     // header_vec.
        // });

        println!("{:?}\n\n", header_vec);
        println!("{:?}", header);

        // for row in content.iter() {
        //     println!("{:?}", row.as_slice());
        // }

        let data = SrcBookingData::from(content);

        println!("{:?}", data);
        println!("time taken: {:?}", time_taken);
    }

    #[test]
    fn test_deserialize_src_booking_status() {
        let booking = "CLOSED";
        let b_status: SrcBookingAvailability = booking.parse().unwrap();
        assert_eq!(b_status, SrcBookingAvailability::Closed);

        let booking: &str = "Avail";
        let b_status: SrcBookingAvailability = booking.parse().unwrap();
        assert_eq!(b_status, SrcBookingAvailability::Available);

        let booking = "SXXXX123A";
        let b_status: SrcBookingAvailability = booking.parse().unwrap();
        assert_eq!(
            b_status,
            SrcBookingAvailability::Unavailable(booking.to_owned())
        );
    }

    #[tokio::test]
    async fn test_squash_data() {
        let facility = SrcFacility {
            name: "whatever".to_string(),
            short_name: "whatever".to_string(),
            code_name: "BB".to_string(),
            courts: 6,
        };

        let date = NaiveDate::from_ymd_opt(2023, 07, 03).unwrap();
        let content = facility.get_table(date).await;

        let data = SrcBookingData::from(content);

        let squashed = SrcSquashedBookingData::from(data);

        println!("{:#?}", squashed);
    }

    #[tokio::test]
    async fn test_get_entry() {
        let facility = SrcFacility {
            name: "whatever".to_string(),
            short_name: "whatever".to_string(),
            code_name: "WG".to_string(),
            courts: 20,
        };
        let date = NaiveDate::from_ymd_opt(2023, 07, 03).unwrap();

        let booking_entry = SrcBookingEntry::get_entry(&facility, date).await;

        let pretty = booking_entry.get_display_table(date);

        // println!("{:#?}", booking_entry.data.avail_col);
        println!("{}", pretty);
    }

    #[test]
    fn test_read_srcscraper_config() {
        // THIS IS TEMP THE PATH SHOULD CHANGE
        let tomlfile: String =
            match SrcFacilities::find_and_read_file(".configs/srcscraper.config.toml") {
                Some(_file) => _file,
                None => panic!(),
            };

        let toml_val: HashMap<String, Vec<SrcFacility>> =
            toml::from_str(&tomlfile).expect("failed to read toml file");
        let _x = toml_val.values().next().unwrap();
    }

    #[tokio::test]
    async fn test_cache_fill() {
        let date = chrono::Local::now().naive_local().date();
        SRC_CACHE.fill_all().await;

        let facil = SRC_CACHE.get_facility(0, date).await;
        println!("{}", facil.get_display_table(date));

        let facil = SRC_CACHE.get_facility(0, date + Duration::days(7)).await;
        println!("{}", facil.get_display_table(date));
    }
}
