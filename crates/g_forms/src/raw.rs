//! Raw data handling
//!
//! All structs here are deserialized from a JSON array
//! because Google hates symbols.
#![allow(unused)]

use std::clone;

use serde_derive::Deserialize;
use serde_json::Value;
use serde_repr::Deserialize_repr;

/// Form data as-is after fetch
#[derive(Clone, Debug, Deserialize)]
struct RawFormData {
    unknown_1: Value,
    question_blob: RawQuestionBlob, // RawQuestionBlob
    unknown_2: String,
    description: String,

    // nulls
    unknown_3: Value,
    unknown_4: Value,
    unknown_5: Value,

    // empty string
    unknown_6: String,

    // null
    unknown_7: Value,

    // 0s
    unknown_8: u32,
    unknown_9: u32,

    unknown_10: Value,
    unknown_11: String,
    unknown_12: u32,
    form_id: String,

    unknown_13: u32,
    unknown_arr: String,

    unknown_14: u32,
    unknown_15: u32,
}

/// Blob of data that contains form questions along with other unknowns
#[derive(Clone, Debug, Deserialize)]
#[serde(
    expecting = "expecting [<form_description>, <questions>, <unknown_vec_1>, <unknown_1>, <unknown_2>, <unknown_vec_2>, <unknown_3>, <unknown_vec_3>, <form_title>, <unknown_number_1>, <unknown_vec_4>, <unknown_4>, <unknown_5>, <unknown_6>, <unknown_7>, <unknown_vec_5>, <unknown_vec_6>, <unknown_8>, <unknown_9>, <unknown_10>, <unknown_11>, <unknown_12>, <unknown_13>, <unknown_14>, <unknown_vec_7>, <unknown_vec_8> array"
)]
struct RawQuestionBlob {
    form_description: String,
    questions: Vec<RawQuestion>, // RawQuestion
    unknown_vec_1: Value,

    unknown_1: Value,
    unknown_2: Value,

    unknown_vec_2: Value,

    unknown_3: Value,

    unknown_vec_3: Value,

    form_title: String,

    unknown_number_1: u32,

    unknown_vec_4: Value,

    unknown_4: Value,
    unknown_5: Value,
    unknown_6: Value,
    unknown_7: Value,

    unknown_vec_5: Value,
    unknown_vec_6: Value,

    unknown_8: Value,
    unknown_9: Value,
    unknown_10: Value,
    unknown_11: Value,
    unknown_12: Value,
    unknown_13: Value,
    unknown_14: Value,

    unknown_vec_7: Value,
    unknown_vec_8: Value,
}

/// Question types
#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[repr(u32)]
pub enum FormQuestion {
    Short,
    Long,
    MultipleChoice,
    DropDown,
    CheckBox,
    Scale,
    Grid,
    Date,
    Time,
}

/// Raw questions information
#[derive(Clone, Debug, Deserialize)]
struct RawQuestion {
    id: u64,
    title: String,
    description: Option<String>,
    question_type: FormQuestion,
    additional_tags: Vec<RawQuestionTags>, // Vec RawQuestionTags

    unknown_2: Value,
    unknown_3: Value,
    unknown_4: Value,
    unknown_5: Value,
    unknown_6: Value,
    unknown_7: Value,

    /// A JSON array containing the question title with html tags.
    ///
    /// schema: `[null, html_title]`
    html_title: Value,

    /// A JSON array containing the question description with html tags.
    ///
    /// schema: `[null, html_description]`
    #[serde(default)]
    html_description: Value,
}

/// Additional tags for question
#[derive(Clone, Debug, Deserialize)]
struct RawQuestionTags {
    id: u64,
    unknown: Value,

    /// Bool in number form
    required: u8,

    #[serde(default)]
    unknown_2: Value,

    /// Contains:
    /// - input validation for open-ended questions
    #[serde(default)]
    unknown_vec: Option<Vec<Value>>,

    #[serde(default)]
    unknown_3: Value,
    #[serde(default)]
    unknown_4: Value,
    #[serde(default)]
    unknown_5: Value,
    #[serde(default)]
    unknown_6: Value,

    #[serde(default)]
    unknown_number: u8
}

#[cfg(test)]
mod test {
    use super::*;
    use scraper::html;

    #[tokio::test]
    async fn test_deserialize_into_raw_form_data() {
        let url = format!(
            "https://docs.google.com/forms/d/e/{}/viewform",
            // "1FAIpQLSfMtt0kvol72F9A2BaLJacr8Xzm9n51KBxVfS8YkDe8SfS5GA"
            "1FAIpQLSdZJlO9DfU1UQyQ1zgOnGLKrycyxP-eEcpzutfETaki2RgtVw"
        );

        let resp = match reqwest::get(url).await {
            Ok(_resp) => _resp,
            Err(_) => panic!(),
        };

        let variable_contents: String = {
            let html_body = match resp.text().await {
                Ok(_html) => _html,
                Err(_) => panic!(),
            };

            let variable_definition = scraper::Html::parse_document(&html_body)
                .select(&scraper::Selector::parse("body script").unwrap())
                .next()
                .expect("should have a variable declared in the script section")
                .text()
                .collect::<String>();

            // println!("variable definition:\n{}", variable_definition);

            let mut variable_content_with_semi = variable_definition
                .split("=")
                .last()
                .unwrap()
                .trim()
                .chars()
                .collect::<Vec<char>>();

            variable_content_with_semi.pop();
            variable_content_with_semi.iter().collect::<String>()
        };
        println!("variable contents:\n{}", &variable_contents);
        let des: RawFormData = serde_json::from_str(&variable_contents).unwrap();

        println!("{:#?}", des);
        // println!("{:#?}", des.question_blob.questions);
    }

    #[test]
    fn test_deserialize_from_known_form() {
        const DATA: &str = r#"[null,["Form description",[[1732369587,"Short answer question (optional)",null,0,[[2123934735,null,0]],null,null,null,null,null,null,[null,"Short answer question (optional)\u003cbr\u003e"]],[311000793,"Short answer question (required)","Question description",0,[[606728391,null,1]],null,null,null,null,null,null,[null,"Short answer question (required)\u003cbr\u003e"],[null,"Question description\u003cbr\u003e"]]],null,null,null,null,null,null,"Form title",66,[null,null,null,2,0,null,1],null,null,null,null,[2],null,null,null,null,null,null,null,null,[null,"Form description\u003cbr\u003e"],[null,"Form title\u003cspan\u003e\u003c/span\u003e"]],"/forms","Untitled form",null,null,null,"",null,0,0,null,"",0,"e/1FAIpQLSdZJlO9DfU1UQyQ1zgOnGLKrycyxP-eEcpzutfETaki2RgtVw",0,"[]",0,0]"#;

        let des: RawFormData = serde_json::from_str(DATA).unwrap();
        println!("{:#?}", des);
    }

    #[test]
    fn test_deserialize_num_to_enum() {

        let json = r#"[0,1]"#;

        let des: Vec<FormQuestion> = serde_json::from_str(json).unwrap();

        println!("{:?}", des);
    }
}
