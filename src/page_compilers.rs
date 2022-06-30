use std::{fmt::Display, fs, io, iter, path::PathBuf, sync::Arc};

use askama::Template;
use peeking_take_while::PeekableExt;
use pulldown_cmark::CowStr;

use crate::{
    config::Config,
    utils::FileNameShortcut,
    website::{ArticleTitle, FileTime, IndexArticleInfo, ModificationTime},
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

pub fn compile_article(path: &PathBuf, config: &Config) -> io::Result<CompiledArticleInfo> {
    let file_contents = fs::read_to_string(&path)?;
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
                html_escape::encode_text(&config.author_name),
                creation_time.format(&config.date_format)
            );
            if creation_time.date() != modification_time.date() {
                signature.push_str(&format!(
                    " (last edit at {})",
                    modification_time.format(&config.date_format)
                ));
            }
            signature.push_str("</em></p>");
            signature
        } else {
            format!(
                r#"<p align="right"><em>- {}, {}</em></p>"#,
                html_escape::encode_text(&config.author_name),
                modification_time.format(&config.date_format)
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
    Ok(CompiledArticleInfo {
        title,
        file_name,
        body: compiled_body,
        modification_time,
    })
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'index_template, AuthorName>
where
    AuthorName: Display,
{
    articles_list: &'index_template [IndexArticleInfo],
    author_name: &'index_template AuthorName,
    background_color_code: &'index_template str,
    title_color_code: &'index_template str,
}

pub fn compile_index_variants(articles_list: &[IndexArticleInfo], config: &Config) -> Vec<String> {
    let mut index_variants = Vec::with_capacity(config.index_page_colors.len());
    for color in &config.index_page_colors[..] {
        index_variants.push(
            IndexTemplate {
                articles_list,
                author_name: &config.author_name,
                background_color_code: color.background(),
                title_color_code: color.title(),
            }
            .render()
            .unwrap(),
        );
    }
    index_variants
}
