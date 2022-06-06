use std::{fmt::Display, fs, iter, path::PathBuf, rc::Rc, marker::PhantomData};

use askama::Template;
use pulldown_cmark::CowStr;

use crate::{
    articles::{FileTime, INDEX_VARIANTS_AMOUNT, ArticleTitle, ArticleFileName},
    file_watcher::WithFileName,
};

pub struct CompiledArticleInfo {
    pub title: String,
    pub file_name: String,
    pub body: String,
    pub creation_time: FileTime,
    pub modification_time: FileTime,
}

pub fn compile_article<AuthorName>(path: PathBuf, author_name: AuthorName) -> CompiledArticleInfo
where
    AuthorName: Display,
{
    let parser = pulldown_cmark::Parser::new_ext(
        {
            let file_contents = fs::read_to_string(path).unwrap();
            &file_contents
        },
        {
            let mut options = pulldown_cmark::Options::empty();
            options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
            options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
            options
        },
    );
    let mut compiled_body = String::new();
    pulldown_cmark::html::push_html(
        &mut compiled_body,
        parser.map_while(|event| match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading(..)) => {
                pulldown_cmark::html::push_html(&mut compiled_body, iter::once(event));
                None
            }
            _ => Some(event),
        }),
    );
    let mut title = String::new();
    pulldown_cmark::html::push_html(
        &mut compiled_body,
        parser.map_while(|event| match event {
            pulldown_cmark::Event::Code(contents)
            | pulldown_cmark::Event::Html(contents)
            | pulldown_cmark::Event::Text(contents) => {
                title.push_str(&contents.to_string());
                Some(event)
            }
            pulldown_cmark::Event::End(pulldown_cmark::Tag::Heading(..)) => {
                pulldown_cmark::html::push_html(&mut compiled_body, iter::once(event));
                None
            }
            _ => Some(event),
        }),
    );
    pulldown_cmark::html::push_html(&mut compiled_body, parser);
    let file_info = fs::metadata(path).unwrap();
    let creation_time: FileTime = file_info.created().unwrap().into();
    let modification_time: FileTime = file_info.modified().unwrap().into();
    {
        const FORMAT: &str = "%Y.%m.%d";
        let signature = format!(
            r#"<p align="right"><em>- {}, {}"#,
            author_name,
            creation_time.format(FORMAT).to_string()
        );
        if creation_time != modification_time {
            signature.push_str(&format!(
                " (last edit at {})",
                modification_time.format(FORMAT).to_string()
            ))
        }
        signature.push_str("</em></p>");
        pulldown_cmark::html::push_html(
            &mut compiled_body,
            iter::once(pulldown_cmark::Event::Html(CowStr::Borrowed(&signature))),
        );
    }
    let file_name = path.get_file_name();
    CompiledArticleInfo {
        title: if title.is_empty() { file_name } else { title },
        creation_time,
        modification_time,
        body: compiled_body,
        file_name,
    }
}

trait ArticlesIterator<'article_info>: Iterator<Item = (&'article_info Rc<ArticleFileName>, &'article_info ArticleTitle)> {}
impl<'article_info, T> ArticlesIterator<'article_info> for T where T: Iterator<Item = (&'article_info Rc<ArticleFileName>, &'article_info ArticleTitle)> {}
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'article_info, ArticlesList, AuthorName>
where
    ArticlesList: ArticlesIterator<'article_info>,
    AuthorName: Display,
{
    articles_list: ArticlesList,
    author_name: AuthorName,
    background_color_code: &'static str,
    title_color_code: &'static str,
}

pub fn compile_index_variants<'article_info, ArticlesList, AuthorName>(
    articles_list: ArticlesList,
    author_name: AuthorName,
) -> [String; INDEX_VARIANTS_AMOUNT]
where
    ArticlesList: ArticlesIterator<'article_info>,
    AuthorName: Display,
{
    const COLORS: [(&str, &str); 6] = [
        ("C8566B", "F6E5E8"),
        ("E78963", "FBEDE7"),
        ("F2D48F", "FDF8EE"),
        ("9D75BF", "F0EAF5"),
        ("9EC299", "F0F5EF"),
        ("6661AB", "E8E7F2"),
    ];
    let mut index_variants: [String; INDEX_VARIANTS_AMOUNT];
    for (color_index, (title_color_code, background_color_code)) in COLORS.iter().enumerate() {
        index_variants[color_index] = IndexTemplate {
            articles_list,
            author_name,
            background_color_code,
            title_color_code,
        }
        .render()
        .unwrap();
    }
    index_variants
}
