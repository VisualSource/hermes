use utoipa::OpenApi;
pub mod middleware;

pub mod db;
pub mod models;
pub mod routes;
pub mod state;

#[derive(OpenApi)]
#[openapi(
    info(description = "Hermes server"),
    paths(
        routes::api::server::get_server,
        routes::api::server::post_server,
        routes::api::server::delete_server,
        routes::api::server::patch_server
    )
)]
pub struct ApiDoc;
