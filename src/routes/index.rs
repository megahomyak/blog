use std::sync::Mutex;

use actix_web::{web, HttpResponse};

use crate::website::Website;

#[allow(clippy::unused_async)]
pub async fn index(website: web::Data<Mutex<Website>>) -> HttpResponse {
    HttpResponse::Ok().body(website.lock().unwrap().get_index_page().clone())
}
