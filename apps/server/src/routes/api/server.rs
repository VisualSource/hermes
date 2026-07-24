use actix_web::{HttpResponse, Responder, delete, get, http::StatusCode, patch, post, web};
use actix_web_validation::Validated;
use serde::Deserialize;
use sqlx::{SqlitePool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::server::Server,
    state::{
        api_errors::{ApplicationError, ErrorDetail, from_sqlx_error},
        oauth::jwt::Claims,
    },
};

#[utoipa::path(
    tag = "server", 
    description = "fetch a server",
    responses(
        (status = 200, description = "single server info", body = Server)
    )
)]
#[get("/server/{server}")]
pub async fn get_server(
    db: web::Data<SqlitePool>,
    server: web::Path<uuid::Uuid>,
    user: web::ReqData<Claims>,
) -> actix_web::Result<web::Json<Server>> {
    let server_id = server.into_inner();

    let server = Server::get(&db, server_id).await.map_err(from_sqlx_error)?;

    Ok(web::Json(server))
}

#[utoipa::path(tag = "server", description = "delete a server")]
#[delete("/server/{server}")]
pub async fn delete_server(db: web::Data<SqlitePool>, server: String) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[utoipa::path(tag = "server", description = "create a server")]
#[post("/server/{server}")]
pub async fn post_server(db: web::Data<SqlitePool>, server: String) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct ServerPatch {
    #[validate(length(min = 3))]
    name: Option<String>,
    #[validate(url)]
    icon: Option<String>,
}

#[utoipa::path(
    tag = "server", 
    description = "update a server",
    responses(
        (status = 201, description = "updated server properties")
    )
)]
#[patch("/server/{server}")]
pub async fn patch_server(
    db: web::Data<SqlitePool>,
    body: Validated<web::Json<ServerPatch>>,
    server: web::Path<Uuid>,
    user: web::ReqData<Claims>,
) -> actix_web::Result<impl Responder> {
    let server_id = server.into_inner();

    // allow non owner to update server?
    match (&body.icon, &body.name) {
        (None, None) => {
            return Err(ApplicationError::new(
                StatusCode::BAD_REQUEST,
                "bad request",
                "body",
                vec![ErrorDetail::new(4001, "body", "no body are where set")],
                None,
            )
            .into());
        }
        (Some(icon), None) => {
            query!(
                "UPDATE servers SET icon = ? WHERE id = ? AND owner_id = ?",
                icon,
                server_id,
                user.sub
            )
            .execute(db.get_ref())
            .await
            .map_err(from_sqlx_error)?;
        }
        (None, Some(name)) => {
            query!(
                "UPDATE servers SET name = ? WHERE id = ? AND owner_id = ?",
                name,
                server_id,
                user.sub
            )
            .execute(db.get_ref())
            .await
            .map_err(from_sqlx_error)?;
        }
        (Some(icon), Some(name)) => {
            query!(
                "UPDATE servers SET name = ?, icon = ? WHERE id = ? AND owner_id = ?",
                name,
                icon,
                server_id,
                user.sub
            )
            .execute(db.get_ref())
            .await
            .map_err(from_sqlx_error)?;
        }
    }

    Ok(HttpResponse::Accepted())
}

#[utoipa::path(tags = ["server","user"], description="list servers that the current user is on")]
#[get("/servers")]
pub async fn list_servers(
    db: web::Data<SqlitePool>,
    user: web::ReqData<Claims>,
) -> actix_web::Result<web::Json<Vec<Server>>> {
    let servers = query_as!(Server,r#"SELECT servers.* FROM servers JOIN server_members ON server_members.server_id = servers.id WHERE server_members.user_id = ? LIMIT 30"#,  user.sub).fetch_all(db.get_ref()).await.map_err(from_sqlx_error)?;

    Ok(web::Json(servers))
}

// ->
