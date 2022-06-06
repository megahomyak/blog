use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    rc::Rc,
};

use chrono::{DateTime, Local};

use crate::page_compilers::{compile_article, compile_index_variants, CompiledArticleInfo};

pub type FileTime = DateTime<Local>;
pub type ModificationTime = FileTime;
pub const INDEX_VARIANTS_AMOUNT: usize = 6;
pub type ArticleFileName = String;
pub type ArticleTitle = String;
struct MinimalArticleInfo {
    compiled_body: String,
    modification_time: Rc<FileTime>,
}
pub struct Articles {
    compiled_articles: HashMap<Rc<ArticleFileName>, MinimalArticleInfo>,
    articles_list: BTreeMap<Rc<ModificationTime>, HashMap<Rc<ArticleFileName>, ArticleTitle>>,
    index_variants: [String; INDEX_VARIANTS_AMOUNT],
    author_name: String,
    base_path: PathBuf,
}

impl Articles {
    fn reload_index_variants(&mut self) {
        self.index_variants = compile_index_variants(
            self.articles_list
                .values()
                .map(|articles_map| articles_map.iter()),
            &self.author_name,
        );
    }

    pub fn remove(&mut self, file_name: Rc<String>) {
        let article_info = self.compiled_articles.remove(&file_name).unwrap();
        let articles_map = self
            .articles_list
            .get(&article_info.modification_time)
            .unwrap();
        articles_map.remove(&file_name).unwrap();
        if articles_map.is_empty() {
            self.articles_list.remove(&article_info.modification_time);
        }
        self.reload_index_variants();
    }

    pub fn rename(&mut self, old_name: String, new_name: Rc<String>) {
        let article_info = self.compiled_articles.remove(&old_name).unwrap();
        self.compiled_articles
            .insert(new_name.clone(), article_info);
        let articles_map = self
            .articles_list
            .get_mut(&article_info.modification_time)
            .unwrap();
        articles_map.insert(new_name, articles_map.remove(&old_name).unwrap());
        self.reload_index_variants();
    }

    pub fn update(&mut self, file_name: String) {
        let full_path = self.base_path.join(file_name);
        let CompiledArticleInfo {
            body,
            file_name,
            modification_time,
            title,
            creation_time,
        } = compile_article(full_path, &self.author_name);
        let modification_time = Rc::new(modification_time);
        let file_name = Rc::new(file_name);
        self.compiled_articles.insert(
            file_name.clone(),
            MinimalArticleInfo {
                compiled_body: body,
                modification_time: modification_time.clone(),
            },
        );
        self.articles_list
            .entry(modification_time.clone())
            .or_insert_with(|| HashMap::new())
            .insert(file_name.clone(), title);
        self.reload_index_variants();
    }
}
