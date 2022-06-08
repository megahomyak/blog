use std::{
    io,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

use actix_web::{web, App, HttpServer};
use articles::Articles;
use constants::{ARTICLES_DIRECTORY_NAME, AUTHOR_NAME};
use file_watcher::{hotwatch::HotwatchFileWatcher, FileWatcher};

mod articles;
mod constants;
mod file_watcher;
mod page_compilers;
mod routes;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let articles = Arc::new(Mutex::new(Articles::new(
        AUTHOR_NAME,
        Path::new(ARTICLES_DIRECTORY_NAME),
    )));
    thread::spawn({
        let mut articles = articles.clone();
        move || {
            let file_watcher = HotwatchFileWatcher::new(move |event| {
                file_watcher::handle_event(&mut articles, event);
            });
            file_watcher.watch(Path::new(ARTICLES_DIRECTORY_NAME));
        }
    });
    HttpServer::new({
        let articles = articles.clone();
        move || {
            App::new()
                .app_data(web::Data::from(articles.clone()))
                .route("/", web::get().to(routes::index::<&'static str>))
                .route("/{filename}", web::get().to(routes::file::<&'static str>))
        }
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
