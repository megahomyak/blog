use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use actix_web::dev::ServerHandle;
use log::{error, info};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;

use crate::{
    page_colors::PageColors, watch_articles, watch_config, website::Website, WatchContext,
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub author_name: String,
    pub index_page_colors: Vec<PageColors>,
    pub articles_directory: PathBuf,
    pub files_directory: PathBuf,
    pub date_format: String,
    pub host_name: String,
    pub port: u16,
    pub log_level: String,
    pub file_watcher_delay_in_milliseconds: u64,
}

impl Config {
    /// Returns a sample configuration, which should __not__ be used in production (because it
    /// lacks the author's name)
    pub fn sample() -> Self {
        Self {
            author_name: "<author name>".into(),
            index_page_colors: vec![
                PageColors::new("C8566B", "F6E5E8"),
                PageColors::new("E78963", "FBEDE7"),
                PageColors::new("F2D48F", "FDF8EE"),
                PageColors::new("9D75BF", "F0EAF5"),
                PageColors::new("9EC299", "F0F5EF"),
                PageColors::new("6661AB", "E8E7F2"),
            ],
            articles_directory: "articles".into(),
            files_directory: "files".into(),
            date_format: "%Y.%m.%d".into(),
            host_name: "localhost".into(),
            port: 8080,
            log_level: "info".into(),
            file_watcher_delay_in_milliseconds: 2,
        }
    }

    #[deny(warnings)] // Because unused variables will mean that I haven't invoked all the handlers
    #[allow(clippy::too_many_lines)]
    pub fn update(
        &mut self,
        new_config: Self,
        server_handle: &mut ServerHandle,
        website: &Mutex<Website>,
        articles_watch_context: &Arc<Mutex<WatchContext>>,
        config_watch_context: &Arc<Mutex<WatchContext>>,
    ) {
        macro_rules! if_changed {
            ($field_name:ident, $body:block) => {
                if self.$field_name != $field_name {
                    self.$field_name = $field_name;
                    $body
                }
            };
        }
        let Config {
            author_name,
            index_page_colors,
            articles_directory,
            files_directory,
            date_format,
            host_name,
            port,
            log_level,
            file_watcher_delay_in_milliseconds,
        } = new_config;
        let mut reload_articles = false;
        let mut reload_index = false;
        let mut reload_server = false;
        {
            let host_name_was_changed = host_name != self.host_name;
            let port_was_changed = port != self.port;
            let target = match (host_name_was_changed, port_was_changed) {
                (true, true) => "host name and port were",
                (false, false) => "",
                (false, true) => "host name was",
                (true, false) => "port was",
            };
            if !target.is_empty() {
                self.host_name = host_name;
                self.port = port;
                info!("The {} changed. Restarting", target);
                reload_server = true;
            }
        }
        {
            let log_level_filter = match &log_level.to_lowercase()[..] {
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
        };
        if_changed!(author_name, {
            reload_articles = true;
        });
        if_changed!(index_page_colors, {
            reload_index = true;
        });
        if_changed!(articles_directory, {
            match watch_articles(self) {
                Ok(new_context) => {
                    *articles_watch_context.lock().unwrap() = new_context;
                    reload_articles = true;
                    reload_index = true;
                }
                Err(error) => {
                    error!(
                        "An error occured while changing the articles directory: {}",
                        error
                    );
                }
            }
        });
        if_changed!(files_directory, {});
        if_changed!(date_format, {
            reload_articles = true;
        });
        if_changed!(file_watcher_delay_in_milliseconds, {
            match watch_articles(self) {
                Ok(new_context) => {
                    *articles_watch_context.lock().unwrap() = new_context;
                }
                Err(error) => {
                    error!(
                        "An error occured while changing the articles watcher delay: {}",
                        error
                    );
                }
            }
            match watch_config(self) {
                Ok(new_context) => {
                    *config_watch_context.lock().unwrap() = new_context;
                }
                Err(error) => {
                    error!(
                        "An error occured while changing the config watcher delay: {}",
                        error
                    );
                }
            }
        });
        if reload_server {
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(async { server_handle.stop(true).await });
        }
        if reload_articles {
            website.lock().unwrap().reload_articles();
        }
        if reload_index {
            website.lock().unwrap().reload_index_variants();
        }
    }
}
