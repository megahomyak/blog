use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::helpers::PageColors;

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
        }
    }
}
