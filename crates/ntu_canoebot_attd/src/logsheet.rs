//! Logsheet logic goes here
//!

#[cfg(test)]
mod tests {

    /// Test if g_forms can deserialize form data
    #[tokio::test]
    async fn test_logsheet_valid() {
        let logsheet_id = *ntu_canoebot_config::FORMFILLER_FORM_ID;

        let form = g_forms::GoogleForm::from_id(logsheet_id).await.unwrap();

        println!("{:#?}", form);
    }
}