use std::{
    fmt::Display,
    path::Path,
    sync::{Arc, Mutex},
};

use actix_files::NamedFile;
use actix_web::{body::BoxBody, web, HttpRequest, HttpResponse, Responder};

use crate::{articles::Articles, constants::FILES_DIRECTORY_NAME};

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
pub async fn file<AuthorName>(
    articles: web::Data<Mutex<Articles<AuthorName>>>,
    path_arguments: web::Path<String>,
) -> FileOrText
where
    AuthorName: Display + Send,
{
    use FileOrText::{File, Text};
    let file_name: Arc<str> = path_arguments.into_inner().into();
    match articles.lock().unwrap().get_article(&file_name) {
        Some(article) => Text(HttpResponse::Ok().body(article)),
        None => match NamedFile::open(Path::new(FILES_DIRECTORY_NAME).join(&file_name[..])) {
            Ok(file) => File(file),
            Err(_) => {
                Text(HttpResponse::NotFound().body("Sorry, the file you requested isn't found!"))
            }
        },
    }
}
