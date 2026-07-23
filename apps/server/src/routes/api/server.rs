use actix_web::{HttpResponse, Responder, delete, get, patch, post, web};
use sqlx::SqlitePool;

use crate::state::oauth::jwt::Claims;

#[utoipa::path(tag = "server", description = "fetch a server")]
#[get("/server/{server}")]
pub async fn get_server(
    db: web::Data<SqlitePool>,
    server: String,
    user: web::ReqData<Claims>,
) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[utoipa::path(tag = "server", description = "delete a server")]
#[delete("/server/{server}")]
pub async fn delete_server(db: web::Data<SqlitePool>, server: String) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[utoipa::path(tag = "server", description = "create a server")]
#[post("/server/{server}")]
pub async fn post_server(db: web::Data<SqlitePool>, server: String) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[utoipa::path(tag = "server", description = "update a server")]
#[patch("/server/{server}")]
pub async fn patch_server(db: web::Data<SqlitePool>, server: String) -> impl Responder {
    HttpResponse::NotImplemented()
}
