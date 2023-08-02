//! Handles update/reload from the configuration sheets

use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use polars::prelude::DataFrame;

use crate::{
    dataframe_cell_to_string, Config, ATTENDANCE_SHEETS, BOATS, BOAT_ALLOCATIONS, NAMES_CERTS,
    SHORTENED_NAMES,
};
use ntu_canoebot_config as config;

/// Performs lookup and stuff and updates lazy-static globals
async fn update_config_from_df(
    df: &DataFrame,
    config: Config,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // SHORTENED_NAMES
    let names_lookup = df
        .columns([
            &*config::SHEETSCRAPER_COLUMNS_ATTD_NAME,
            &*config::SHEETSCRAPER_COLUMNS_ATTD_SHORT_NAME,
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
            &*config::SHEETSCRAPER_COLUMNS_ATTD_BOAT_PRIMARY,
            &*config::SHEETSCRAPER_COLUMNS_ATTD_BOAT_ALTERNATE,
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
            &*config::SHEETSCRAPER_COLUMNS_ATTD_NAME,
            &*config::SHEETSCRAPER_COLUMNS_ATTD_CERTIFICATION,
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
            let pri_boat = if pri.len() == 0 { None } else { Some(pri) };

            let alt_boat = if alt.len() == 0 { None } else { Some(alt) };

            Some((name, (pri_boat, alt_boat)))
        })
        .collect::<HashMap<String, (Option<String>, Option<String>)>>();

    // println!("boat allocations: {:#?}", allocations);
    let mut lock = BOAT_ALLOCATIONS[config as usize].write().await;
    lock.clear();
    lock.extend(allocations);

    Ok(())
}

/// Initialize/reload from the configs sheet
pub async fn init() {
    for (idx, sheet_id) in ATTENDANCE_SHEETS.iter().enumerate() {
        let df =
            g_sheets::get_as_dataframe(sheet_id, Some(*config::SHEETSCRAPER_CONFIGURATION_SHEET))
                .await;
        update_config_from_df(&df, idx.into()).await.unwrap()
    }
}
