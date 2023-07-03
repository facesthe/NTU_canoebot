//! SRC booking interface
#![allow(unused)]

use std::{
    any,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Instant,
};

use chrono::{Datelike, Local, NaiveDate};
use serde::{__private::de, de::Error};
use serde_derive::{Deserialize, Serialize};
use toml::Table;

lazy_static::lazy_static! {
    /// Cache for SRC booking slots. 2 - way set associative
    static ref SRC_CACHE: Arc<Mutex<SrcCache>> = Arc::new(Mutex::new(SrcCache::default()));

    /// Lookup table for src facilities, fixed at runtime
    static ref SRC_FACILITIES: SrcFacilities = {
        todo!()
    };
}

/// Internal cache struct
/// 2 way set associative -> 2 separate copies at each entry
#[derive(Clone, Debug)]
struct SrcCache {
    inner: Vec<[SrcBookingEntry; 2]>,
}

/// Contains an 8-day block of booking info
#[derive(Clone, Debug)]
struct SrcBookingEntry {
    /// Date of entry
    date: NaiveDate,
    /// Time at which table was fetched
    fetch_time: Instant,
    /// Time taken to fetch table in seconds
    latency: f32,
    /// Marks if cache line is the older one (out of 2 cache lines)
    old: bool,
    /// Booking data
    data: SrcSquashedBookingData,
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

            /// get time data
            if _row.len() > 8 + 1 {
                time_col.push(_row[0].clone())
            }

            /// take the last 8 elements
            let actual: Vec<String> = _row
                .iter()
                .rev()
                .take(8)
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

#[derive(Clone, Debug)]
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

        /// iterate over columns
        for col_idx in (0..8) {
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

                    ()
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
        Self { inner: Vec::new() }
    }
}

impl SrcCache {
    /// Populate/repopulate the cache with fresh data.
    /// Overwrites all data previously inside.
    pub async fn fill_all(&mut self) {}

    /// Populate a single cache line with fresh data.
    /// The date param represents the start date of a consecutive 8-day block.
    pub async fn fill(
        &mut self,
        date: NaiveDate,
        facility_num: u8,
        cache_line: u8,
    ) -> Result<(), errors::FacilityError> {
        let facility = {
            let res = SRC_FACILITIES.get_index(facility_num as usize);
            match res {
                Some(_facility) => _facility,
                None => return Err(errors::FacilityError {}),
            }
        };

        Ok(())
    }
}

impl SrcBookingEntry {
    /// Retrieve data given a facility and date
    pub async fn get_entry(facility: &SrcFacility, date: NaiveDate) -> Self {
        let start_time = Instant::now();
        let table = facility.get_table(date).await;
        let end_time = Instant::now();

        let data = SrcBookingData::from(table);
        let squashed = SrcSquashedBookingData::from(data);

        Self {
            date,
            fetch_time: end_time,
            latency: (end_time - start_time).as_secs_f32(),
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

        let max = tup_vec
            .clone()
            .iter()
            .map(|(a, b)| (a.len(), b.len()))
            .collect::<Vec<(usize, usize)>>()
            .iter()
            .max()
            .expect("unable to iterate to find max values")
            .clone();

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

        display_str
    }
}

/// Contains information about one SRC facility
#[derive(Clone, Debug, Deserialize)]
pub struct SrcFacility {
    /// Full name as listed on SRC website
    name: String,
    /// Short form for display
    short_name: String,
    /// Code name for querying SRC
    code_name: String,
    /// Number of courts, also for querying SRC
    courts: u8,
}

/// Wrapper around a vector of [SrcFacility]
pub struct SrcFacilities {
    inner: Vec<SrcFacility>,
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
}

/// Returns the booking table for a particular day, along with the next 7 days.
/// This returns the raw table direct from the page
async fn get_booking_table() {}

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

        let pretty = booking_entry.get_display_table(date);

        // println!("{:#?}", booking_entry.data.avail_col);
        println!("{}", pretty);
    }
}

// const x: &str = "https://wis.ntu.edu.sg/pls/webexe88/srce_smain_s.srce$sel31_v?choice=1&fcode=WG&fcourt=20&ftype=2&p_date=03-JUL-2023&p_mode=2";
