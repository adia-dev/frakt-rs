use chrono::Local;
use colored::Colorize;
use env_logger::{Builder, Env};
use log::Level;
use std::io::Write;

pub fn init() {
    init_with_level("info");
}

pub fn init_with_level(level_filter: &str) {
    Builder::new()
        .format(|buf, record| {
            let timestamp = Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .bright_black();
            let level = record.level();
            let level_colored = match level {
                Level::Error => level.to_string().red(),
                Level::Warn => level.to_string().yellow(),
                Level::Info => level.to_string().cyan(),
                Level::Debug => level.to_string().purple(),
                Level::Trace => level.to_string().magenta(),
            };
            writeln!(buf, "{} {} {}", timestamp, level_colored, record.args())
        })
        // .filter(None, LevelFilter::Info)
        .parse_env(Env::default().default_filter_or(level_filter))
        .init();
}
