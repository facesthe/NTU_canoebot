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

    // this variable is set only when an override file is present (debug/deploy config).
    // we can use this to check if defaults have been overriden
    match *config::USE {
        true => (),
        false => {
            log::error!("no config file specified. Bot cannot start.");
            std::process::exit(1);
        }
    }

    let bot = Bot::from_env();

    tokio::task::spawn(start_events());

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    log::info!("startup");
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
    tokio::task::spawn(ntu_canoebot_attd::init());
    tokio::task::spawn(ntu_canoebot_attd::refresh_attd_sheet_cache(true));
    tokio::task::spawn(ntu_canoebot_attd::refresh_prog_sheet_cache(true));

    let cache_refresh = tokio_schedule::every(*config::SRC_CACHE_REFRESH as u32)
        .minutes()
        .perform(|| async { SRC_CACHE.refresh_all().await });
    tokio::task::spawn(cache_refresh);

    let attd_cache_refresh = tokio_schedule::every(REFRESH_INTERVAL)
        .minutes()
        .perform(|| async {
            ntu_canoebot_attd::refresh_attd_sheet_cache(false)
                .await
                .expect("attd cache refresh failed");
        });
    tokio::task::spawn(attd_cache_refresh);

    let prog_cache_refresh = tokio_schedule::every(REFRESH_INTERVAL)
        .minute()
        .perform(|| async {
            ntu_canoebot_attd::refresh_prog_sheet_cache(false)
                .await
                .expect("prog cache refresh failed")
        });
    tokio::task::spawn(prog_cache_refresh);
}
