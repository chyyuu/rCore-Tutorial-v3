/*！

本模块利用 log crate 为你提供了日志功能，使用方式见 main.rs.

*/

use crate::timer::get_time_ms;
use log::{self, Level, LevelFilter, Log, Metadata, Record};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => 31, // Red
            Level::Warn => 93,  // BrightYellow
            Level::Info => 34,  // Blue
            Level::Debug => 32, // Green
            Level::Trace => 90, // BrightBlack
        };
        let time_ms = get_time_ms();
        // Use print! (without timestamp) to avoid double timestamp
        print!(
            "\u{1B}[{}m[{:>5} ms] [{:>5}] {}\u{1B}[0m\n",
            color,
            time_ms,
            record.level(),
            record.args(),
        );
    }
    fn flush(&self) {}
}

/// Initialize logger
pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}
