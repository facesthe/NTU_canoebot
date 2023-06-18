//! This module contains various `frames`, or inline keyboard layouts
//! that will be used for the bot.
//!
//! Also contains constants for the names and text messages that accompany each
//! frame.

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::callback::booking::{self, Booking};
use crate::callback::menu::Menu;
use crate::callback::Callback;

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

/// The main menu
pub fn main_menu() -> InlineKeyboardMarkup {
    let button_names = [["booking"], ["status"]];

    let callback_data = [
        [Callback::Menu(Menu::Booking(Booking::Here))],
        [Callback::Menu(Menu::Status)],
    ];

    construct_keyboard(button_names, callback_data)
}

/// The booking menu
/// Function params still WIP
pub fn booking_menu(_uuid: u128) -> InlineKeyboardMarkup {
    let callback_names = [vec!["BTP", "BTE"], vec![common_buttons::BACK]];

    let callback_data = [
        vec![
            Callback::Menu(Menu::Booking(Booking::BTP(booking::BTP {}))),
            Callback::Menu(Menu::Booking(Booking::BTE(booking::BTE {}))),
        ],
        vec![Callback::Menu(Menu::Here)],
    ];

    construct_keyboard(callback_names, callback_data)
}

/// Commonly used button names throughout this crate
pub mod common_buttons {
    pub const BACK: &str = "<< back";
}

/// Commonly used inline keyboard descriptions for each frame
pub mod common_descriptions {
    pub const MENU: &str = "Choose from the options below:";
    pub const BOOKING: &str = "Choose a booking option below:";
}
