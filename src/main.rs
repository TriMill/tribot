//#![feature(async_closure)]
use log::{info, error};
use std::env;
use serenity::{
    prelude::*
};
mod commands;
mod event;
mod state;
mod utils;
use crate::event::*;
use crate::state::*;

#[tokio::main]
async fn main() {
    init_logger().expect("Could not initlialize logger");
    info!("Starting bot...");
    let token = env::var("DISCORD_BOT_TOKEN").expect("No token found in environment");
    let state_filename = env::var("DISCORD_STATE_FILE").expect("No state filename found in environment");
    let state = State::load_from_file(&state_filename).unwrap();
    let mut client = Client::builder(&token)
        .event_handler(Handler).await
        .expect("Client creation failed");
    let shmgr1 = client.shard_manager.clone();
    let shmgr2 = client.shard_manager.clone();
    {
        let mut data = client.data.write().await;
        data.insert::<State>(state);
        data.insert::<ShardManagerKey>(shmgr1);
    }
    tokio::task::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Error setting Ctrl+C handler: {:?}", e);
            return
        }
        info!("Got Ctrl+C, exiting");
        shmgr2.lock().await.shutdown_all().await;
        std::process::exit(0);
    });
    if let Err(e) = client.start().await {
        error!("Error starting: {:?}", e);
    }
}

fn init_logger() -> Result<(), fern::InitError> {
    let level_env_var = match env::args().nth(1) {
        Some(x) => Some(x.to_lowercase()),
        None => None
    };
    let is_default = level_env_var.is_none();
    let level = match level_env_var {
        Some(x) => match &x[..] {
            "off" => log::LevelFilter::Off,
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => {
                println!("[LOGGER ERROR] Invalid log filter, assuming Info");
                log::LevelFilter::Info
            }
        }
        None => {
            log::LevelFilter::Info
        }
    };
    let default_level = match is_default {
        true => log::LevelFilter::Warn,
        false => level
    };
    fn color_for(level: log::Level) -> &'static str {
        match level {
            log::Level::Trace => "\x1b[0;90m",
            log::Level::Debug => "\x1b[0;37m",
            log::Level::Info => "\x1b[0;97m",
            log::Level::Warn => "\x1b[0;33m",
            log::Level::Error => "\x1b[0;31m",
        }
    }
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}{}[{}][{}] {}",
                color_for(record.level()),
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .level_for("serenity", default_level)
        .level_for("tracing", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
