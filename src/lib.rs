#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

pub mod strategy;
pub mod api;
pub mod ws;
pub mod strategy_server;
pub mod proto;

pub fn setup_logger() -> Result<(), fern::InitError> {
    let level = match std::env::var("RUST_LOG") {
        Ok(str) => if str == "info" { log::LevelFilter::Info } else { log::LevelFilter::Debug },
        _ => log::LevelFilter::Debug,
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}:{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.line().unwrap_or(0),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
