//! SRC booking interface
//!
//! ```
//! use ntu_src::{SRC_CACHE, SRC_FACILITIES};
//!
//! let num_facilities = SRC_FACILITIES.len();
//! for facility_id in 0..num_facilities {
//!     let facility = SRC_FACILITIES.get_index(facility_id);
//! }
//! ```
//!

use std::{fmt::Display, ops::DerefMut, str::FromStr, sync::Arc};

use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use lazy_static::__Deref;

use serde::{de, Deserialize, Deserializer};
use tokio::sync::Mutex;

use ntu_canoebot_config as config;
use ntu_canoebot_util::debug_println;

const WINDOW_DAYS: usize = 8;

lazy_static::lazy_static! {
    /// Cache for SRC booking slots. 2 - way set associative
    pub static ref SRC_CACHE: SrcCache = SrcCache::default();

    /// Lookup table for src facilities, fixed at runtime
    pub static ref SRC_FACILITIES: SrcFacilities = {

        // let tomlfile: String = match SrcFacilities::find_and_read_file(".configs/srcscraper.config.toml") {
        //     Some(_file) => {_file},
        //     None => panic!()
        // };

        // let toml_val: HashMap<String, Vec<SrcFacility>> = toml::from_str(&tomlfile).expect("failed to parse toml");

        // let inner_vec = toml_val.values().next().expect("map should have one entry");
        let serialized_str = serde_json::to_string(&*config::FACILITIES).unwrap();
        let inner_vec: Vec<SrcFacility> = serde_json::from_str(&serialized_str).unwrap();

        debug_println!("constructed global static SRC_FACILITIES");
        SrcFacilities {
            inner: inner_vec
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
        debug_println!("created SrcCache instance");
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
                tokio::spawn(async move { SrcBookingEntry::get_entry(facil, date_a).await }),
                tokio::spawn(async move { SrcBookingEntry::get_entry(facil, date_b).await }),
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
        debug_println!("cache line {}:{}", facility_num, cache_line as u8);

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
    ///
    /// Set `refresh` to `true` to force a fetch from SRC.
    pub async fn get_facility(
        &self,
        facility_num: u8,
        date: NaiveDate,
        refresh: bool,
    ) -> Option<SrcBookingEntry> {
        let mut lock = self.lock().await;
        let cache_line = &mut lock[facility_num as usize];

        // used for cache replacement
        let mut old_line: usize = 0;
        let mut newer_date: NaiveDate = date;
        debug_println!("fetch date: {}", &date);
        debug_println!(
            "0: {} - {} old: {}\n1: {} - {} old: {}",
            cache_line[0].date,
            cache_line[0].date + Duration::days(WINDOW_DAYS as i64 - 1),
            cache_line[0].old,
            cache_line[1].date,
            cache_line[1].date + Duration::days(WINDOW_DAYS as i64 - 1),
            cache_line[1].old
        );

        let mut hit: Option<usize> = None;

        for (idx, entry) in cache_line.iter().enumerate() {
            if entry.old {
                old_line = idx;
            } else {
                newer_date = entry.date;
            }

            if let None = hit {
                // can't combine if let with another condition
                if (date - entry.date).num_days() < WINDOW_DAYS as i64 && date >= entry.date {
                    hit = Some(idx);
                } else {
                    continue;
                }
            }
        }

        match hit {
            Some(idx) => {
                debug_println!("cache hit: {}", idx);
                if refresh {
                    drop(lock);
                    self.fill(date, facility_num, idx != 0).await.ok()?;

                    let mut lock = self.lock().await;
                    let line = &mut lock[facility_num as usize];
                    return Some(line[idx].clone());
                } else {
                    return Some(cache_line[idx].clone());
                }
            }
            None => {
                debug_println!("cache miss, evicting: {}", old_line);
                // drop(cache_line);
                drop(lock);

                let date_to_fetch = Self::calculate_date_block(newer_date, date);

                match self.fill(date_to_fetch, facility_num, old_line != 0).await {
                    Ok(_) => (),
                    Err(_) => return None,
                }

                return Some(self.lock().await[facility_num as usize][old_line].clone());
            }
        }
    }

    /// Determine the date to fetch,
    /// by referencing the date in existing cache line.
    /// Other cache line will be overwritten.
    fn calculate_date_block(existing_line: NaiveDate, replacement: NaiveDate) -> NaiveDate {
        let diff = (existing_line - replacement).num_days().abs() as usize;

        let next_block_ahead = WINDOW_DAYS;
        let next_block_end = next_block_ahead + WINDOW_DAYS - 1;

        // let block_behind = WINDOW_DAYS;

        match replacement >= existing_line {
            true => {
                // inside existing block
                if diff < next_block_ahead {
                    // return existing
                    // this branch should never be taken
                    return existing_line;
                // ahead and within one block
                } else if diff <= next_block_end {
                    return existing_line + Duration::days(next_block_ahead as i64);
                // ahead and past one block
                } else {
                    return replacement;
                }
            }
            false => {
                // inside existing block
                if diff <= next_block_ahead {
                    return existing_line - Duration::days(next_block_ahead as i64);
                // ahead and past one block
                } else {
                    return replacement;
                }
            }
        }
    }
}

impl SrcBookingEntry {
    /// Retrieve data given a facility and date
    pub async fn get_entry(facility: &SrcFacility, date: NaiveDate) -> Self {
        debug_println!("facility: {}, date: {}", facility.code_name, date.day());
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
    pub fn get_display_table(&self, date: NaiveDate) -> Option<String> {
        let diff = (date - self.date).num_days();
        if diff >= WINDOW_DAYS as i64 {
            return None;
        } else if diff < 0 {
            return None;
        }

        let col: &Vec<SrcSqashedBookingAvailability> =
            self.data.avail_col.get(diff as usize).unwrap();

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

        Some(format!(
            "{}\n{}\n\n{}\nfetched on: {}\nfetch time:    {:.2}s",
            date.format("%d %b %y, %A"),
            facility_name,
            display_str,
            self.fetch_time.format("%H:%M:%S"),
            self.latency as f32 / 1000 as f32
        ))
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
    pub code_name: String,
    /// Number of courts, also for querying SRC
    #[serde(deserialize_with = "from_str")]
    courts: u8,
}

/// Custom deserializer for data represented as strings
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

/// Wrapper around a vector of [SrcFacility]
#[derive(Debug)]
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

        let pretty = booking_entry.get_display_table(date).unwrap();

        // println!("{:#?}", booking_entry.data.avail_col);
        println!("{}", pretty);
    }

    #[test]
    fn test_read_srcscraper_config() {
        for facil in &*SRC_FACILITIES.inner {
            println!("{:?}", facil);
        }
    }

    #[tokio::test]
    async fn test_cache_fill() {
        let original_date = chrono::Local::now().naive_local().date() + Duration::days(-1);
        SRC_CACHE.fill_all().await;

        let facil_no = SRC_FACILITIES.len() - 1;

        let facil = SRC_CACHE
            .get_facility(facil_no as u8, original_date, false)
            .await
            .unwrap();
        println!("{}", facil.get_display_table(original_date).unwrap());

        let date = original_date + Duration::days(7);
        let facil = SRC_CACHE
            .get_facility(facil_no as u8, date, false)
            .await
            .unwrap();
        println!("{}", facil.get_display_table(date).unwrap());

        let date = original_date + Duration::days(9);
        let facil = SRC_CACHE
            .get_facility(facil_no as u8, date, false)
            .await
            .unwrap();
        println!("{}", facil.get_display_table(date).unwrap());

        let date = original_date + Duration::days(-7);
        let facil = SRC_CACHE
            .get_facility(facil_no as u8, date, false)
            .await
            .unwrap();
        println!("{}", facil.get_display_table(date).unwrap());

        let date = original_date + Duration::days(-9);
        let facil = SRC_CACHE
            .get_facility(facil_no as u8, date, false)
            .await
            .unwrap();
        println!("{}", facil.get_display_table(date).unwrap());
    }

    #[tokio::test]
    async fn test_cache_fill_refresh_all() {
        // let original_date = chrono::Local::now().naive_local().date();
        SRC_CACHE.fill_all().await;

        SRC_CACHE.refresh_all().await;
    }

    #[tokio::test]
    async fn test_cache_fill_refresh() {}

    #[test]
    fn test_date_calculation() {
        let base_date = chrono::Local::now().naive_local().date();

        println!("base date: {}", &base_date);
        // check block calculation for
        // dates that fall behind existing line date
        // within 1 block away from existing block
        let base_date_before = base_date - Duration::days(WINDOW_DAYS as i64);

        println!("date before: {}", &base_date_before);

        for offset in 0..WINDOW_DAYS {
            let res = SrcCache::calculate_date_block(
                base_date,
                base_date_before + Duration::days(offset as i64),
            );

            println!(
                "date {} -> {}",
                base_date_before + Duration::days(offset as i64),
                res
            );

            assert_eq!(
                (base_date - res).num_days(),
                WINDOW_DAYS as i64,
                "dates falling within one block ({} days) before existing cache line\
                should be shifted to that block",
                WINDOW_DAYS
            );
        }

        // check for dates that fall after existing line date
        // within 1 block away from existing block
        let base_date_after = base_date + Duration::days(WINDOW_DAYS as i64);
        println!("date after: {}", &base_date_after);

        for offset in 0..WINDOW_DAYS {
            let res = SrcCache::calculate_date_block(
                base_date,
                base_date_after + Duration::days(offset as i64),
            );

            println!(
                "date {} -> {}",
                base_date_after + Duration::days(offset as i64),
                res
            );

            assert_eq!(
                (res - base_date).num_days(),
                WINDOW_DAYS as i64,
                "dates falling within one block ({} days) after existing cache line\
                should be shifted to that block",
                WINDOW_DAYS
            );
        }
    }

    #[test]
    fn test_config_fetch() {
        use ntu_canoebot_config as config;

        let x = config::FACILITIES.clone();
        let serialized = serde_json::to_string_pretty(&x).unwrap();

        println!("{}", serialized);
        let des: Vec<SrcFacility> = serde_json::from_str(&serialized).unwrap();
        println!("{:#?}", des);
    }
}
