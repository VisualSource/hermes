use actix_web::{HttpResponse, Responder, delete, get, patch, post};

#[patch["/account"]]
pub async fn update_user() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

#[get("/account")]
pub async fn get_user() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

#[get("/account/keys")]
pub async fn get_encypt_keys() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

#[get("/account/key/{uuid}")]
pub async fn get_encypt_key() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

#[post("/account/key")]
pub async fn add_encypt_key() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

#[delete("/account/key/{uuid}")]
pub async fn delete_encypt_key() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
