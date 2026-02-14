use actix_web::{HttpResponse, Responder, get};

pub mod api;
pub mod oauth;
pub mod websocket;

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, World")
}
