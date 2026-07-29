use actix_web::{HttpResponse, Responder, delete, get, http::StatusCode, patch, post, web};
use actix_web_validation::Validated;
use serde::Deserialize;
use sqlx::{SqlitePool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::channel::{Channel, ChannelKind},
    state::{api_errors::ApplicationError, oauth::jwt::Claims},
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct CreateChannelPayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: String,
    #[validate(length(min = 3, max = 255), non_control_character)]
    category: Option<String>,

    kind: ChannelKind,

    server_id: Uuid,
}

#[utoipa::path(
    tag = "channel",
    request_body = CreateChannelPayload,
    responses(
        (status = 200, description = "new channel", body = Channel),
        (status = 400, description = "invalid payload", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/channel")]
pub async fn create_channel(
    db: web::Data<SqlitePool>,
    Validated(web::Json(body)): Validated<web::Json<CreateChannelPayload>>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Channel>, ApplicationError> {
    //TODO: validate user can make channel on given server

    let id = uuid::Uuid::now_v7();
    let channel = query_as!(
        Channel,
        "INSERT INTO channels VALUES (?,?,?,?,?) RETURNING *",
        id,
        body.kind,
        body.server_id,
        body.name,
        body.category
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(channel))
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct PatchChannelPayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: Option<String>,
    #[validate(length(max = 255), non_control_character)]
    category: Option<String>,
}

#[utoipa::path(
    tag = "channel",
    request_body = PatchChannelPayload,
    responses(
        (status = 201, description = "accepted changes"),
        (status = 400, description = "invalid payload", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[patch("/channel/{channel}")]
pub async fn patch_channel(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    user: web::ReqData<Claims>,
    Validated(web::Json(body)): Validated<web::Json<PatchChannelPayload>>,
) -> Result<HttpResponse, ApplicationError> {
    let channel_id = params.into_inner();
    // TODO: validate user can modify channel

    match (body.name, body.category) {
        (None, None) => {
            return Err(ApplicationError::new(
                StatusCode::BAD_REQUEST,
                "empty body",
                "body",
                Vec::default(),
                None,
            ));
        }
        (Some(name), Some(category)) => {
            query!(
                "UPDATE channels SET name = ?, category = ? WHERE id = ?",
                name,
                category,
                channel_id
            )
            .execute(db.get_ref())
            .await?;
        }
        (Some(name), None) => {
            query!(
                "UPDATE channels SET name = ? WHERE id = ?",
                name,
                &channel_id
            )
            .execute(db.get_ref())
            .await?;
        }
        (None, Some(category)) => {
            query!(
                "UPDATE channels SET category = ? WHERE id = ?;",
                category,
                &channel_id
            )
            .execute(db.get_ref())
            .await?;
        }
    }

    Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
    tag = "channel",
    responses(
        (status = 200, description = "accepted changes", body = Channel),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/channel/{channel}")]
pub async fn get_channel(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Channel>, ApplicationError> {
    let channel_id = params.into_inner();

    // TODO: validate user can fetch channel

    let channel = query_as!(Channel, "SELECT * FROM channels WHERE id = ?", &channel_id)
        .fetch_one(db.get_ref())
        .await?;

    Ok(web::Json(channel))
}

#[utoipa::path(
    tag = "channel",
    responses(
        (status = 201, description = "accepted deletion"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/channel/{channel}")]
pub async fn delete_channel(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    user: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    let channel_id = params.into_inner();

    //TOOD: validate user can delete channel

    query!("DELETE FROM channels WHERE id = ?", &channel_id)
        .execute(db.get_ref())
        .await?;

    Ok(HttpResponse::Accepted().finish())
}
