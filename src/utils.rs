use std::{path::Path, sync::Arc};

use simple_logger::SimpleLogger;

pub trait FileNameShortcut {
    fn file_name_arc_str(&self) -> Arc<str>;
}

impl FileNameShortcut for Path {
    fn file_name_arc_str(&self) -> Arc<str> {
        self.file_name().unwrap().to_string_lossy().into()
    }
}

pub fn set_global_log_level(log_level_name: impl AsRef<str>) -> Result<(), &'static str> {
    let log_level_filter = match &log_level_name.as_ref().to_lowercase()[..] {
        "off" => log::LevelFilter::Off,
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => {
            return Err(
                r#"Log level's lowercase representation isn't in \
                ["off", "error", "warn", "info", "debug", "trace"]!"#
            )
        }
    };
    SimpleLogger::new()
        .with_level(log_level_filter)
        .init()
        .unwrap();
    Ok(())
}
