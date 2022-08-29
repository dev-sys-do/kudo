use chrono::Utc;
use log::{LevelFilter, SetLoggerError};
use std::io::Write;

/// Configure the logger with the given verbosity level.
pub fn setup_env_logger(verbosity_level: LevelFilter) -> Result<(), SetLoggerError> {
    env_logger::builder()
        .format(move |buf, record| {
            let utc = Utc::now();

            match verbosity_level {
                // Write the file path and more time details if we are in trace mode
                LevelFilter::Trace => writeln!(
                    buf,
                    "{} - {}:{} [{}] {}",
                    utc,
                    record.file().unwrap_or(""),
                    record.line().unwrap_or(0),
                    record.level(),
                    record.args()
                ),
                _ => writeln!(
                    buf,
                    "{} [{:5}] {}",
                    utc.format("%F %T"),
                    record.level(),
                    record.args()
                ),
            }
        })
        .filter_level(verbosity_level)
        .try_init()
}
