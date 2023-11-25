use std::fs::{self, OpenOptions};

use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

const LOG_FOLDER: &str = "logs/";
pub fn create_logger() {
    let _ = fs::create_dir(LOG_FOLDER);
    let path = format!("{}/last_run.rpglog", LOG_FOLDER);
    if let Ok(file) = OpenOptions::new().create(true).append(true).open(path) {
        let file_logger = WriteLogger::new(log::LevelFilter::Info, Config::default(), file);
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            file_logger,
        ])
        .unwrap();
        info!("Logger initialized");
    } else {
        println!("Logging initialization failure");
    }
}
