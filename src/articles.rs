use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    path::Path,
    sync::Arc,
};

use chrono::{DateTime, Local};
use rand::prelude::SliceRandom;

use crate::page_compilers::{
    compile_article, compile_index_variants, CompiledArticleInfo, ExtractBaseName,
};

pub type FileTime = DateTime<Local>;
pub type ModificationTime = FileTime;
pub type ArticleFileName = str;
struct MinimalArticleInfo {
    compiled_body: String,
    modification_time: Arc<FileTime>,
}
pub enum ArticleTitle {
    FromFileName(Arc<str>),
    FromFirstHeading(Arc<str>),
}
impl ArticleTitle {
    pub fn clone_contents(&self) -> Arc<str> {
        match self {
            Self::FromFileName(file_name) => file_name,
            Self::FromFirstHeading(first_heading) => first_heading,
        }
        .clone()
    }
}
pub struct Articles<AuthorName> {
    compiled_articles: HashMap<Arc<ArticleFileName>, MinimalArticleInfo>,
    articles_list: BTreeMap<Arc<ModificationTime>, HashMap<Arc<ArticleFileName>, ArticleTitle>>,
    index_variants: Vec<String>,
    author_name: AuthorName,
    base_path: &'static Path,
}
pub struct IndexArticleInfo {
    pub file_name: Arc<ArticleFileName>,
    pub title: Arc<str>,
}

impl<AuthorName> Articles<AuthorName>
where
    AuthorName: Display,
{
    pub fn new(author_name: AuthorName, base_path: &'static Path) -> Self {
        let article_file_names = base_path
            .read_dir()
            .expect("`articles` directory was not found!");
        let mut instance = Self {
            compiled_articles: HashMap::new(),
            articles_list: BTreeMap::new(),
            index_variants: Vec::new(),
            author_name,
            base_path,
        };
        for entry in article_file_names {
            let file_name: Arc<ArticleFileName> =
                entry.unwrap().file_name().to_str().unwrap().into();
            instance.update_without_index_reload(&file_name);
        }
        instance.reload_index_variants();
        instance
    }

    fn reload_index_variants(&mut self) {
        self.index_variants = compile_index_variants(
            &self
                .articles_list
                .values()
                .rev()
                .flat_map(|articles_map| {
                    articles_map
                        .iter()
                        .map(|(file_name, title)| IndexArticleInfo {
                            file_name: file_name.clone(),
                            title: title.clone_contents(),
                        })
                })
                .collect::<Vec<_>>(),
            &self.author_name,
        );
    }

    pub fn get_article(&self, file_name: &Arc<ArticleFileName>) -> Option<String> {
        self.compiled_articles
            .get(file_name)
            .map(|minimal_article_info| minimal_article_info.compiled_body.clone())
    }

    pub fn get_index_page(&self) -> &String {
        self.index_variants.choose(&mut rand::thread_rng()).unwrap()
    }

    pub fn remove_article(&mut self, file_name: &Arc<ArticleFileName>) {
        let article_info = self.compiled_articles.remove(file_name).unwrap();
        let articles_map = self
            .articles_list
            .get_mut(&article_info.modification_time)
            .unwrap();
        articles_map.remove(file_name).unwrap();
        if articles_map.is_empty() {
            self.articles_list.remove(&article_info.modification_time);
        }
        self.reload_index_variants();
    }

    pub fn rename_article(
        &mut self,
        old_file_name: &Arc<ArticleFileName>,
        new_file_name: Arc<ArticleFileName>,
    ) {
        let article_info = self.compiled_articles.remove(old_file_name).unwrap();
        {
            let articles_map = self
                .articles_list
                .get_mut(&article_info.modification_time)
                .unwrap();
            let mut article_title = articles_map.remove(old_file_name).unwrap();
            if matches!(article_title, ArticleTitle::FromFileName(..)) {
                article_title = ArticleTitle::FromFileName(new_file_name.base_name());
            }
            articles_map.insert(new_file_name.clone(), article_title);
        }
        self.compiled_articles.insert(new_file_name, article_info);
        self.reload_index_variants();
    }

    fn update_without_index_reload(&mut self, file_name: &Arc<ArticleFileName>) {
        {
            if let Some(article_info) = self.compiled_articles.get_mut(file_name) {
                self.articles_list
                    .get_mut(&article_info.modification_time)
                    .unwrap()
                    .remove(file_name);
            }
        }
        let full_path = self.base_path.join(&file_name[..]);
        let CompiledArticleInfo {
            body,
            file_name,
            modification_time,
            title,
        } = compile_article(&full_path, &self.author_name);
        let modification_time = Arc::new(modification_time);
        self.compiled_articles.insert(
            file_name.clone(),
            MinimalArticleInfo {
                compiled_body: body,
                modification_time: modification_time.clone(),
            },
        );
        self.articles_list
            .entry(modification_time)
            .or_insert_with(HashMap::new)
            .insert(file_name, title);
    }

    pub fn update_article(&mut self, file_name: &Arc<ArticleFileName>) {
        self.update_without_index_reload(file_name);
        self.reload_index_variants();
    }
}
