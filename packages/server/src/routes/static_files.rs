use actix_web::{Responder, Scope, get, web};

#[get("/stylesheet.css")]
pub async fn get_stylesheet() -> impl Responder {
    actix_files::NamedFile::open_async("./public/stylesheet.css").await
}

#[get("/favicon.ico")]
pub async fn get_favicon() -> impl Responder {
    actix_files::NamedFile::open_async("./public/favicon.ico").await
}

#[get("/shared.js")]
pub async fn get_sharedjs() -> impl Responder {
    actix_files::NamedFile::open_async("./public/shared.js").await
}

pub fn get_static_files() -> Scope {
    web::scope("/static").service((get_favicon, get_stylesheet,get_sharedjs))
}
