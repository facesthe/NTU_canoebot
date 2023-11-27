//! Callbacks for the urban dictionary

use std::error::Error;

use async_trait::async_trait;
use ntu_canoebot_util::HiddenString;

use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::{
    dictionaries,
    frame::{common_buttons::BACK_ARROW, construct_keyboard_tuple},
};

use super::{message_from_callback_query, Callback, HandleCallback};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WhatActually {
    Get {
        query: HiddenString,
        prev: Option<HiddenString>,
    },
}

#[async_trait]
impl HandleCallback for WhatActually {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            WhatActually::Get { query, prev } => {
                whatactually_get(query, prev.to_owned(), bot, msg, true).await?;
            }
        }

        Ok(())
    }
}

pub async fn whatactually_get(
    query: &str,
    query_prev: Option<HiddenString>,
    bot: Bot,
    msg: &Message,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let res = dictionaries::urbandictonary::query(query).await;

    let resp = if let Some(result) = res {
        result
    } else {
        return Ok(());
    };

    // links to more urban dictionary pages
    let links = {
        let iter = SquareBracketIterator::from(resp.as_str());
        iter.collect::<Vec<&str>>()
    };

    // one line per button
    let mut callbacks = links
        .into_iter()
        .map(|l| {
            vec![(
                l,
                Callback::WhatActually(WhatActually::Get {
                    query: HiddenString::from(l),
                    prev: Some(HiddenString::from(query)),
                }),
            )]
        })
        .collect::<Vec<_>>();

    if let Some(p) = query_prev {
        callbacks.push(vec![(
            BACK_ARROW,
            Callback::WhatActually(WhatActually::Get {
                query: p,
                prev: None,
            }),
        )]);
    }

    let keyboard = construct_keyboard_tuple(callbacks);

    match is_callback {
        true => {
            bot.edit_message_text(msg.chat.id, msg.id, resp)
                .reply_markup(keyboard)
                .await?;
        }
        false => {
            bot.send_message(msg.chat.id, resp)
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}

/// An iterator over the string contents that reside within square brackets.
struct SquareBracketIterator<'a> {
    data: &'a str,
    idx: usize,
}

impl<'a> From<&'a str> for SquareBracketIterator<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            data: value,
            idx: 0,
        }
    }
}

impl<'a> Iterator for SquareBracketIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut char_iter = self.data[self.idx..].chars();
        let start_idx: usize;
        let end_idx: usize;

        loop {
            let c = char_iter.next()?;
            self.idx += c.len_utf8();
            if c == '[' {
                start_idx = self.idx;
                break;
            }
        }

        loop {
            let c = char_iter.next()?;
            self.idx += c.len_utf8();
            if c == ']' {
                end_idx = self.idx - 1;
                break;
            }
        }

        Some(&self.data[start_idx..end_idx].trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_bracket_iterator() {
        let sample = "the [quick] brown fox [ jumps over ] the lazy dog [ 😂 😂 😂 ]";

        let iter = SquareBracketIterator::from(sample);

        let res = iter.collect::<Vec<&str>>();
        println!("{:#?}", res);
        assert_eq!(res, vec!["quick", "jumps over", "😂 😂 😂"])
    }
}
