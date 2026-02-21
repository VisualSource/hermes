use actix_web::{Scope, web};

mod account;
mod channel;
mod message;

pub fn api_routes() -> Scope {
    web::scope("/v1")
        .service((
            account::add_encypt_key,
            account::delete_encypt_key,
            account::get_encypt_key,
            account::get_encypt_keys,
            account::get_user,
            account::update_user,
        ))
        .service((
            channel::create_server,
            channel::delete_server,
            channel::update_server,
            channel::get_server,
            channel::get_servers,
        ))
        .service((
            channel::delete_channel,
            channel::get_channel,
            channel::get_channel_messages,
            channel::get_channels,
            channel::update_channel,
        ))
}
