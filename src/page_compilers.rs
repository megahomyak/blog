use std::{fmt::Display, fs, iter, path::PathBuf, sync::Arc};

use askama::Template;
use peeking_take_while::PeekableExt;
use pulldown_cmark::CowStr;

use crate::{
    articles::{ArticleTitle, FileTime, IndexArticleInfo, ModificationTime},
    constants::{DATE_FORMAT, PAGE_COLORS},
    file_watcher::FileNameShortcut,
};

pub struct CompiledArticleInfo {
    pub title: ArticleTitle,
    pub file_name: Arc<str>,
    pub body: String,
    pub modification_time: ModificationTime,
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    body: String,
    title: Arc<str>,
}

pub trait ExtractBaseName {
    fn base_name(&self) -> Arc<str>;
}

impl ExtractBaseName for Arc<str> {
    fn base_name(&self) -> Arc<str> {
        self.rfind('.')
            .and_then(|last_dot_index| {
                if last_dot_index == 0 {
                    None
                } else {
                    Some(self[..last_dot_index].into())
                }
            })
            .unwrap_or_else(|| self.clone())
    }
}

pub fn compile_article<AuthorName>(path: &PathBuf, author_name: AuthorName) -> CompiledArticleInfo
where
    AuthorName: Display,
{
    let file_contents = fs::read_to_string(&path).unwrap();
    let mut parser = pulldown_cmark::Parser::new_ext(&file_contents, {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        options
    })
    .peekable();
    let mut compiled_body = String::new();
    pulldown_cmark::html::push_html(
        &mut compiled_body,
        parser.by_ref().peeking_take_while(|event| {
            !matches!(
                event,
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading(..))
            )
        }),
    );
    let mut title = String::new();
    pulldown_cmark::html::push_html(
        &mut compiled_body,
        parser.by_ref().peeking_take_while(|event| {
            if let pulldown_cmark::Event::Code(contents)
            | pulldown_cmark::Event::Html(contents)
            | pulldown_cmark::Event::Text(contents) = event
            {
                title.push_str(contents);
            }
            !matches!(
                event,
                pulldown_cmark::Event::End(pulldown_cmark::Tag::Heading(..))
            )
        }),
    );
    let title: Arc<str> = title.into();
    pulldown_cmark::html::push_html(&mut compiled_body, parser);
    let file_info = fs::metadata(&path).unwrap();
    let modification_time: FileTime = file_info.modified().unwrap().into();
    {
        let signature = if let Ok(creation_time) = file_info.created() {
            let creation_time: FileTime = creation_time.into();
            let mut signature = format!(
                r#"<p align="right"><em>- {}, {}"#,
                author_name,
                creation_time.format(DATE_FORMAT)
            );
            if creation_time.date() != modification_time.date() {
                signature.push_str(&format!(
                    " (last edit at {})",
                    modification_time.format(DATE_FORMAT)
                ));
            }
            signature.push_str("</em></p>");
            signature
        } else {
            format!(
                r#"<p align="right"><em>- {}, {}</em></p>"#,
                author_name,
                modification_time.format(DATE_FORMAT)
            )
        };
        pulldown_cmark::html::push_html(
            &mut compiled_body,
            iter::once(pulldown_cmark::Event::Html(CowStr::Borrowed(&signature))),
        );
    }
    let file_name: Arc<str> = path.file_name_arc_str();
    let title = if title.is_empty() {
        ArticleTitle::FromFileName(file_name.base_name())
    } else {
        ArticleTitle::FromFirstHeading(title)
    };
    let compiled_body = ArticleTemplate {
        body: compiled_body,
        title: title.clone_contents(),
    }
    .render()
    .unwrap();
    CompiledArticleInfo {
        title,
        file_name,
        body: compiled_body,
        modification_time,
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'index_template, AuthorName>
where
    AuthorName: Display,
{
    articles_list: &'index_template [IndexArticleInfo],
    author_name: &'index_template AuthorName,
    background_color_code: &'static str,
    title_color_code: &'static str,
}

pub fn compile_index_variants<AuthorName>(
    articles_list: &[IndexArticleInfo],
    author_name: AuthorName,
) -> Vec<String>
where
    AuthorName: Display,
{
    let mut index_variants = Vec::with_capacity(PAGE_COLORS.len());
    for color in PAGE_COLORS {
        index_variants.push(
            IndexTemplate {
                articles_list,
                author_name: &author_name,
                background_color_code: color.background(),
                title_color_code: color.title(),
            }
            .render()
            .unwrap(),
        );
    }
    index_variants
}