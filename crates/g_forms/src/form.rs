//! Public form structs

use std::{fmt::Debug, marker::PhantomData, str::FromStr};

pub use serde_json::Number;

use crate::{
    question::{question_types::*, Question},
    raw::{DateType, FormQuestion, RawInputValidation, RawQuestion, RawQuestionInfo, TimeType},
};

use self::subtypes::{Long, Short};

/// Every response to a question is eventually serialized/generated
/// to a string. This trait implements the logic to convert a form response
/// to that string.
///
/// ```
/// use g_forms::GoogleForm;
///
///
/// ```
pub trait Response {
    fn response(&self) -> Option<String>;
}

/// Unit structs for question subtypes
pub mod subtypes {
    /// Short answer question
    #[derive(Clone, Debug, Default)]
    pub struct Short {}

    /// Long answer question
    #[derive(Clone, Debug, Default)]
    pub struct Long {}
}

/// This represents a form and all its contents
#[derive(Clone, Debug)]
pub struct GoogleForm {
    /// Form id used in the url
    id: String,
    title: String,
    description: String,

    questions: Vec<QuestionHeader>,
}

/// Common question attributes
#[derive(Clone, Debug)]
pub struct QuestionHeader {
    title: Option<String>,
    /// Used for submissions
    id: u64,
    description: Option<String>,
    question_type: QuestionType,
}

impl From<RawQuestion> for QuestionHeader {
    #[rustfmt::skip]
    fn from(value: RawQuestion) -> Self {
        let mut question = QuestionHeader {
            title: value.title,
            id: value.id,
            description: value.description,
            question_type: QuestionType::from(value.question_type),
        };

        match &mut question.question_type {
            QuestionType::ShortAnswer(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
            QuestionType::LongAnswer(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
            QuestionType::MultipleChoice(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
            QuestionType::DropDown(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
            QuestionType::CheckBox(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
            QuestionType::LinearScale(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
            QuestionType::Grid => todo!(),
            // QuestionType::Grid(qn) => todo!(),
            QuestionType::Date(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
            QuestionType::Time(qn) => {
                *qn = Question::try_from(value.additional_info).unwrap()
            }
        }

        question
    }
}

/// One form question
#[derive(Clone, Debug)]
pub enum QuestionType {
    ShortAnswer(Question<ShortAnswer>),
    LongAnswer(Question<LongAnswer>),
    MultipleChoice(Question<MultipleChoice>),
    DropDown(Question<DropDown>),
    CheckBox(Question<CheckBox>),
    LinearScale(Question<LinearScale>),

    Grid,
    // Grid(Question<Grid>),
    Date(Question<Date>),
    Time(Question<Time>),
}

impl From<FormQuestion> for QuestionType {
    fn from(value: FormQuestion) -> Self {
        match value {
            FormQuestion::Short => Self::ShortAnswer(Default::default()),
            FormQuestion::Long => Self::LongAnswer(Default::default()),
            FormQuestion::MultipleChoice => Self::MultipleChoice(Default::default()),
            FormQuestion::DropDown => Self::DropDown(Default::default()),
            FormQuestion::CheckBox => Self::CheckBox(Default::default()),
            FormQuestion::LinearScale => Self::LinearScale(Default::default()),
            FormQuestion::Grid => Self::Grid,
            // FormQuestion::Grid => Self::Grid(Default::default()),
            FormQuestion::Date => Self::Date(Default::default()),
            FormQuestion::Time => Self::Time(Default::default()),
        }
    }
}

/// For open-ended type questions, such as
/// short and long answer questions.
#[derive(Clone, Debug, Default)]
pub struct OpenEndedQuestion<T> {
    marker: PhantomData<T>,

    /// Form response
    response: Option<String>,

    /// Response validation, if any
    validation: Option<InputValidation>,

    /// Error message if response validation fails
    validation_error: Option<String>,
}

impl<T> TryFrom<Vec<RawQuestionInfo>> for OpenEndedQuestion<T> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        // vec should have a valid first element
        // for open ended questions
        let inner = value.into_iter().next().ok_or(())?;

        let mut qn = Self {
            marker: PhantomData,
            response: None,
            validation: None,
            validation_error: None,
        };

        // // the inner vec should also have a single element
        // let raw = inner
        //     .input_validation
        //     .ok_or(())?
        //     .iter()
        //     .next()
        //     .ok_or(())?
        //     .to_owned();

        let raw: Option<RawInputValidation> = inner
            .input_validation
            .and_then(|v| v.iter().next().and_then(|elem| Some(elem.to_owned())));

        qn.validation_error = raw.as_ref().and_then(|v| v.error_text.clone());
        qn.validation = raw.and_then(|v| Some(InputValidation::try_from(v).ok()?));

        Ok(qn)
    }
}

/// Response validation
#[derive(Clone, Debug)]
#[repr(u32)]
#[rustfmt::skip]
pub enum InputValidation {
    /// Greater than
    NumberGT(Number) =          1,

    /// Greater than or equal to
    NumberGTE(Number) =         2,

    /// Less than
    NumberLT(Number) =          3,

    /// Less than or equal to
    NumberLTE(Number) =         4,

    /// Equal to
    NumberEQ(Number) =          5,

    /// Not equal to
    NumberNEQ(Number) =         6,

    /// Between range
    NumberBT(Number, Number) =  7,

    /// Not between range
    NumberNBT(Number, Number) = 8,

    /// Is a valid number
    NumberIsNumber =            9,

    /// Is a whole number
    NumberIsWhole =             10,

    /// Response valid is text contains the pattern
    TextContains(String) =      100,

    /// Response valid if text does not contain the pattern
    TextNotContains(String) =   101,

    /// Response valid if text is a valid url
    TextIsUrl =                 102,

    /// Response valid if text is a valid email
    TextIsEmail =               103,

    /// At least x number of responses checked
    CheckBoxGTE(u32) =          200,

    /// At most x number of responses
    CheckBoxLTE(u32) =          201,

    /// Exactly x number of responses
    CheckBoxEQ(u32) =           204,

    /// Response valid if it is smaller or equal to the maximum length
    LengthMaximumChars(u32) =   202,

    /// Response valid if it is greater or equal to the minimum length
    LengthMinimumChars(u32) =   203,

    /// Response valid if it contains the pattern
    RegexContains(String) =     299,

    /// Response valid if it does not contain the pattern
    RegexNotContains(String) =  300,

    /// Response valid if it matches the pattern
    RegexMatches(String) =      301,

    /// Response valid if it does not match the pattern
    RegexNotMatches(String) =   302,
}

impl TryFrom<RawInputValidation> for InputValidation {
    type Error = ();

    fn try_from(value: RawInputValidation) -> Result<Self, Self::Error> {
        let validation = Self::from_subtype(value.validation_subtype)
            .expect("should match to valid subtype. Did Google change the type assignment?")
            .with_condition(&value.condition);

        match validation {
            Some(v) => Ok(v),
            None => Err(()),
        }
    }
}

impl InputValidation {
    /// Create an InputValidation instance from the `validation_subtype`
    /// field in [RawInputValidation]
    ///
    /// ```
    /// use g_forms::form::InputValidation;
    ///
    /// // subtype '1' exists
    /// let v = InputValidation::from_subtype(1);
    /// assert!(matches!(v, Some(_)));
    ///
    /// // subtype '0' does not exist
    /// let v = InputValidation::from_subtype(0);
    /// assert!(matches!(v, None));
    /// ```
    pub fn from_subtype(subtype: u32) -> Option<Self> {
        match subtype {
            1 => Some(Self::NumberGT(Number::from(0))),
            2 => Some(Self::NumberGTE(Number::from(0))),
            3 => Some(Self::NumberLT(Number::from(0))),
            4 => Some(Self::NumberLTE(Number::from(0))),
            5 => Some(Self::NumberEQ(Number::from(0))),
            6 => Some(Self::NumberNEQ(Number::from(0))),
            7 => Some(Self::NumberBT(Number::from(0), Number::from(0))),
            8 => Some(Self::NumberNBT(Number::from(0), Number::from(0))),
            9 => Some(Self::NumberIsNumber),
            10 => Some(Self::NumberIsWhole),

            100 => Some(Self::TextContains(Default::default())),
            101 => Some(Self::TextNotContains(Default::default())),
            102 => Some(Self::TextIsEmail),
            103 => Some(Self::TextIsUrl),

            200 => Some(Self::CheckBoxGTE(Default::default())),
            201 => Some(Self::CheckBoxLTE(Default::default())),
            204 => Some(Self::CheckBoxEQ(Default::default())),

            202 => Some(Self::LengthMaximumChars(Default::default())),
            203 => Some(Self::LengthMinimumChars(Default::default())),

            299 => Some(Self::RegexContains(Default::default())),
            300 => Some(Self::RegexNotContains(Default::default())),
            301 => Some(Self::RegexMatches(Default::default())),
            302 => Some(Self::RegexNotMatches(Default::default())),

            _ => None,
        }
    }

    /// Sets the condition for input validation.
    ///
    /// The condition as taken from JSON is in an optional vector of strings.
    ///
    /// If the condition does not match it's associated variant, self will
    /// be consumed and return None.
    ///
    /// ```
    /// use g_forms::form::InputValidation;
    ///
    /// // Self::NumberGTE expects a vector with a single string element
    /// // that must parse successfully into a number.
    ///
    /// // successful
    /// let v = InputValidation::NumberGTE(0.into());
    /// let vec: Option<Vec<String>> = Some(vec!["12.5".to_string()]);
    /// let v = v.with_condition(&vec);
    ///
    /// assert!(matches!(v, Some(_)));
    ///
    /// // not successful
    /// let vec: Option<Vec<String>> = Some(vec!["not a number".to_string()]);
    /// let v = InputValidation::NumberGTE(0.into());
    /// let v_fail = v.with_condition(&vec);
    ///
    /// assert!(matches!(v_fail, None));
    /// ```
    pub fn with_condition(mut self, cond: &Option<Vec<String>>) -> Option<Self> {
        match &mut self {
            Self::NumberGT(x)
            | Self::NumberGTE(x)
            | Self::NumberLT(x)
            | Self::NumberLTE(x)
            | Self::NumberEQ(x)
            | Self::NumberNEQ(x) => {
                let num = cond.clone()?.iter().next()?.parse::<Number>().ok()?;

                *x = num;
            }

            Self::NumberBT(x, y) | Self::NumberNBT(x, y) => {
                let pair = cond
                    .clone()?
                    .chunks(2)
                    .next()?
                    .iter()
                    .map(|elem| elem.parse::<Number>().ok())
                    .collect::<Vec<Option<Number>>>();

                let unwrapped_pair = {
                    let mut unwrapped = Vec::new();
                    for item in pair {
                        let u = item?;
                        unwrapped.push(u)
                    }
                    unwrapped
                };

                *x = unwrapped_pair[0].clone();
                *y = unwrapped_pair[1].clone();
            }

            Self::NumberIsNumber => (),
            Self::NumberIsWhole => (),

            Self::TextIsUrl => (),
            Self::TextIsEmail => (),

            Self::LengthMaximumChars(x)
            | Self::LengthMinimumChars(x)
            | Self::CheckBoxEQ(x)
            | Self::CheckBoxGTE(x)
            | Self::CheckBoxLTE(x) => {
                let num = cond.clone()?.iter().next()?.parse::<u32>().ok()?;

                *x = num;
            }
            Self::TextContains(x)
            | Self::TextNotContains(x)
            | Self::RegexContains(x)
            | Self::RegexNotContains(x)
            | Self::RegexMatches(x)
            | Self::RegexNotMatches(x) => {
                let string = cond.clone()?.into_iter().next()?;

                *x = string;
            }
        }

        Some(self)
    }
}

/// Selection type question.
///
/// Applies to:
/// - Multiple choice
/// - Check box
/// - Drop down
#[derive(Clone, Debug, Default)]
pub struct SelectionQuestion {
    /// Vector of available choices
    inner: Vec<SingleSelection>,

    // /// Multi-select?
    // multiple: bool,
    /// Lower and upper limits
    /// for linear scale questions
    limits: Option<SelectionLimits>,

    /// Applicable to checkbox questions only
    validation: Option<InputValidation>,
}

/// Represents a single selection option for
/// questions that consist of selection-type
/// responses.
#[derive(Clone, Debug)]
pub struct SingleSelection {
    /// String response
    pub answer: String,

    /// Marks if option is selected
    pub selected: bool,
}

/// Labels for upper and lower selection limits
#[derive(Clone, Debug, Default)]
pub struct SelectionLimits {
    pub(crate) lower: String,
    pub(crate) upper: String,
}

impl TryFrom<Vec<RawQuestionInfo>> for SelectionQuestion {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        let raw = value.into_iter().next().ok_or(())?;

        let questions = raw
            .dimension_1
            .ok_or(())?
            .into_iter()
            .map(|elem| SingleSelection {
                answer: elem.name,
                selected: false,
            })
            .collect::<Vec<SingleSelection>>();

        let limits: Option<SelectionLimits> = match raw.dimension_2 {
            Some(_limit) => {
                let chunk = _limit.chunks(2).next().ok_or(())?;
                Some(SelectionLimits {
                    lower: chunk[0].clone(),
                    upper: chunk[1].clone(),
                })
            }
            None => None,
        };

        let validation: Option<InputValidation> = match raw.input_validation {
            Some(_iv) => {
                let inner = _iv.into_iter().next().ok_or(())?;
                Some(InputValidation::try_from(inner)?)
            }
            None => None,
        };

        Ok(Self {
            inner: questions,
            limits,
            validation,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct DateQuestion {
    variant: DateType,
    inner: Option<chrono::NaiveDateTime>,
}

impl TryFrom<Vec<RawQuestionInfo>> for DateQuestion {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        let raw = value.into_iter().next().ok_or(())?;
        let date = DateType::try_from(raw.date_type.ok_or(())?)?;

        Ok(Self {
            variant: date,
            inner: None,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct TimeQuestion {
    variant: TimeType,
    inner: Option<chrono::NaiveTime>,
}

impl TryFrom<Vec<RawQuestionInfo>> for TimeQuestion {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        let raw = value.into_iter().next().ok_or(())?;

        Ok(Self {
            variant: raw.time_type.ok_or(())?.inner,
            inner: None,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::raw::RawFormData;

    #[test]
    fn test_repr_enum() {}

    #[test]
    fn test_inputvalidation_tryfrom_raw() {
        let raw = RawInputValidation {
            validation_type: 1,
            validation_subtype: 3,
            condition: Some(vec!["9.99".to_string()]),
            error_text: Some("number needs to be greater than 9".to_string()),
        };

        let res = InputValidation::try_from(raw);
        assert!(matches!(res, Ok(_)));

        let validation = res.unwrap();
        assert!(matches!(validation, InputValidation::NumberLT(_)));
    }

    /// Da big on'e
    #[tokio::test]
    async fn test_raw_to_form() {
        let url = format!(
            "https://docs.google.com/forms/d/e/{}/viewform",
            // "1FAIpQLSfMtt0kvol72F9A2BaLJacr8Xzm9n51KBxVfS8YkDe8SfS5GA"
            // "1FAIpQLSdZJlO9DfU1UQyQ1zgOnGLKrycyxP-eEcpzutfETaki2RgtVw"
            "1FAIpQLSfmF5b8tCebE5ZT_1qw6_C42LA5azs5NboRxuqjJP4XPmlstg"
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

        let questions = des
            .question_blob
            .questions
            .into_iter()
            .map(|raw| QuestionHeader::from(raw))
            .collect::<Vec<QuestionHeader>>();
        println!("{:#?}", questions);
    }
}
