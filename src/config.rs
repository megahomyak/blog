use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    absolute_path::AbsolutePath, page_colors::PageColors, utils::set_global_log_level,
    watch_articles, watch_config, website::Website, ArticlesWatcher, ConfigWatcher,
    CustomServerHandle, WatchContext,
};

#[derive(Deserialize, Serialize)]
pub struct Base<ArticlesDirectoryPath> {
    pub author_name: String,
    pub index_page_colors: Vec<PageColors>,
    pub articles_directory: ArticlesDirectoryPath,
    pub files_directory: PathBuf,
    pub date_format: String,
    pub host_name: String,
    pub port: u16,
    pub log_level: String,
    pub file_watcher_delay_in_milliseconds: u64,
}

impl Base<PathBuf> {
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
            file_watcher_delay_in_milliseconds: 2000,
        }
    }

    /// Upgrades itself to [`BaseConfig<AbsolutePath<PathBuf>>`],
    /// returning an error if `articles_directory.canonicalize()` failed.
    pub fn upgrade(mut self) -> Result<Config, (io::Error, Self)> {
        let articles_directory = {
            match AbsolutePath::new(self.articles_directory) {
                Ok(articles_directory) => articles_directory,
                Err(old_articles_directory) => match old_articles_directory.canonicalize() {
                    Ok(old_articles_directory) => {
                        AbsolutePath::new(old_articles_directory).unwrap()
                    }
                    Err(error) => {
                        self.articles_directory = old_articles_directory;
                        return Err((error, self));
                    }
                },
            }
        };
        Ok(Config {
            articles_directory,
            port: self.port,
            host_name: self.host_name,
            log_level: self.log_level,
            author_name: self.author_name,
            date_format: self.date_format,
            files_directory: self.files_directory,
            index_page_colors: self.index_page_colors,
            file_watcher_delay_in_milliseconds: self.file_watcher_delay_in_milliseconds,
        })
    }
}

/// Working config that should be used in the server's code
pub type Config = Base<AbsolutePath<PathBuf>>;

impl Config {
    #[deny(unused_variables)] // Unused variables will mean that I haven't handled everything
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cognitive_complexity)]
    pub fn update(
        old_config: &Mutex<Self>,
        new_config: Base<PathBuf>,
        server_handle: &CustomServerHandle,
        website: &Mutex<Website>,
        articles_watch_context: &Arc<Mutex<WatchContext<ArticlesWatcher>>>,
        config_watch_context: &Arc<Mutex<WatchContext<ConfigWatcher>>>,
    ) {
        let mut reload_articles = false;
        let mut reload_index = false;
        let mut reload_server = false;
        {
            let mut old_config = old_config.lock().unwrap();
            macro_rules! if_changed {
                ($field_name:ident, $body:block) => {
                    if old_config.$field_name != $field_name {
                        $body
                        old_config.$field_name = $field_name;
                    }
                };
            }
            let Base {
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
            {
                let host_name_was_changed = host_name != old_config.host_name;
                let port_was_changed = port != old_config.port;
                let target = match (host_name_was_changed, port_was_changed) {
                    (true, true) => "host name and port were",
                    (false, false) => "",
                    (false, true) => "host name was",
                    (true, false) => "port was",
                };
                if !target.is_empty() {
                    old_config.host_name = host_name;
                    old_config.port = port;
                    info!("{} changed. Restarting", target);
                    reload_server = true;
                }
            }
            if_changed!(log_level, {
                set_global_log_level(&log_level).unwrap_or_else(|error| error!("{}", error));
            });
            if_changed!(author_name, {
                reload_articles = true;
            });
            if_changed!(index_page_colors, {
                reload_index = true;
            });
            {
                if articles_directory != old_config.articles_directory.as_ref() {
                    let articles_directory = match AbsolutePath::new(articles_directory) {
                        Ok(articles_directory) => Some(articles_directory),
                        Err(articles_directory) => match articles_directory.canonicalize() {
                            Ok(articles_directory) => {
                                Some(AbsolutePath::new(articles_directory).unwrap())
                            }
                            Err(error) => {
                                error!(
                                    "An error occured while getting the full path to the articles \
                                    directory (which was changed in the config): {}",
                                    error
                                );
                                None
                            }
                        },
                    };
                    if let Some(articles_directory) = articles_directory {
                        if articles_directory.as_ref() == old_config.articles_directory.as_ref() {
                            match watch_articles(&old_config) {
                                Ok(new_context) => {
                                    *articles_watch_context.lock().unwrap() = new_context;
                                    reload_articles = true;
                                    reload_index = true;
                                    old_config.articles_directory = articles_directory;
                                }
                                Err(error) => {
                                    error!(
                                        "An error occured while changing the articles \
                                        directory: {}",
                                        error
                                    );
                                }
                            }
                        }
                    }
                }
            }
            if_changed!(files_directory, {});
            if_changed!(date_format, {
                reload_articles = true;
            });
            if_changed!(file_watcher_delay_in_milliseconds, {
                match watch_articles(&old_config) {
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
                match watch_config(&old_config) {
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
        }
        if reload_server {
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(async { server_handle.request_restart().await });
        }
        if reload_articles {
            website.lock().unwrap().reload_articles();
        }
        if reload_index {
            website.lock().unwrap().reload_index_variants();
        }
    }
}
