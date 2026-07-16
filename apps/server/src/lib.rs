use utoipa::OpenApi;

pub mod db;
pub mod models;
pub mod routes;
pub mod state;

#[derive(OpenApi)]
#[openapi(info(description = "Hermes server"), paths())]
pub struct ApiDoc;
