//send
//get
//edit
//dlete

//get by id

use std::str::FromStr;

use actix_web::{HttpResponse, delete, get, http::StatusCode, patch, post, web};
use actix_web_validation::Validated;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::channel::Message,
    state::{
        api_errors::{ApplicationError, ErrorDetail, InnerError},
        oauth::jwt::Claims,
    },
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct PostMessagePayload {
    content: String,
}

#[utoipa::path(
    tags = ["channel","message"],
    request_body = PostMessagePayload,
    responses(
        (status = 200, description = "new message", body = Message),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/channel/{channel}/message")]
pub async fn create_message(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    params: web::Path<Uuid>,
    body: Validated<web::Json<PostMessagePayload>>,
) -> Result<web::Json<Message>, ApplicationError> {
    //TODO: validate user can send message
    let message_id = Uuid::now_v7();
    let channel_id = params.into_inner();
    let created_at = time::OffsetDateTime::now_utc();

    let message = query_as!(
        Message,
        "INSERT INTO messages VALUES (?,?,?,?,?, NULL, NULL) RETURNING *",
        message_id,
        channel_id,
        claims.sub,
        body.content,
        created_at
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(message))
}

#[utoipa::path(
    tags = ["channel","message"],
    responses(
        (status = 200, description = "message", body = Message),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/channel/{channel}/message/{message}")]
pub async fn get_message(
    db: web::Data<SqlitePool>,
    user: web::ReqData<Claims>,
    params: web::Path<(Uuid, Uuid)>,
) -> Result<web::Json<Message>, ApplicationError> {
    //TODO: validate user can get message
    let (channel_id, message_id) = params.into_inner();

    let message = query_as!(
        Message,
        "SELECT * FROM messages WHERE id = ? AND channel_id = ?",
        message_id,
        channel_id
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(message))
}

#[derive(Debug, Deserialize, ToSchema)]
struct PatchMessagePayload {
    content: String,
}

#[utoipa::path(
    tags = ["channel","message"],
    request_body = PatchMessagePayload,
    responses(
        (status = 201, description = "accepted update message"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[patch("/channel/{channel}/message/{message}")]
pub async fn patch_message(
    db: web::Data<SqlitePool>,
    user: web::ReqData<Claims>,
    body: web::Json<PatchMessagePayload>,
    params: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, ApplicationError> {
    //TODO: validate user can update message
    let (channel_id, message_id) = params.into_inner();
    let edited_at = time::OffsetDateTime::now_utc();

    query!(
        "UPDATE messages SET content = ?, edited_at = ? WHERE id = ? AND channel_id = ?",
        body.content,
        edited_at,
        message_id,
        channel_id
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
    tags = ["channel","message"],
    responses(
        (status = 201, description = "accepted deletion"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/channel/{channel}/message/{message}")]
pub async fn delete_message(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    user: web::ReqData<Claims>,
) -> Result<HttpResponse, ApplicationError> {
    //TODO validate user can delete this message
    let (channel_id, message_id) = params.into_inner();

    let deleted_at = time::OffsetDateTime::now_utc();

    query!(
        "UPDATE messages SET deleted_at = ? WHERE id = ? AND channel_id = ?",
        deleted_at,
        channel_id,
        message_id
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct MessagesQuery {
    cursor: Option<String>,
    /*#[serde(with = "time::serde::rfc3339::option")]
    from: Option<time::OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    to: Option<time::OffsetDateTime>,*/
}

#[derive(Debug, Serialize, ToSchema)]
struct MessageQueryResult {
    pub results: Vec<Message>,
    pub count: usize,
    pub cursor: Option<String>,
}

#[utoipa::path(
    tags = ["channel","message"],
    responses(
        (status = 200, description = "accepted deletion", body = MessageQueryResult),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/channel/{channel}/messages")]
pub async fn list_messages(
    db: web::Data<SqlitePool>,
    Validated(web::Query(query)): Validated<web::Query<MessagesQuery>>,
    params: web::Path<Uuid>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<MessageQueryResult>, ApplicationError> {
    // validate user can fetch messages from this channel
    let channel_id = params.into_inner();

    let mut messages = if let Some(bcursor) = query.cursor {
        let (created_at, id) = parse_cursor(bcursor)?;

        query_as!(Message, "SELECT * FROM messages WHERE (created_at,id) < (?,?) AND channel_id = ? AND deleted_at = NULL ORDER BY created_at DESC, id DESC LIMIT 51",&created_at,id,channel_id).fetch_all(db.get_ref()).await?
    } else {
        let now = time::OffsetDateTime::now_utc();
        query_as!(Message, "SELECT * FROM messages WHERE created_at < ? AND channel_id = ? AND deleted_at = NULL ORDER BY created_at DESC, id DESC LIMIT 51",now,channel_id).fetch_all(db.get_ref()).await?
    };

    let (results, cursor) = if messages.len() > 50 {
        let cursor = if let Some(last) = messages.last() {
            let mut cur = String::default();

            let timestamp = last
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .map_err(|err| {
                    ApplicationError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal server error",
                        "server",
                        Vec::default(),
                        Some(InnerError::new(err.to_string())),
                    )
                })?;

            base64::prelude::BASE64_URL_SAFE
                .encode_string(format!("{},{}", timestamp, last.id), &mut cur);
            Some(cur)
        } else {
            None
        };

        messages.remove(51);
        (messages, cursor)
    } else {
        (messages, None)
    };

    let count = results.len();
    Ok(web::Json(MessageQueryResult {
        results,
        count,
        cursor,
    }))
}

fn make_cursor_error(code: u16, err: Option<InnerError>) -> ApplicationError {
    ApplicationError::new(
        StatusCode::BAD_REQUEST,
        "a query param is invalid or malformed",
        "query",
        vec![ErrorDetail::new(
            code,
            "cursor",
            "cursor is invalid or malformed",
        )],
        err,
    )
}

fn parse_cursor(raw_cursor: String) -> Result<(time::OffsetDateTime, Uuid), ApplicationError> {
    let decoded_cursor = base64::prelude::BASE64_URL_SAFE
        .decode(raw_cursor)
        .map_err(|err| make_cursor_error(4001, Some(InnerError::new(err.to_string()))))?;
    let str_cursor = String::from_utf8(decoded_cursor)
        .map_err(|err| make_cursor_error(4002, Some(InnerError::new(err.to_string()))))?;

    let (timestamp, id) = str_cursor
        .split_once(',')
        .ok_or_else(|| make_cursor_error(4003, None))?;
    let t = time::OffsetDateTime::parse(timestamp, &time::format_description::well_known::Rfc3339)
        .map_err(|err| make_cursor_error(4004, Some(InnerError::new(err.to_string()))))?;
    let uuid = Uuid::from_str(id)
        .map_err(|err| make_cursor_error(4005, Some(InnerError::new(err.to_string()))))?;

    Ok((t, uuid))
}
