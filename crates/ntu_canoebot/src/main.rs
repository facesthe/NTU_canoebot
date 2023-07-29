mod callback;
mod command;
mod frame;

use ntu_src::SRC_CACHE;
use teloxide::prelude::*;

use crate::callback::callback_handler;
use crate::command::message_handler;

use ntu_canoebot_config as config;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", config::LOGGER_LOG_LEVEL.to_string());
    std::env::set_var("TELOXIDE_TOKEN", config::CANOEBOT_APIKEY.to_string());
    // println!("ASDASD");
    pretty_env_logger::init();
    let bot = Bot::from_env();

    tokio::task::spawn(SRC_CACHE.fill_all());

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
