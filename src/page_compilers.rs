use std::{
    ffi::OsString,
    fs, iter,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use pulldown_cmark::CowStr;

use crate::articles::Articles;

pub struct ArticleInfo {
    title: String,
    file_name: OsString,
    html_body: String,
    creation_time: SystemTime,
    modification_time: SystemTime,
}

pub fn compile_article(path: PathBuf) -> ArticleInfo {
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
    let mut html_body = String::new();
    pulldown_cmark::html::push_html(
        &mut html_body,
        parser.map_while(|event| match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading(..)) => {
                pulldown_cmark::html::push_html(&mut html_body, iter::once(event));
                None
            }
            _ => Some(event),
        }),
    );
    let mut title = String::new();
    pulldown_cmark::html::push_html(
        &mut html_body,
        parser.map_while(|event| match event {
            pulldown_cmark::Event::Code(contents)
            | pulldown_cmark::Event::Html(contents)
            | pulldown_cmark::Event::Text(contents) => {
                title.push_str(&contents.to_string());
                Some(event)
            }
            pulldown_cmark::Event::End(pulldown_cmark::Tag::Heading(..)) => {
                pulldown_cmark::html::push_html(&mut html_body, iter::once(event));
                None
            }
            _ => Some(event),
        }),
    );
    pulldown_cmark::html::push_html(&mut html_body, parser);
    let file_info = fs::metadata(path).unwrap();
    let creation_time = file_info.created().unwrap();
    let modification_time = file_info.modified().unwrap();
    pulldown_cmark::html::push_html(
        &mut html_body,
        iter::once(pulldown_cmark::Event::Text(CowStr::Borrowed(format!(
            "Created at {}, modified at {}",
            creation_time, modification_time.into,
        )))),
    );
    let file_name = path.file_name().unwrap().to_os_string();
    ArticleInfo {
        title: if title.is_empty() {
            file_name.to_string_lossy().to_string()
        } else {
            title
        },
        creation_time,
        modification_time,
        html_body,
        file_name,
    }
}

pub fn compile_index(articles: Arc<Mutex<Articles>>) {}
