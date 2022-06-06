use std::{collections::{HashMap, BTreeMap}, time::SystemTime};

pub struct ShortArticleInfo {
    file_name: String,
    title: String,
}
type ModificationTime = SystemTime;
pub struct Articles {
    compiled_articles: HashMap<String, String>,
    articles_list: BTreeMap<ModificationTime, ShortArticleInfo>,
    compiled_index: String,
}
