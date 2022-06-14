use std::sync::{Arc, Mutex};

use actix_files::NamedFile;
use actix_web::{body::BoxBody, web, HttpRequest, HttpResponse, Responder};

use crate::{context::Context, file_watcher};

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::large_enum_variant)]
pub enum FileOrText {
    File(NamedFile),
    Text(HttpResponse),
}

impl Responder for FileOrText {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            Self::File(file) => file.into_response(req),
            Self::Text(text) => text,
        }
    }
}

#[allow(clippy::unused_async)]
pub async fn file<ArticlesWatchGuard, ConfigWatchGuard>(
    context: web::Data<Mutex<Context<ArticlesWatchGuard, ConfigWatchGuard>>>,
    path_arguments: web::Path<String>,
) -> FileOrText where ArticlesWatchGuard: file_watcher::WatchGuard {
    use FileOrText::{File, Text};
    let file_name: Arc<str> = path_arguments.into_inner().into();
    match context.lock().unwrap().get_article(&file_name) {
        Some(article) => Text(HttpResponse::Ok().body(article)),
        None => match NamedFile::open(
            context
                .lock()
                .unwrap()
                .config()
                .files_directory
                .join(&file_name[..]),
        ) {
            Ok(file) => File(file),
            Err(_) => {
                Text(HttpResponse::NotFound().body("Sorry, the file you requested isn't found!"))
            }
        },
    }
}
