mod callback;
mod command;
mod frame;

use ntu_src::SRC_CACHE;
use teloxide::prelude::*;
use tokio_schedule::Job;

use crate::callback::callback_handler;
use crate::command::message_handler;

use ntu_canoebot_config as config;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", config::LOGGER_LOG_LEVEL.to_string());
    std::env::set_var("TELOXIDE_TOKEN", config::CANOEBOT_APIKEY.to_string());

    pretty_env_logger::init();
    let bot = Bot::from_env();

    tokio::task::spawn(start_events());

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

/// Periodic tasks / init tasks go here
async fn start_events() {
    const REFRESH_INTERVAL: u32 = 10;

    tokio::task::spawn(SRC_CACHE.fill_all());
    tokio::task::spawn(ntu_canoebot_attd::refresh_sheet_cache(true));

    let cache_refresh = tokio_schedule::every(REFRESH_INTERVAL)
        .minutes()
        .perform(|| async { SRC_CACHE.refresh_all().await });
    tokio::task::spawn(cache_refresh);

    let attendance_cache_refresh =
        tokio_schedule::every(REFRESH_INTERVAL)
            .minutes()
            .perform(|| async {
                ntu_canoebot_attd::refresh_sheet_cache(false)
                    .await
                    .expect("attendance sheet refresh failed");
            });
    tokio::task::spawn(attendance_cache_refresh);
}
