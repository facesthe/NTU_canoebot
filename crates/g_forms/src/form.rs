//! Public form structs
#![allow(unused)]

use crate::raw::TimeType;

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
pub struct  QuestionHeader {
    title: String,
    /// Used for submissions
    id: String,
    description: Option<String>,
    question_type: QuestionType
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
    Time(TimeQuestion)
}


/// For open-ended type questions, such as
/// short and long answer questions.
#[derive(Clone, Debug)]
pub struct OpenEndedQuestion {
    /// Form response
    response: Option<String>,

    /// Response validation, if any
    validation: Option<OpenEndedValidation>,

    /// Error message if response validation fails
    validation_error: String,
}

/// Response validation
#[derive(Clone, Debug)]
pub enum OpenEndedValidation {
    /// Response valid if it is greater or equal to the minimum length
    LengthMinimumChars(u32),

    /// Response valid if it is smaller or equal to the maximum length
    LengthMaximumChars(u32),

    /// Response valid if it contains the pattern
    RegexContains(String),

    /// Response valid if it does not contain the pattern
    RegexNotContains(String),

    /// Response valid if it matches the pattern
    RegexMatches(String),

    /// Response valid if it does not match the pattern
    RegexNotMatches(String),

    /// Response valid is text contains the pattern
    TextContains(String),

    /// Response valid if text does not contain the pattern
    TextNotContains(String),

    /// Response valid if text is a valid url
    TextIsUrl,

    /// Response valid if text is a valid email
    TextIsEmail,

    /// Greater than
    NumberGT(Number),

    /// Greater than or equal to
    NumberGTE(Number),

    /// Less than
    NumberLT(Number),

    /// Less than or equal to
    NumberLTE(Number),

    /// Equal to
    NumberEQ(Number),

    /// Not equal to
    NumberNEQ(Number),

    /// Between range
    NumberBT(Number, Number),

    /// Not between range
    NumberNBT(Number, Number),

    /// Is a valid number
    NumberIsNumber,

    /// Is a whole number
    NumberIsWhole
}

#[derive(Clone, Copy, Debug)]
pub enum Number {
    Integer(i32),
    Float(f32),
}

/// Represents a single selection option for
/// questions that consist of selection-type
/// responses.
#[derive(Clone, Debug)]
pub struct SingleSelectionQuestion {
    name: String,

}

#[derive(Clone, Copy, Debug)]
pub enum DateType {
    /// Day and month only
    Date,
    /// Day, month and year
    DateYear,
    /// Day and month with time
    DateTime,
    /// Day, month and year with time
    DateTimeYear

}

#[derive(Clone, Debug)]
pub struct DateQuestion {
    variant: DateType
}

#[derive(Clone, Debug)]
pub struct TimeQuestion {
    variant: TimeType
}


