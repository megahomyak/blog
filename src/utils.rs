use std::{path::Path, str::FromStr, sync::Arc};

use simple_logger::SimpleLogger;

pub trait FileNameShortcut {
    fn file_name_arc_str(&self) -> Arc<str>;
}

impl FileNameShortcut for Path {
    fn file_name_arc_str(&self) -> Arc<str> {
        self.file_name().unwrap().to_string_lossy().into()
    }
}

pub fn set_global_log_level(log_level_name: impl AsRef<str>) -> Result<(), String> {
    if let Ok(log_level_filter) = log::LevelFilter::from_str(log_level_name.as_ref()) {
        SimpleLogger::new()
            .with_level(log_level_filter)
            .init()
            .unwrap();
        Ok(())
    } else {
        Err(format!(
            "Log level not in [{}]!",
            itertools::Itertools::intersperse(
                log::LevelFilter::iter().map(|variant| variant.as_str()),
                ", "
            )
            .collect::<String>()
        ))
    }
}
