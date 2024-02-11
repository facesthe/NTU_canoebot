//! This module contains various `frames`, or inline keyboard layouts
//! that will be used for the bot.
//!
//! Also contains constants for the names and text messages that accompany each
//! frame.
#![allow(unused)]

use chrono::{Datelike, Duration, NaiveDate};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::callback::Callback;
use crate::callback::Date;
use crate::frame::common_buttons::{BACK, DATE, TIME, WEEKDAYS};

use self::common_buttons::TIME_AM;
use self::common_buttons::TIME_PM;
use self::common_buttons::{BACK_ARROW, BLANK, FORWARD_ARROW, MONTHS, REFRESH, UNDERLINE};

/// Construct a keyboard from two 2D arrays/vec consisting of the callback
/// button name and the callback data.
///
/// The arrays should have the same shape, or else the smaller of the two will
/// be taken.
///
/// Vecs must be used when not all rows have the same number of elements.
pub fn construct_keyboard<Names2D, Data2D, Name>(
    names: Names2D,
    data: Data2D,
) -> InlineKeyboardMarkup
where
    Names2D: IntoIterator,
    Data2D: IntoIterator,

    Names2D::Item: IntoIterator<Item = Name>,
    Name: ToString,

    Data2D::Item: IntoIterator<Item = Callback>,
{
    let mut buttons_vec: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for (name_row, data_row) in names.into_iter().zip(data.into_iter()) {
        buttons_vec.push(
            name_row
                .into_iter()
                .zip(data_row.into_iter())
                .map(|(name, data)| InlineKeyboardButton::callback(name.to_string(), data))
                .collect::<Vec<InlineKeyboardButton>>(),
        );
    }

    InlineKeyboardMarkup::new(buttons_vec)
}

/// Construct the keyboard from a matrix of tuples
pub fn construct_keyboard_tuple<Buttons2D, Name>(buttons: Buttons2D) -> InlineKeyboardMarkup
where
    Buttons2D: IntoIterator,
    Buttons2D::Item: IntoIterator<Item = (Name, Callback)>,

    Name: ToString,
{
    let mut buttons_vec: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for row in buttons.into_iter() {
        buttons_vec.push(
            row.into_iter()
                .map(|(name, data)| InlineKeyboardButton::callback(name.to_string(), data))
                .collect::<Vec<InlineKeyboardButton>>(),
        )
    }

    InlineKeyboardMarkup::new(buttons_vec)
}

/// Turns a slice of somethinig into a vector of vectors.
/// The number of elements in each vector is controlled by the arg provided.
/// The last element may contain less than the specified number of elements, if the
/// source vector does not have an integer multiple of cols.
pub fn convert_to_2d<T: Clone>(input: &[T], cols: usize) -> Vec<Vec<T>> {
    input.chunks(cols).map(|c| c.to_vec()).collect::<Vec<_>>()
}

/// Generate a month calendar with navigation buttons.
///
/// Pass in a reference to a callback data slice
/// ordered for each day of the month + navigation buttons.
///
/// Optionally, pass in callback data for a back button.
pub fn calendar_month_gen(
    date: NaiveDate,
    data: &[Callback],
    year: Callback,
    next: Callback,
    prev: Callback,
    back: Option<Callback>,
) -> InlineKeyboardMarkup {
    let num_days_in_month = days_in_month(date.year(), date.month());

    let mut buttons = (1..=num_days_in_month)
        .into_iter()
        .zip(data)
        .map(|(name, data)| (name.to_string(), data.to_owned()))
        .collect::<Vec<(String, Callback)>>();

    let today = chrono::Local::now().date_naive();

    // underline the current day
    if today.year() == date.year() && today.month() == date.month() {
        let day = buttons.get_mut(today.day() as usize - 1).unwrap();
        let old = &day.0;
        let new = old
            .chars()
            .map(|c| [c, UNDERLINE].iter().collect::<String>())
            .collect::<Vec<String>>()
            .concat();

        day.0 = new;
    }

    let (pre, post) = month_padding(date.year(), date.month());

    let pre = vec![(BLANK.to_string(), Callback::Empty); pre as usize];
    let post = vec![(BLANK.to_string(), Callback::Empty); post as usize];
    let days = WEEKDAYS
        .iter()
        .map(|day| (day.to_string(), Callback::Empty))
        .collect::<Vec<(String, Callback)>>();

    let buttons = [days, pre, buttons, post].concat();
    // let (names, cdata): (Vec<String>, Vec<Callback>) = buttons.into_iter().unzip();

    let mut folded_buttons = convert_to_2d(&buttons, 7);

    // add header (navi buttons + month name) and footer (back button if is Some())
    let header = vec![
        (BACK_ARROW.to_string(), prev),
        (date.format("%b %Y").to_string(), year),
        (FORWARD_ARROW.to_string(), next),
    ];

    folded_buttons.insert(0, header);

    if let Some(data) = back {
        folded_buttons.push(vec![("back".to_string(), data.to_owned())])
    }

    construct_keyboard_tuple(folded_buttons)
}

/// Generate a year calendar with navigation buttons.
///
/// Pass in a reference to a callback data slice
/// ordered for each month of the year + navigation buttons.
///
/// Optionally, pass in callback data for a back button.
pub fn calendar_year_gen(
    date: NaiveDate,
    data: &[Callback],
    next: Callback,
    prev: Callback,
    back: Option<Callback>,
) -> InlineKeyboardMarkup {
    let buttons = MONTHS
        .into_iter()
        .zip(data.iter())
        .map(|(month, data)| (month.to_string(), data.to_owned()))
        .collect::<Vec<(String, Callback)>>();

    let header = vec![
        (BACK_ARROW.to_string(), prev),
        (date.format("%Y").to_string(), Callback::Empty),
        (FORWARD_ARROW.to_string(), next),
    ];

    let mut buttons = convert_to_2d(&buttons, 3);
    buttons.insert(0, header);

    if let Some(data) = back {
        buttons.push(vec![("back".to_string(), data.to_owned())])
    }

    construct_keyboard_tuple(buttons)
}

/// Calculate the number of days in a month
fn days_in_month(year: i32, month: u32) -> u32 {
    let curr = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let next = {
        let next_year;
        let next_month;

        if month == 12 {
            next_month = 1;
            next_year = year + 1;
        } else {
            next_year = year;
            next_month = month + 1;
        }

        NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap()
    };

    (next - curr).num_days() as u32
}

/// Returns the padding days for the start and end of each month.
///
/// For example, for Jan 1970, the first day (01) is on a Thursday,
/// and the last day (31) on a Saturday.
///
/// Following ISO 8601's spec that Monday is the first day of the week,
/// there would be 3 days preceding the start of the month (Monday - Wednesday),
/// and 1 day following the end of the month (Sunday) to end on Sunday.
///
/// ```no_run
/// assert_eq!(month_padding(1970, 1), (3, 1));
/// ```
fn month_padding(year: i32, month: u32) -> (u32, u32) {
    let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(year, month, days_in_month(year, month)).unwrap();

    let start_day = start.weekday();
    let end_day = end.weekday();

    let padding_start = start_day.num_days_from_monday();
    let padding_end = 7 - end_day.num_days_from_sunday();

    (padding_start, padding_end % 7)
}

/// Common navigation buttons for namelist, training, etc
///
/// Shows (in row order):
/// - previous, refresh, next
/// - time, calendar
///
pub fn date_am_pm_navigation(
    date: NaiveDate,
    refresh: Callback,
    next: Callback,
    prev: Callback,
    time_slot: Callback,
    calendar: Callback,
    time_slot_bool: bool,
) -> InlineKeyboardMarkup {
    let navi_row = vec![
        (BACK_ARROW, prev),
        (REFRESH, refresh),
        (FORWARD_ARROW, next),
    ];

    let other_row = vec![
        (
            {
                if time_slot_bool {
                    TIME_PM
                } else {
                    TIME_AM
                }
            },
            time_slot,
        ),
        (DATE, calendar),
    ];

    construct_keyboard_tuple([navi_row, other_row])
}

/// Commonly used button names throughout this crate
pub mod common_buttons {
    pub const BACK: &str = "back";
    pub const BACK_ARROW: &str = "<<";
    pub const FORWARD_ARROW: &str = ">>";
    pub const REFRESH: &str = "⟳";
    pub const BLANK: &str = " ";

    pub const TIME: &str = "time";
    pub const DATE: &str = "date";

    pub const TIME_AM: &str = "AM";
    pub const TIME_PM: &str = "PM";

    pub const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    pub const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    /// Unicode combining character
    pub const UNDERLINE: char = '\u{FE2D}';
}

/// Commonly used inline keyboard descriptions for each frame
pub mod common_descriptions {
    pub const MENU: &str = "Choose from the options below:";
    pub const BOOKING: &str = "Choose a booking option below:";
    pub const CALENDAR: &str = "Choose a date:";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2023, 7), 31);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 12), 31);
    }

    #[test]
    fn test_month_padding() {
        assert_eq!(month_padding(1970, 1), (3, 1));
        assert_eq!(month_padding(1980, 1), (1, 3));
        assert_eq!(month_padding(1990, 1), (0, 4));
        assert_eq!(month_padding(2023, 12), (4, 0))
    }

    #[test]
    fn test_something() {
        let x = chrono::Month::try_from(1).unwrap();
        let y = NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .format("%b")
            .to_string();

        println!("{:?}", x);
        println!("{:?}", y)
    }
}
