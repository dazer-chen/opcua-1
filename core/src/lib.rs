#[macro_use]
extern crate log;
extern crate chrono;
extern crate regex;
extern crate rand;
extern crate openssl;
#[cfg(test)]
extern crate tempdir;

extern crate opcua_types;

pub mod comms;
pub mod crypto;

/// The prelude mod contains all the things you typically need to access from a client / server.
pub mod prelude {
    pub use opcua_types::*;
    pub use comms::*;
    pub use crypto::*;
}

use std::collections::HashSet;

/// Simple logger (as the name suggests) is a bare bones implementation of the log::Log trait
/// that may be used to print debug information out to the console.
struct SimpleLogger {
    target_map: HashSet<&'static str>,
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        metadata.level() <= log::LogLevel::Debug
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            if !self.target_map.is_empty() && !self.target_map.contains(record.metadata().target()) {
                return;
            }

            use chrono;
            let now = chrono::UTC::now();
            let time_fmt = now.format("%Y-%m-%d %H:%M:%S%.3f");

            match record.metadata().level() {
                log::LogLevel::Error => {
                    println!("{} - \x1b[37m\x1b[41m{}\x1b[0m - {} - {}", time_fmt, record.level(), record.metadata().target(), record.args());
                }
                log::LogLevel::Warn => {
                    println!("{} - \x1b[33m{}\x1b[0m - {} - {}", time_fmt, record.level(), record.metadata().target(), record.args());
                }
                log::LogLevel::Info => {
                    println!("{} - \x1b[36m{}\x1b[0m - {} - {}", time_fmt, record.level(), record.metadata().target(), record.args());
                }
                _ => {
                    println!("{} - {} - {} - {}", time_fmt, record.level(), record.metadata().target(), record.args());
                }
            }
        }
    }
}

/// This is directly analogous to the enum of the same name in the log crate,
// but we expose ours to remove a dependency for client code to worry about.
pub enum LogLevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace
}

/// Initialise OPC UA logging on the executable.
pub fn init_logging(logging_level: LogLevelFilter) {
    let _ = log::set_logger(|max_log_level| {
        let logging_level = match logging_level {
            LogLevelFilter::Off => log::LogLevelFilter::Off,
            LogLevelFilter::Error => log::LogLevelFilter::Error,
            LogLevelFilter::Warn => log::LogLevelFilter::Warn,
            LogLevelFilter::Info => log::LogLevelFilter::Info,
            LogLevelFilter::Debug => log::LogLevelFilter::Debug,
            LogLevelFilter::Trace => log::LogLevelFilter::Trace,
        };

        max_log_level.set(logging_level);
        Box::new(SimpleLogger {
            target_map: HashSet::new(),
        })
    });
}

/// Contains debugging utility helper functions
pub mod debug {
    pub const SUBSCRIPTION: &'static str = "subscription";

    /// Prints out the content of a slice in hex and visible char format to aid debugging. Format
    /// is similar to corresponding functionality in node-opcua
    pub fn debug_buffer(message: &str, buf: &[u8]) {
        use log;
        // No point doing anything unless debug level is on
        if !log_enabled!(log::LogLevel::Debug) {
            return;
        }

        let line_len = 32;
        let len = buf.len();
        let last_line_padding = ((len / line_len) + 1) * line_len - len;

        debug!("{}", message);

        let mut char_line = String::new();
        let mut hex_line = format!("{:08x}: ", 0);

        for (i, b) in buf.iter().enumerate() {
            let value = *b as u8;
            if i > 0 && i % line_len == 0 {
                debug!(target: "hex", "{} {}", hex_line, char_line);
                hex_line = format!("{:08}: ", i);
                char_line.clear();
            }
            hex_line = format!("{} {:02x}", hex_line, value);
            char_line.push(if value >= 32 && value <= 126 { value as char } else { '.' });
        }
        if last_line_padding > 0 {
            for _ in 0..last_line_padding {
                hex_line.push_str("   ");
            }
            debug!(target: "hex", "{} {}", hex_line, char_line);
        }
    }
}

#[cfg(test)]
mod tests;
