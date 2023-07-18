//! Google sheets fetch interface
// #![allow(unused)]

/// Returns the contents of a sheet as a csv object
///
/// - `sheet_id`: the resource ID for a sheet. sheet needs to
/// be publicly accessible!
/// - `sheet_name`: the exact sheet name to fetch. passing an
/// invalid name/None will not result in a failure; google will instead
/// return the first sheet created for that sheet_id.
pub async fn get_sheet<T: ToString>(sheet_id: T, sheet_name: Option<T>) -> String {
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
        const SHEET_ID: &str = "1fSt_sO1s7moXPHbxBCD3JIKPa8QIZxtKWYUjD6ElZ-c";

        const NAME: Option<&str> = Some("airlines");

        let resp = get_sheet(SHEET_ID, NAME).await;

        println!("{:?}", resp)
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
