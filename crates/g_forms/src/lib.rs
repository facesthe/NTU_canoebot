//! A simple interface for creating and submitting Google forms.
#![allow(unused)]

pub mod form;
mod raw;

pub use form::GoogleForm;

// impl GoogleForm {
//     /// Link to a new form
//     pub fn from_url<T: ToString>(url: T) -> GoogleForm {
//         todo!()
//     }

//     /// Link to a new form by it's id
//     pub fn from_id<T: ToString>(id: T) -> GoogleForm {
//         GoogleForm {
//             form_id: id.to_string(),
//         }
//     }
// }

#[cfg(test)]
#[allow(unused)]
mod test {
    use serde_derive::{Deserialize, Serialize};

    use super::*;

    #[tokio::test]
    async fn test_get_google_form() {
        let url = format!(
            "https://docs.google.com/forms/d/e/{}/viewform",
            "1FAIpQLSfMtt0kvol72F9A2BaLJacr8Xzm9n51KBxVfS8YkDe8SfS5GA"
        );

        let resp = match reqwest::get(url).await {
            Ok(_resp) => _resp,
            Err(_) => panic!(),
        };

        let contents = resp.text().await.unwrap();
        // println!("{}", contents);

        let html_doc = scraper::Html::parse_document(&contents);

        let res = html_doc
            .select(&scraper::Selector::parse("body script").unwrap())
            .next()
            .unwrap();

        let script_contents = res.text().collect::<String>();

        // println!("{}", script_contents);

        let mut variable_contents: Vec<char> =
            script_contents.split("=").last().unwrap().chars().collect();
        variable_contents.pop(); // remove the last element, which is not part of variable (semicolon)
        let actual_contents: String = variable_contents.iter().collect();

        // println!("{}", actual_contents);

        let jsonified: serde_json::Value = serde_json::from_str(&actual_contents).unwrap();

        let pretty = serde_json::to_string_pretty(&jsonified).unwrap();

        println!("{}", pretty);
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(
        expecting = "expecting [<timestamp>, <open>, <high>, <low>, <close>, <vwap>, <volume>, <trades>] array"
    )]
    pub struct Candle {
        pub timestamp: u32,
        pub open: String,
        pub high: String,
        pub low: String,
        pub close: String,
        pub vwap: String,
        pub volume: String,
        pub trades: u32,
    }

    #[test]
    fn test_serialize_struct_to_array() {
        let instance = Candle::default();

        let ser = serde_json::to_string_pretty(&instance).unwrap();

        println!("{}", ser);

        let sample = r#"[1377648000,"97.0","98.2","96.7","96.7","96.9","2.85000000",6]"#;
        let instance: Candle = serde_json::from_str(sample).unwrap();

        println!("{:?}", instance);
    }
}
