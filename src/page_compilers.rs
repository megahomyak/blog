use std::{fmt::Display, fs, iter, path::PathBuf, rc::Rc, cell::RefCell};

use askama::Template;
use pulldown_cmark::CowStr;

use crate::{
    articles::{FileTime, IndexArticleInfo, INDEX_VARIANTS_AMOUNT, ModificationTime},
    file_watcher::WithFileName,
};

pub struct CompiledArticleInfo {
    pub title: Rc<String>,
    pub file_name: Rc<String>,
    pub body: String,
    pub modification_time: ModificationTime,
}

pub fn compile_article<AuthorName>(path: PathBuf, author_name: AuthorName) -> CompiledArticleInfo
where
    AuthorName: Display,
{
    let file_contents = fs::read_to_string(&path).unwrap();
    let mut parser = pulldown_cmark::Parser::new_ext(
        &file_contents,
        {
            let mut options = pulldown_cmark::Options::empty();
            options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
            options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
            options
        },
    ).into_iter();
    let compiled_body = RefCell::new(String::new());
    pulldown_cmark::html::push_html(
        &mut compiled_body.borrow_mut(), parser.by_ref().map_while(|event| match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading(..)) => {
                pulldown_cmark::html::push_html(&mut compiled_body.borrow_mut(), iter::once(event));
                None
            }
            _ => Some(event),
        }),
    );
    let mut title = String::new();
    pulldown_cmark::html::push_html(
        &mut compiled_body.borrow_mut(),
        parser.by_ref().map_while(|event| match &event {
            pulldown_cmark::Event::Code(contents)
            | pulldown_cmark::Event::Html(contents)
            | pulldown_cmark::Event::Text(contents) => {
                title.push_str(&contents.to_string());
                Some(event)
            }
            pulldown_cmark::Event::End(pulldown_cmark::Tag::Heading(..)) => {
                pulldown_cmark::html::push_html(&mut compiled_body.borrow_mut(), iter::once(event));
                None
            }
            _ => Some(event),
        }),
    );
    pulldown_cmark::html::push_html(&mut compiled_body.borrow_mut(), parser);
    let file_info = fs::metadata(&path).unwrap();
    let modification_time: FileTime = file_info.modified().unwrap().into();
    {
        const FORMAT: &str = "%Y.%m.%d";
        let creation_time: FileTime = file_info.created().unwrap().into();
        let mut signature = format!(
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
            &mut compiled_body.borrow_mut(),
            iter::once(pulldown_cmark::Event::Html(CowStr::Borrowed(&signature))),
        );
    }
    let file_name = Rc::new(path.get_file_name());
    CompiledArticleInfo {
        title: if title.is_empty() { file_name.clone() } else { Rc::new(title) },
        modification_time,
        body: compiled_body.into_inner(),
        file_name,
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'index_template, AuthorName>
where
    AuthorName: Display,
{
    articles_list: &'index_template Vec<IndexArticleInfo>,
    author_name: &'index_template AuthorName,
    background_color_code: &'static str,
    title_color_code: &'static str,
}

pub fn compile_index_variants<AuthorName>(
    articles_list: Vec<IndexArticleInfo>,
    author_name: AuthorName,
) -> [String; INDEX_VARIANTS_AMOUNT]
where
    AuthorName: Display,
{
    let apply_colors = move |title_color_code, background_color_code| {
        IndexTemplate {
            articles_list: &articles_list,
            author_name: &author_name,
            background_color_code,
            title_color_code,
        }
        .render()
        .unwrap()
    };
    let index_variants: [String; INDEX_VARIANTS_AMOUNT] = [
        apply_colors("C8566B", "F6E5E8"),
        apply_colors("E78963", "FBEDE7"),
        apply_colors("F2D48F", "FDF8EE"),
        apply_colors("9D75BF", "F0EAF5"),
        apply_colors("9EC299", "F0F5EF"),
        apply_colors("6661AB", "E8E7F2"),
    ];
    index_variants
}
