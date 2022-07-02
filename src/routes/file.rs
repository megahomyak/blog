use std::sync::{Arc, Mutex};

use actix_files::NamedFile;
use actix_web::{body::BoxBody, web, HttpRequest, HttpResponse, Responder};

use crate::website::Website;

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
pub async fn file(
    website: web::Data<Mutex<Website>>,
    path_arguments: web::Path<String>,
) -> FileOrText {
    use FileOrText::{File, Text};
    let file_name: Arc<str> = path_arguments.into_inner().into();
    let article = website.lock().unwrap().get_article(&file_name);
    match article {
        Some(article) => Text(HttpResponse::Ok().body(article)),
        None => match NamedFile::open(
            website
                .lock()
                .unwrap()
                .config()
                .lock()
                .unwrap()
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
