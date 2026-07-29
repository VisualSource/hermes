use actix_web::{HttpResponse, Responder, delete, get, http::StatusCode, patch, post, web};
use actix_web_validation::Validated;
use serde::Deserialize;
use sqlx::{SqlitePool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        channel::Channel,
        server::{Server, ServerMember},
    },
    state::{
        api_errors::{ApplicationError, ErrorDetail},
        oauth::jwt::Claims,
    },
};

#[utoipa::path(
    tag = "server", 
    description = "fetch a server",
    responses(
        (status = 200, description = "single server info", body = Server),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/server/{server}")]
pub async fn get_server(
    db: web::Data<SqlitePool>,
    server: web::Path<uuid::Uuid>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Server>, ApplicationError> {
    let server_id = server.into_inner();
    let server = query_as!(Server, "SELECT * FROM servers WHERE id = ?", server_id)
        .fetch_one(db.get_ref())
        .await?;

    Ok(web::Json(server))
}

#[utoipa::path(
    tag = "server",
    description = "delete a server",
    responses(
        (status = 201, description = "server deleted"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/server/{server}")]
pub async fn delete_server(
    db: web::Data<SqlitePool>,
    params: web::Path<uuid::Uuid>,
    user: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    let server_id = params.into_inner();

    query!(
        "DELETE FROM servers WHERE id = ? AND owner_id = ?",
        server_id,
        user.sub
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted())
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct PostServerPayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: String,
    #[validate(url)]
    icon: Option<String>,
}

#[utoipa::path(
    tag = "server", 
    description = "create a server",
    request_body = PostServerPayload,
    responses(
        (status = 200, description = "created server", body = Server),
        (status = 400, description = "invalid payload", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server")]
pub async fn post_server(
    db: web::Data<SqlitePool>,
    user: web::ReqData<Claims>,
    body: Validated<web::Json<PostServerPayload>>,
) -> Result<impl Responder, ApplicationError> {
    let server_id = uuid::Uuid::now_v7();
    let now = time::OffsetDateTime::now_utc();

    let server = query_as!(
        Server,
        "INSERT INTO servers VALUES (?,?,?,?,?) RETURNING *",
        server_id,
        body.name,
        user.sub,
        now,
        body.icon
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(server))
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct PatchServerPayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: Option<String>,
    #[validate(url)]
    icon: Option<String>,
}

#[utoipa::path(
    tag = "server", 
    description = "update a server",
    request_body = PatchServerPayload,
    responses(
        (status = 201, description = "updated server properties"),
        (status = 400, description = "invalid payload", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[patch("/server/{server}")]
pub async fn patch_server(
    db: web::Data<SqlitePool>,
    Validated(web::Json(body)): Validated<web::Json<PatchServerPayload>>,
    server: web::Path<Uuid>,
    user: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
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
            .await?;
        }
        (None, Some(name)) => {
            query!(
                "UPDATE servers SET name = ? WHERE id = ? AND owner_id = ?",
                name,
                server_id,
                user.sub
            )
            .execute(db.get_ref())
            .await?;
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
            .await?;
        }
    }

    // TODO: notify connected clients of changes?

    Ok(HttpResponse::Accepted())
}

#[utoipa::path(
    tags = ["server","user"], 
    description="list servers that the current user is in",
    responses(
        (status = 200, description = "created server", body = Vec<Server>),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/servers")]
pub async fn list_servers(
    db: web::Data<SqlitePool>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Vec<Server>>, ApplicationError> {
    let servers = query_as!(Server,r#"SELECT servers.* FROM servers JOIN server_members ON server_members.server_id = servers.id WHERE server_members.user_id = ? LIMIT 30"#,  user.sub).fetch_all(db.get_ref()).await?;

    Ok(web::Json(servers))
}

#[utoipa::path(
    tags = ["server","members"], 
    description= "server members list",
    responses(
        (status = 200, description = "list of members", body = Vec<ServerMember>),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/server/{server}/members")]
pub async fn list_members(
    db: web::Data<SqlitePool>,
    params: web::Path<uuid::Uuid>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Vec<ServerMember>>, ApplicationError> {
    let server_id = params.into_inner();

    let members = query_as!(
        ServerMember,
        "SELECT * FROM server_members WHERE server_id = ?",
        server_id
    )
    .fetch_all(db.get_ref())
    .await?;

    Ok(web::Json(members))
}

#[utoipa::path(
    tags = ["server","channels"],
    responses(
        (status = 200, description = "list of channels", body = Vec<ServerMember>),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/server/{server}/channels")]
pub async fn list_channels(
    db: web::Data<SqlitePool>,
    params: web::Path<uuid::Uuid>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Vec<Channel>>, ApplicationError> {
    let server_id = params.into_inner();

    //TODO: validate user can fetch

    let channels = query_as!(
        Channel,
        "SELECT * FROM channels WHERE server_id = ?",
        &server_id
    )
    .fetch_all(db.get_ref())
    .await?;

    Ok(web::Json(channels))
}

/*
 * create server
 * delete server
 * update server
 * get server
 *
 * list servers by user
 * list server members
 *
 *
 *
 */
