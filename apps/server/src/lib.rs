use utoipa::OpenApi;
use utoipa_actix_web::AppExt;
pub mod middleware;

pub mod db;
pub mod models;
pub mod routes;
pub mod state;

/// Base OpenAPI document. Carries the spec metadata (info/components/tags);
/// the paths are collected automatically from `#[utoipa::path]` handlers as
/// they are mounted (see [`openapi`]).
#[derive(OpenApi)]
#[openapi(info(description = "Hermes server"))]
pub struct ApiDoc;

/// Build the full OpenAPI document by mounting the `/api/v1` route set onto a
/// throwaway `UtoipaApp` and collecting the paths. This never listens or serves
/// — it exists purely so the offline generator ([`crate::bin`]'s `gen-openapi`)
/// shares the exact same route registration as the live server, eliminating
/// drift between the served routes and the documented ones.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_app, api) = actix_web::App::new()
        .into_utoipa_app()
        .openapi(ApiDoc::openapi())
        .service(utoipa_actix_web::scope("/api/v1").configure(routes::api::configure_v1))
        .split_for_parts();
    api
}
