mod callback;
mod command;
mod events;
mod frame;

use lazy_static::lazy_static;
use ntu_canoebot_util::debug_println;
use ntu_src::SRC_CACHE;
use teloxide::prelude::*;
use tokio_schedule::Job;

use crate::callback::callback_handler;
use crate::command::message_handler;
use crate::events::EXCO_CHAT_ID;

use ntu_canoebot_config as config;

lazy_static! {
    static ref BOT: Bot = {
        std::env::set_var("RUST_LOG", config::LOGGER_LOG_LEVEL.to_string());
        std::env::set_var("TELOXIDE_TOKEN", config::CANOEBOT_APIKEY.to_string());

        // this variable is set only when an override file is present (debug/deploy config).
        // we can use this to check if defaults have been overriden
        match *config::USE {
            true => (),
            false => {
                log::error!("no config file specified. Bot cannot start.");
                std::process::exit(1);
            }
        }

        Bot::from_env()
    };
}

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .parse_filters(*config::LOGGER_LOG_LEVEL)
        .default_format()
        .init();

    tokio::task::spawn(start_events());

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    log::info!("startup");
    Dispatcher::builder(BOT.clone(), handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

/// Periodic tasks / init tasks go here
async fn start_events() {
    const REFRESH_INTERVAL: u32 = 10;

    ntu_canoebot_attd::init().await;

    tokio::task::spawn(SRC_CACHE.fill_all());
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

    debug_println!("chat_id: {:?}", *EXCO_CHAT_ID);
    if *config::EVENTS_DAILY_LOGSHEET_PROMPT_ENABLE {
        let prompt_time = config::EVENTS_DAILY_LOGSHEET_PROMPT_TIME.time.unwrap();
        let logsheet_task = tokio_schedule::every(1)
            .day()
            .at(
                prompt_time.hour as u32,
                prompt_time.minute as u32,
                prompt_time.second as u32,
            )
            .perform(|| async {
                events::logsheet_prompt(BOT.clone())
                    .await
                    .expect("logsheet prompt failed")
            });

        tokio::task::spawn(logsheet_task);
    }

    if *config::EVENTS_DAILY_ATTENDANCE_REMINDER_ENABLE {
        let prompt_time = config::EVENTS_DAILY_ATTENDANCE_REMINDER_TIME.time.unwrap();
        let attendance_event = tokio_schedule::every(1)
            .day()
            .at(
                prompt_time.hour as u32,
                prompt_time.minute as u32,
                prompt_time.second as u32,
            )
            .perform(|| async {
                events::attendance_prompt(BOT.clone())
                    .await
                    .expect("attendance prompt failed")
            });

        tokio::task::spawn(attendance_event);
    }

    if *config::EVENTS_WEEKLY_BREAKDOWN_ENABLE {
        let prompt_time = config::EVENTS_WEEKLY_BREAKDOWN_TIME.time.unwrap();
        let breakdown_event = tokio_schedule::every(1)
            .week()
            .on(chrono::Weekday::Wed)
            .at(
                prompt_time.hour as u32,
                prompt_time.minute as u32,
                prompt_time.second as u32,
            )
            .perform(|| async {
                events::breakdown_prompt(BOT.clone())
                    .await
                    .expect("breakdown prompt failed")
            });

        tokio::task::spawn(breakdown_event);
    }
}
