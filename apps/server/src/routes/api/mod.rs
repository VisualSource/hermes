use actix_web::{dev::HttpServiceFactory, middleware::from_fn, web};

use crate::middleware;

pub mod account;
pub mod channel;
pub mod message;
pub mod server;

pub fn api_routes() -> impl HttpServiceFactory {
    web::scope("/v1")
        .wrap(from_fn(middleware::require_jwt))
        .service((
            account::add_encypt_key,
            account::delete_encypt_key,
            account::get_encypt_key,
            account::get_encypt_keys,
            account::get_user,
            account::update_user,
            server::get_server,
            server::delete_server,
            server::post_server,
            server::patch_server,
        ))
}
