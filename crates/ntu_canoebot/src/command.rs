pub mod commands;

use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;

/// Main commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands:")]
pub enum Commands {
    #[command(description = "This is the help command")]
    Help(commands::Help),

    #[command(description = "Start your interaction with this bot")]
    // #[command(hide)]
    Start(commands::Start),

    // prefix, description, rename, parse_with, separator
    #[command(description = "off")]
    Button(commands::Button),

    #[command(description = "Main menu (testing)")]
    Menu(commands::Menu),

    #[command(description = "give feedback")]
    Feedback,
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
            Commands::Button(cmd) => cmd.handle_command(bot, msg, me).await,
            Commands::Menu(cmd) => cmd.handle_command(bot, msg, me).await,
            Commands::Feedback => Ok(()), // todo
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
    if let Some(text) = msg.text() {
        match Commands::parse(text, me.username()) {
            Ok(_cmd) => _cmd.handle_command(bot, msg, me).await?,

            Err(_err) => {
                empty_command_handler(bot, msg, me).await;
            }
        }
    }

    Ok(())
}

/// Handler for plain text messages
async fn empty_command_handler(_bot: Bot, _msg: Message, _me: Me) {
    log::trace!("doing nothing");
    log::trace!(
        "Chat id: {}, User id: {}",
        _msg.chat.id,
        _msg.from().unwrap().id
    );

    tokio::time::sleep(Duration::from_millis(500)).await;

    _bot.delete_message(_msg.chat.id, _msg.id).await.unwrap();
    // do nothing for now
}
