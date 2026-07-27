use actix_web::{HttpResponse, Responder, delete, get, patch, post, web};
use serde::Deserialize;
use sqlx::{SqlitePool, query_as};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::channel::{Channel, ChannelKind},
    state::{api_errors::ApplicationError, oauth::jwt::Claims},
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct CreateChannelPayload {
    #[validate(length(min = 3, max = 255))]
    name: String,
    #[validate(length(min = 3, max = 255))]
    category: Option<String>,

    kind: ChannelKind,

    server_id: Uuid,
}

#[post("/channel")]
pub async fn create_channel(
    db: web::Data<SqlitePool>,
    body: web::Json<CreateChannelPayload>,
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
