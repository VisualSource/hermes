use actix_web::{HttpResponse, Responder, delete, get, patch, post};

/// fetch server linked to user
#[get("/servers")]
pub async fn get_servers() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

/// create a server
#[post("/server")]
pub async fn create_server() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

///fetch server details
#[get("/server/{server}")]
pub async fn get_server() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

/// update server
#[patch("/server/{server}")]
pub async fn update_server() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

// delete server
#[delete("/server/{server}")]
pub async fn delete_server() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

/// list all channels in server
#[get("/server/{server}/channels")]
pub async fn get_channels() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

/// delete details channel
#[delete("/server/{server}/channel/{channel}")]
pub async fn delete_channel() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

#[patch("/server/{server}/channel/{channel}")]
pub async fn update_channel() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

/// get details about channel
#[get("/server/{server}/channel/{channel}")]
pub async fn get_channel() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

/// cursor paginadted endpoint via timestamp
#[get("/server/{server}/channel/{channel}/messages")]
pub async fn get_channel_messages() -> impl Responder {
    HttpResponse::NotImplemented().finish()
}
