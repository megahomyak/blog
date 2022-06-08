use std::{fmt::Display, sync::Mutex};

use actix_web::{web, HttpResponse};

use crate::articles::Articles;

#[allow(clippy::unused_async)]
pub async fn index<AuthorName>(articles: web::Data<Mutex<Articles<AuthorName>>>) -> HttpResponse
where
    AuthorName: Display + Send,
{
    HttpResponse::Ok().body(articles.lock().unwrap().get_index_page().clone())
}
