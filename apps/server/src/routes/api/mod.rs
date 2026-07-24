use utoipa_actix_web::service_config::ServiceConfig;

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
        // Served but not documented (no `#[utoipa::path]`, so no `OpenApiFactory`
        // impl). Register through the raw-config passthrough.
        .map(|c| {
            c.service(account::add_encypt_key)
                .service(account::delete_encypt_key)
                .service(account::get_encypt_key)
                .service(account::get_encypt_keys)
                .service(account::get_user)
                .service(account::update_user)
        });
}
