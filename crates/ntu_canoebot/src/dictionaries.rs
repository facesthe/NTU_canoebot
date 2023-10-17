//! Simple APIs for various web dictionaries.
//!

/// A simple urban dictionary interface
pub mod urbandictonary {
    use serde::Deserialize;
    use serde_json::Value;

    const UB_API_URL: &str = "https://api.urbandictionary.com/v0/define";

    #[derive(Clone, Debug, Deserialize)]
    pub struct UrbanDictionaryEntry {
        pub definition: String,
        pub permalink: String,
        pub thumbs_up: u32,
        pub thumbs_down: u32,
        pub author: String,
        pub word: String,
        pub defid: u32,
        pub current_vote: String,
        pub written_on: String,
        pub example: String,
    }

    /// Perform a query
    pub async fn query<T: AsRef<str>>(term: T) -> Option<String> {
        let resp = reqwest::Client::new()
            .get(UB_API_URL)
            .query(&[("term", term.as_ref())])
            .send()
            .await
            .ok()?;

        let res: Value = resp.json().await.ok()?;
        let choices: Vec<UrbanDictionaryEntry> =
            serde_json::from_value(res.get("list").cloned()?).ok()?;

        Some(choices.first()?.definition.clone())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_search_query() {
            let res = query("wow").await;

            println!("{:?}", res);
        }
    }
}

/// A simple wikipedia interface
pub mod wikipedia {
    use std::collections::HashMap;

    use serde_json::Value;

    const WIKI_API_URL: &str = "https://en.wikipedia.org/w/api.php";
    const API_LIMIT: u32 = 10;

    /// Perform a query
    pub async fn query<T: AsRef<str>>(term: T) -> Option<String> {
        let resp = reqwest::Client::new()
            .get(WIKI_API_URL)
            .query(&[
                ("action", "opensearch"),
                ("namespace", "0"),
                ("search", term.as_ref()),
                ("limit", &format!("{}", API_LIMIT)),
                ("format", "json"),
            ])
            .send()
            .await
            .ok()?;

        let data: Value = resp.json().await.ok()?;

        let entries: Vec<String> =
            serde_json::from_value(data.as_array()?.get(1).cloned()?).ok()?;

        get_summary(entries.first()?).await
    }

    /// Gets the article summary of a **known** article name
    async fn get_summary<T: AsRef<str>>(known_query: T) -> Option<String> {
        let resp = reqwest::Client::new()
            .get(WIKI_API_URL)
            .query(&[
                ("action", "query"),
                ("prop", "extracts"),
                ("exintro", ""),
                ("exsectionformat", "plain"),
                ("explaintext", ""),
                ("format", "json"),
                ("titles", known_query.as_ref()),
            ])
            .send()
            .await
            .ok()?;

        let data = resp
            .json::<Value>()
            .await
            .ok()?
            .get("query")?
            .get("pages")?
            .clone();

        // debug_println!("{:#?}", data);

        let map: HashMap<String, Value> = serde_json::from_value(data).ok()?;

        let val = map.values().next()?;
        let res = val.get("extract").unwrap();

        // debug_println!("{:?}", res);

        Some(res.to_string())
    }

    #[cfg(test)]
    mod tests {

        use super::*;

        #[tokio::test]
        async fn test_query() {
            let res = query("earth").await;
            println!("{:?}", res);
        }
    }
}
