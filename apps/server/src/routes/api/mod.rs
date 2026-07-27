use utoipa_actix_web::service_config::ServiceConfig;

use crate::routes::api::server::list_servers;

pub mod account;
pub mod channel;
pub mod message;
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
        .service(list_servers);
}
