use std::sync::Mutex;

use actix_web::{web, HttpResponse};

use crate::{context::Context, file_watcher};

#[allow(clippy::unused_async)]
pub async fn index<ArticlesWatchGuard>(
    context: web::Data<Mutex<Context<ArticlesWatchGuard>>>,
) -> HttpResponse where ArticlesWatchGuard: file_watcher::WatchGuard {
    HttpResponse::Ok().body(context.lock().unwrap().get_index_page().clone())
}
