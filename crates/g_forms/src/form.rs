//! Public form structs
#![allow(unused)]

use std::str::FromStr;

pub use serde_json::Number;

use crate::raw::{
    DateType, FormQuestion, RawInputValidation, RawQuestion, RawQuestionInfo, TimeType,
};

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
    fn response(&self) -> Result<String, ()>;
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
    fn from(value: RawQuestion) -> Self {
        let mut question = QuestionHeader {
            title: value.title,
            id: value.id,
            description: value.description,
            question_type: QuestionType::from(value.question_type),
        };

        // #[rustfmt::skip]
        match &mut question.question_type {
            QuestionType::ShortAnswer(qn) | QuestionType::LongAnswer(qn) => {
                todo!()
            }
            //  => todo!(),
            QuestionType::MultipleChoice => todo!(),
            QuestionType::DropDown => todo!(),
            QuestionType::CheckBox => todo!(),
            QuestionType::LinearScale => todo!(),
            QuestionType::Grid => todo!(),
            QuestionType::Date(qn) => {
                todo!()
            }
            QuestionType::Time(qn) => {
                todo!()
            }
        }

        todo!()
    }
}

/// One form question
#[derive(Clone, Debug)]
pub enum QuestionType {
    ShortAnswer(OpenEndedQuestion),
    LongAnswer(OpenEndedQuestion),
    MultipleChoice,
    DropDown,
    CheckBox,
    LinearScale,
    Grid,
    Date(DateQuestion),
    Time(TimeQuestion),
}

impl From<FormQuestion> for QuestionType {
    fn from(value: FormQuestion) -> Self {
        match value {
            FormQuestion::Short => Self::ShortAnswer(Default::default()),
            FormQuestion::Long => Self::LongAnswer(Default::default()),
            FormQuestion::MultipleChoice => Self::MultipleChoice,
            FormQuestion::DropDown => Self::DropDown,
            FormQuestion::CheckBox => Self::CheckBox,
            FormQuestion::LinearScale => Self::LinearScale,
            FormQuestion::Grid => Self::Grid,
            FormQuestion::Date => Self::Date(Default::default()),
            FormQuestion::Time => Self::Time(Default::default()),
        }
    }
}

/// For open-ended type questions, such as
/// short and long answer questions.
#[derive(Clone, Debug, Default)]
pub struct OpenEndedQuestion {
    /// Form response
    response: Option<String>,

    /// Response validation, if any
    validation: Option<InputValidation>,

    /// Error message if response validation fails
    validation_error: Option<String>,
}

impl TryFrom<Vec<RawQuestionInfo>> for OpenEndedQuestion {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        // vec should have a valid first element
        // for open ended questions
        let inner = value.into_iter().next().ok_or(())?;

        let mut qn = Self {
            response: None,
            validation: None,
            validation_error: None,
        };

        // the inner vec should also have a single element
        let raw = inner
            .input_validation
            .ok_or(())?
            .iter()
            .next()
            .ok_or(())?
            .to_owned();

        qn.validation = Some(InputValidation::try_from(raw.clone())?);
        qn.validation_error = raw.error_text;

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

            Self::LengthMaximumChars(x) | Self::LengthMinimumChars(x) => {
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
#[derive(Clone, Debug)]
pub struct SelectionQuestion {
    inner: Vec<SingleSelection>
}

/// Represents a single selection option for
/// questions that consist of selection-type
/// responses.
#[derive(Clone, Debug)]
pub struct SingleSelection {
    name: String,
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
    use super::InputValidation;
    use super::*;

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

    #[test]
    fn test_openendedqn_tryfrom_vec_rawqninfo() {
        // z
    }
}
