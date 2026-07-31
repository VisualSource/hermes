use utoipa_actix_web::service_config::ServiceConfig;

pub mod account;
pub mod channel;
pub mod dm;
mod invite;
pub mod message;
pub mod roles;
pub mod server;
/// Register all `/api/v1` handlers. Documented handlers (those with
/// `#[utoipa::path]`) are collected into the OpenAPI doc when this runs under a
/// `UtoipaApp`; under a plain actix config the collected paths are dropped.
///
/// This is the single source of truth for the `/api/v1` route set, shared by
/// the live server ([`crate::routes`] wiring in `main.rs`) and the offline
/// OpenAPI generator ([`crate::openapi`]).
pub fn configure_v1(cfg: &mut ServiceConfig) {
    // Documented (carry `#[utoipa::path]`) — collected into the spec.
    cfg.service(server::get_server)
        .service(server::delete_server)
        .service(server::post_server)
        .service(server::patch_server)
        .service(server::list_members)
        .service(server::list_channels)
        .service(server::list_servers)
        .service(channel::create_channel)
        .service(channel::delete_channel)
        .service(channel::get_channel)
        .service(channel::patch_channel)
        .service(roles::add_role_to_user)
        .service(roles::create_role)
        .service(roles::delete_role)
        .service(roles::get_role)
        .service(roles::patch_role)
        .service(roles::remove_role_from_user)
        .service(dm::get_dms)
        .service(dm::create_friend_request)
        .service(dm::delete_friend_request)
        .service(dm::list_friend_requests)
        .service(dm::put_friend_request)
        .service(message::create_message)
        .service(message::delete_message)
        .service(message::get_message)
        .service(message::list_messages)
        .service(message::patch_message)
        .service(invite::create_invite)
        .service(invite::revoke_invite);
}
