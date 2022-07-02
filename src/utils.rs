use std::{path::Path, sync::Arc};

use log::error;
use simple_logger::SimpleLogger;

use crate::lower_case_string::LowerCaseString;

pub trait FileNameShortcut {
    fn file_name_arc_str(&self) -> Arc<str>;
}

impl FileNameShortcut for Path {
    fn file_name_arc_str(&self) -> Arc<str> {
        self.file_name().unwrap().to_string_lossy().into()
    }
}

pub fn set_global_log_level<S: AsRef<str>>(log_level_name: &LowerCaseString<S>) {
    let log_level_filter = match log_level_name.as_ref() {
        "off" => Some(log::LevelFilter::Off),
        "error" => Some(log::LevelFilter::Error),
        "warn" => Some(log::LevelFilter::Warn),
        "info" => Some(log::LevelFilter::Info),
        "debug" => Some(log::LevelFilter::Debug),
        "trace" => Some(log::LevelFilter::Trace),
        _ => {
            error!(
                r#"Log level's lowercase representation isn't in \
                           ["off", "error", "warn", "info", "debug", "trace"]!
                           Using the old log level for now"#
            );
            None
        }
    };
    if let Some(log_level_filter) = log_level_filter {
        SimpleLogger::new()
            .with_level(log_level_filter)
            .init()
            .unwrap();
    }
}
