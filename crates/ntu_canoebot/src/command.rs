//! Command definitions and their code reside here.
//! Each command is an enum variant in [Commands].
//! If a variant contains a struct, it **must**
//! implement the [std::str::FromStr] and [HandleCommand] traits.
//!
//! The HandleCommand trait is where the "business logic"
//! for a command goes. If the response to a command is
//! to send a message, that message should be sent inside the trait method.

pub mod commands;
mod silence;

use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
use ntu_canoebot_util::{debug_println, HiddenString};
use teloxide::prelude::*;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;

use crate::callback::{self, whatactually_get, Callback};
use crate::dictionaries;
use crate::frame::common_buttons::BLANK;
use crate::frame::{calendar_month_gen, calendar_year_gen};
use crate::threadmonitor::{DynResult, THREAD_WATCH};

/// Main commands
#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Supported commands:")]
pub enum Commands {
    #[command(description = "view this help message")]
    Help(commands::Help),

    #[command(description = "start your interaction with this bot")]
    Start(commands::Start),

    #[command(description = "bot version")]
    Version,

    #[command(description = "off")]
    Calendar,

    // prefix, description, rename, parse_with, separator
    #[command(description = "off")]
    Button(commands::Button),

    // #[command(description = "give feedback")]
    #[command(description = "off")]
    Feedback,

    #[command(description = "reload boat configs")]
    Reload,

    // #[command(description = "off")]
    // Src,
    #[command(description = "see who's going training")]
    Namelist,

    #[command(description = "view training program")]
    Training,

    #[command(description = "full paddling attendance")]
    Paddling,

    #[command(description = "freshie paddling attendance")]
    FreshiePaddling,

    #[command(description = "land training")]
    Land,

    #[command(description = "freshie land training")]
    FreshieLand,

    #[command(description = "show weekly paddling statistics")]
    WeeklyBreakdown,

    #[command(description = "send SCF logsheet")]
    Logsheet,

    // secondary commands
    /// Logs the users chat info
    #[command(description = "off")]
    Ping,
    // #[command(description = "Simple wikipedia search")]
    #[command(description = "what is it?")]
    What { query: HiddenString },

    // #[command(description = "Simple urban dictionary search")]
    #[command(description = "what is it actually?")]
    WhatActually { query: HiddenString },

    #[command(description = "✨ vomit ✨")]
    EmojiVomit { text: HiddenString },

    #[command(description = "^ ω ^")]
    Uwuify { text: HiddenString },

    #[command(description = "silence, ...")]
    Silence(silence::Silence),

    #[command(description = "off")]
    Panic,
}

/// Handle a specific command.
///
/// Each command must contain a struct (unit struct or otherwise).
///
/// ```no_run
/// use std::error::Error;
/// use std::str::FromStr;
///
/// use async_trait::async_trait;
/// use teloxide::prelude::*;
/// use teloxide::types::Me;
/// use teloxide::utils::command::BotCommands;
///
/// /// Supported commands
/// #[derive(BotCommands, Clone, Debug)]
/// pub enum Command {
///     Template(TemplateData),
/// }
///
/// #[derive(Clone, Debug)]
/// // TemplateData is the inner struct of the Template enum variant.
/// pub struct TemplateData {}
///
/// impl FromStr for TemplateData {
///     type Err = String;
///
///     fn from_str(_s: &str) -> Result<Self, Self::Err> {
///         Ok(TemplateData {})
///     }
/// }
///
/// #[async_trait]
/// impl HandleCommand for TemplateData {
///     async fn handle_command(
///         &self,
///         _bot: Bot,
///         _msg: Message,
///         _me: Me,
///     ) -> Result<(), Box<dyn Error + Send + Sync>> {
///         // send a message, do some processing, etc.
///         todo!()
///     }
/// }
/// ```
#[async_trait]
trait HandleCommand {
    /// Perform an action that corresponds to its command.
    ///
    /// This trait function is asynchronous, so you will need to `await`
    /// the result for the function to execute.
    async fn handle_command(
        &self,
        bot: Bot,
        msg: Message,
        me: Me,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl HandleCommand for Commands {
    async fn handle_command(
        &self,
        bot: Bot,
        msg: Message,
        me: Me,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self {
            Commands::Help(cmd) => cmd.handle_command(bot, msg, me).await,
            Commands::Start(cmd) => cmd.handle_command(bot, msg, me).await,
            Commands::Version => {
                let ver = env!("CARGO_PKG_VERSION");
                let name = env!("CARGO_PKG_NAME");
                let resp = format!("`{} v{}`", name, ver);

                bot.send_message(msg.chat.id, resp)
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;

                Ok(())
            }

            // Commands::Src => {
            //     let (text, keyboard) = src_menu_create();

            //     bot.send_message(msg.chat.id, text)
            //         .reply_markup(keyboard)
            //         .await?;

            //     Ok(())
            // }
            Commands::Reload => {
                ntu_canoebot_attd::init().await;
                bot.send_message(msg.chat.id, "configs updated").await?;
                Ok(())
            }
            Commands::Namelist => {
                callback::namelist_get(
                    (chrono::Local::now().date_naive() + chrono::Duration::days(1)).into(),
                    false,
                    false,
                    bot,
                    &msg,
                    false,
                )
                .await
            }
            Commands::Training => {
                callback::training_get(
                    (chrono::Local::now().date_naive() + chrono::Duration::days(1)).into(),
                    false,
                    false,
                    bot,
                    &msg,
                    false,
                )
                .await
            }
            Commands::Paddling => {
                callback::paddling_get(
                    (chrono::Local::now().date_naive() + chrono::Duration::days(1)).into(),
                    false,
                    false,
                    true,
                    false,
                    u64::MAX, // & 0b1111,
                    bot,
                    &msg,
                    false,
                )
                .await
            }
            Commands::FreshiePaddling => {
                callback::paddling_get(
                    (chrono::Local::now().date_naive() + chrono::Duration::days(1)).into(),
                    false,
                    true,
                    true,
                    false,
                    u64::MAX, // & 0b1111,
                    bot,
                    &msg,
                    false,
                )
                .await
            }
            Commands::Land => {
                callback::land_get(
                    bot,
                    &msg,
                    chrono::Local::now().date_naive() + chrono::Duration::days(1),
                    false,
                    false,
                )
                .await
            }
            Commands::FreshieLand => {
                callback::land_get(
                    bot,
                    &msg,
                    chrono::Local::now().date_naive() + chrono::Duration::days(1),
                    true,
                    false,
                )
                .await
            }
            Commands::WeeklyBreakdown => {
                callback::breakdown_get(
                    chrono::Local::now().date_naive(),
                    false,
                    false,
                    bot,
                    &msg,
                    false,
                )
                .await
            }
            Commands::Logsheet => {
                callback::logsheet_start(chrono::Local::now().date_naive(), bot, &msg, false).await
            }

            Commands::Ping => callback::ping_start(bot, &msg).await,

            Commands::What { query } => {
                let res = dictionaries::wikipedia::query(query.as_str()).await;

                if let Some(result) = res {
                    bot.send_message(msg.chat.id, result).await?;
                }

                Ok(())
            }

            Commands::WhatActually { query } => {
                whatactually_get(query.as_str(), None, bot, &msg, false).await
            }

            Commands::EmojiVomit { text } => {
                let text = match text.len() {
                    // if no text is passed, look for a reply
                    0 => match msg.reply_to_message() {
                        Some(repl_msg) => {
                            if let Some(text) = repl_msg.text() {
                                if text.len() != 0 {
                                    text
                                } else {
                                    return Ok(());
                                }
                            } else {
                                return Ok(());
                            }
                        }
                        None => return Ok(()),
                    },
                    // if some text is passed, do that
                    _ => text.as_str(),
                };

                let vomit = emoji_vomit::vomit(text);
                bot.send_message(msg.chat.id, vomit).await?;

                Ok(())
            }

            Commands::Uwuify { text } => {
                let text = match text.len() {
                    // if no text is passed, look for a reply
                    0 => match msg.reply_to_message() {
                        Some(repl_msg) => {
                            if let Some(text) = repl_msg.text() {
                                if text.len() != 0 {
                                    text
                                } else {
                                    return Ok(());
                                }
                            } else {
                                return Ok(());
                            }
                        }
                        None => return Ok(()),
                    },
                    // if some text is passed, do that
                    _ => text.as_str(),
                };

                let uwu = emoji_vomit::uwuify(text);
                bot.send_message(msg.chat.id, uwu).await?;

                Ok(())
            }

            Commands::Silence(cmd) => cmd.handle_command(bot, msg, me).await,

            // test cmds
            Commands::Button(_cmd) => {
                const UNDERLINE: char = '\u{FE2D}';

                let rand = "lorem ipsum!\n*bold*?\n_italic_";
                let _rand_underline: String = rand
                    .chars()
                    .map(|c| [c, UNDERLINE])
                    .collect::<Vec<[char; 2]>>()
                    .concat()
                    .iter()
                    .collect();

                bot.send_message(msg.chat.id, format!("```\n{}```", rand))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
                // cmd.handle_command(bot, msg, me).await
                Ok(())
            }
            Commands::Feedback => Ok(()), // todo
            Commands::Calendar => {
                let keyboard = calendar_month_gen(
                    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                    &(0..31)
                        .into_iter()
                        .map(|_| Callback::Empty)
                        .collect::<Vec<Callback>>(),
                    Callback::Empty,
                    Callback::Empty,
                    Callback::Empty,
                    None,
                );

                bot.send_message(msg.chat.id, "sample calendar")
                    .reply_markup(keyboard)
                    .await?;

                let keyboard = calendar_year_gen(
                    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                    &(0..12)
                        .into_iter()
                        .map(|_| Callback::Empty)
                        .collect::<Vec<Callback>>(),
                    Callback::Empty,
                    Callback::Empty,
                    None,
                );

                bot.send_message(msg.chat.id, "sample calendar")
                    .reply_markup(keyboard)
                    .await?;

                Ok(())
            }

            Commands::Panic => Err("BIG PANIC".into()),

            // placeholder arm for unimpl'd commands
            #[allow(unreachable_patterns)]
            _ => Ok(()),
        }
    }
}

/// Main message handler
///
/// Add or remove commands and their implementations in their respective structs
pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    /// Inner async fn
    async fn inner_handler(bot: Bot, msg: Message, me: Me) -> DynResult {
        if let Some(text) = msg.text() {
            match Commands::parse(text, me.username()) {
                Ok(cmd) => {
                    log::info!("{:?}", cmd);
                    cmd.handle_command(bot, msg, me).await?;
                }

                Err(_err) => {
                    empty_command_handler(bot, msg, me).await;
                }
            }
        }

        Ok(())
    }

    let handle: tokio::task::JoinHandle<DynResult> = tokio::spawn(inner_handler(bot, msg, me));

    tokio::spawn(THREAD_WATCH.push(handle, Duration::from_secs(5)));

    Ok(())
}

/// Handler for plain text messages
async fn empty_command_handler(_bot: Bot, _msg: Message, _me: Me) {
    log::trace!(
        "doing nothing for command: {}",
        _msg.text().unwrap_or(BLANK)
    );
    log::trace!(
        "Chat id: {}, User id: {}",
        _msg.chat.id,
        _msg.from().unwrap().id
    );

    debug_println!("message contents: {:?}", _msg.text());

    // delete the unknown message sent by user
    // tokio::time::sleep(Duration::from_millis(500)).await;
    // _bot.delete_message(_msg.chat.id, _msg.id).await.unwrap();
}
