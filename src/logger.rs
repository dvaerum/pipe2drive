extern crate log;
extern crate chrono;
//extern crate term;

#[cfg(feature = "colored")]
extern crate colored;
#[cfg(feature = "colored")]
use colored::*;

use log::{Log,Level,Metadata,Record,SetLoggerError};
use chrono::Local;

struct SimpleLogger {
    level: Level,
    term: ::std::io::Stderr,
}

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        use std::io::Write;
        if self.enabled(record.metadata()) {
            let level_string = {
                #[cfg(feature = "colored")] {
                    match record.level() {
                        Level::Error => record.level().to_string().red(),
                        Level::Warn => record.level().to_string().yellow(),
                        Level::Info => record.level().to_string().cyan(),
                        Level::Debug => record.level().to_string().purple(),
                        Level::Trace => record.level().to_string().normal(),
                    }
                }
                #[cfg(not(feature = "colored"))] {
                    record.level().to_string()
                }
            };
            let target = if record.target().len() > 0 {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };
            writeln!(self.term.lock(),
                "{} {:<5} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S,%3f"),
                level_string,
                target,
                record.args())
                .expect("Something went wrong when trying to write log it stderr");
        }
    }

    fn flush(&self) {
    }
}

pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    let logger = SimpleLogger {
        level,
        term: ::std::io::stderr()
    };
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}
