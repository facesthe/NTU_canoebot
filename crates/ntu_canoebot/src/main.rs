mod callback;
mod command;
mod frame;

use teloxide::prelude::*;

use crate::callback::callback_handler;
use crate::command::message_handler;

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env_gen").ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
