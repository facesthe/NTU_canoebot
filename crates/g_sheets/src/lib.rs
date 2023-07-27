//! Google sheets fetch interface
// #![allow(unused)]

use std::io::Cursor;

use polars::prelude::{CsvReader, DataFrame, SerReader};

/// Returns the contents of a sheet as a csv object
///
/// - `sheet_id`: the resource ID for a sheet. sheet needs to
/// be publicly accessible!
/// - `sheet_name`: the exact sheet name to fetch. passing an
/// invalid name/None will not result in a failure; google will instead
/// return the first sheet created for that sheet_id.
pub async fn get_as_csv<T: ToString>(sheet_id: T, sheet_name: Option<T>) -> String {
    let url = format!(
        "https://docs.google.com/spreadsheets/d/{}/gviz/tq?tqx=out:csv&sheet={}",
        sheet_id.to_string(),
        sheet_name
            .and_then(|s| Some(s.to_string()))
            .unwrap_or("".to_string())
    );

    let resp = match reqwest::get(url).await {
        Ok(_resp) => _resp,
        Err(_) => panic!(),
    };

    resp.text().await.unwrap()
}

/// Returns the contents of a sheet as a polars dataframe
pub async fn get_as_dataframe<T: ToString>(sheet_id: T, sheet_name: Option<T>) -> DataFrame {
    let csv_str = get_as_csv(sheet_id, sheet_name).await;

    let curs = Cursor::new(csv_str);

    let df = CsvReader::new(curs).finish().unwrap();

    df
}

#[cfg(test)]
mod tests {

    use std::marker::PhantomData;

    use super::*;

    pub struct ASD<T> {
        marker: PhantomData<T>,
    }

    #[derive(Default)]
    pub struct MarkerA {}
    #[derive(Default)]
    pub struct MarkerB {}

    /// Trait for markers. Only valid unit types that implement this
    /// trait can be instantiated.
    pub trait MarkerTrait {}
    impl MarkerTrait for MarkerA {}
    impl MarkerTrait for MarkerB {}

    impl<T: MarkerTrait> Default for ASD<T> {
        fn default() -> Self {
            Self {
                marker: Default::default(),
            }
        }
    }

    impl<T: MarkerTrait> ASD<T> {
        fn private_fn(&self) {}

        pub fn common_public_fn(&self) {}
    }

    impl ASD<MarkerA> {
        /// Public function for struct that contains [MarkerA]
        pub fn public_fn_a(&self) {
            self.private_fn()
        }
    }

    /// Test fetching a google resource sheet
    #[tokio::test]
    async fn test_get_sheet() {
        // const SHEET_ID: &str = "1fSt_sO1s7moXPHbxBCD3JIKPa8QIZxtKWYUjD6ElZ-c";
        const SHEET_ID: &str = "1O76QJOFOypuB8Ri62YEUnXF9r8YfLtMzTg3UsS9rzvQ";

        const NAME: Option<&str> = Some("JUL-2023");

        let resp = get_as_csv(SHEET_ID, NAME).await;

        // let csv_str = resp.().await;

        println!("{}", resp);

        let mut reader = csv::Reader::from_reader(resp.as_bytes());

        // for rec in reader.records() {
        //     let rec = rec.unwrap();
        //     // let
        // }

        let matrix = reader
            .records()
            .into_iter()
            .map(|row| {
                let row = row.unwrap();
                row.iter()
                    .map(|cell| cell.to_owned())
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();

        for row in matrix {
            println!("row size: {}", row.len())
        }
    }

    #[tokio::test]
    async fn test_get_as_dataframe() {
        const SHEET_ID: &str = "1O76QJOFOypuB8Ri62YEUnXF9r8YfLtMzTg3UsS9rzvQ";

        const NAME: Option<&str> = Some("JUL-2023");

        let df = get_as_dataframe(SHEET_ID, NAME).await;

        println!("{:?}", df.head(None));
    }
}

#[cfg(test)]
mod separate_tests {
    use crate::tests::{MarkerA, MarkerB, ASD};

    #[test]
    fn test_stateful_structs() {
        let a = ASD::<MarkerA>::default();
        let b = ASD::<MarkerB>::default();

        a.common_public_fn();
        b.common_public_fn();

        a.public_fn_a();
    }
}
