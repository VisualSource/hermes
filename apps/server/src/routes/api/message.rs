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

    let r = query!(
        "UPDATE messages SET deleted_at = ? WHERE id = ? AND channel_id = ?",
        deleted_at,
        message_id,
        channel_id,
    )
    .execute(db.get_ref())
    .await?;

    if r.rows_affected() != 1 {
        return Err(ApplicationError::new(
            StatusCode::NOT_FOUND,
            "no message was found with given id",
            "url",
            vec![ErrorDetail::new(
                4001,
                "message",
                "no message with given id",
            )],
            None,
        ));
    }

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

    // One row past the page size: its presence is what says "there's more".
    let mut messages = if let Some(bcursor) = query.cursor {
        let (created_at, id) = parse_cursor(bcursor)?;

        query_as!(Message, "SELECT * FROM messages WHERE (created_at,id) < (?,?) AND channel_id = ? AND deleted_at IS NULL ORDER BY created_at DESC, id DESC LIMIT 51",&created_at,id,channel_id).fetch_all(db.get_ref()).await?
    } else {
        let now = time::OffsetDateTime::now_utc();
        query_as!(Message, "SELECT * FROM messages WHERE created_at < ? AND channel_id = ? AND deleted_at IS NULL ORDER BY created_at DESC, id DESC LIMIT 51",now,channel_id).fetch_all(db.get_ref()).await?
    };

    let (results, cursor) = if messages.len() > 50 {
        // The cursor has to point at the last message the caller actually gets
        // (index 49), not the lookahead row — otherwise the next page skips it.
        let last = &messages[49];

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

        let mut cursor = String::default();
        base64::prelude::BASE64_URL_SAFE
            .encode_string(format!("{},{}", timestamp, last.id), &mut cursor);

        messages.truncate(50);
        (messages, Some(cursor))
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

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::http::header::AUTHORIZATION;
    use actix_web::test;
    use serde_json::json;
    use time::Duration;
    use time::OffsetDateTime;

    use crate::test_support::TestCtx;

    use super::*;

    /// `MessageQueryResult` is serialize-only, so tests read the body through
    /// this mirror rather than adding a `Deserialize` derive to the wire type.
    #[derive(Debug, Deserialize)]
    struct QueryResult {
        results: Vec<Message>,
        count: usize,
        cursor: Option<String>,
    }

    /// Seed `count` messages a minute apart in the past, and return their ids in
    /// the order `list_messages` should hand them back: newest first.
    ///
    /// Everything is backdated so the no-cursor branch (`created_at < now`)
    /// can't race the wall clock.
    async fn seed_backlog(ctx: &TestCtx, channel: Uuid, user: Uuid, count: usize) -> Vec<Uuid> {
        let oldest = OffsetDateTime::now_utc() - Duration::days(1);

        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            ids.push(
                ctx.seed_message_at(
                    channel,
                    user,
                    &format!("message {i}"),
                    oldest + Duration::minutes(i as i64),
                )
                .await,
            );
        }

        ids.reverse();
        ids
    }

    fn encode_cursor(created_at: OffsetDateTime, id: Uuid) -> String {
        let timestamp = created_at
            .format(&time::format_description::well_known::Rfc3339)
            .expect("format cursor timestamp");

        base64::prelude::BASE64_URL_SAFE.encode(format!("{timestamp},{id}"))
    }

    async fn list(ctx: &TestCtx, user: Uuid, channel: Uuid, cursor: Option<&str>) -> QueryResult {
        // The base64url alphabet needs no percent-encoding, and `=` padding is
        // kept verbatim by the form decoder (it only splits on the first `=`).
        let uri = match cursor {
            Some(cursor) => format!("/api/v1/channel/{channel}/messages?cursor={cursor}"),
            None => format!("/api/v1/channel/{channel}/messages"),
        };

        let resp = ctx
            .as_user(user)
            .call(test::TestRequest::get().uri(&uri))
            .await;

        assert_eq!(resp.status(), StatusCode::OK);

        test::read_body_json(resp).await
    }

    #[actix_web::test]
    async fn create_message() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/channel/{channel}/message"))
            .set_json(json!({
                "content": "Some Message"
            }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let srv: Message = test::read_body_json(resp).await;

        assert_eq!(srv.user_id, Some(user));
        assert_eq!(srv.channel_id, channel);
        assert_eq!(srv.edited_at, None);
        assert_eq!(srv.deleted_at, None);
        assert_eq!(srv.content, "Some Message");
    }

    #[actix_web::test]
    async fn get_message() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;

        let msg = ctx.seed_message(channel, user, "Some content").await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/channel/{channel}/message/{msg}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let srv: Message = test::read_body_json(resp).await;

        assert_eq!(srv.id, msg);
        assert_eq!(srv.user_id, Some(user));
        assert_eq!(srv.channel_id, channel);
    }

    #[actix_web::test]
    async fn patch_message() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        let msg = ctx.seed_message(channel, user, "Some content").await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/channel/{channel}/message/{msg}"))
            .set_json(json!({
                "content": "Some Message"
            }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let r = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = ?")
            .bind(msg)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to query");

        assert_eq!(r.user_id, Some(user));
        assert_eq!(r.channel_id, channel);
        assert!(r.edited_at.is_some());
        assert_eq!(r.deleted_at, None);
        assert_eq!(r.content, "Some Message");
    }

    #[actix_web::test]
    async fn delete_message() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;

        let msg = ctx.seed_message(channel, user, "Some content").await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/channel/{channel}/message/{msg}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let r = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = ?")
            .bind(msg)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to query");
        assert_eq!(r.id, msg);
        assert_eq!(r.user_id, Some(user));
        assert_eq!(r.channel_id, channel);
        assert!(r.deleted_at.is_some());
    }

    #[actix_web::test]
    async fn list_messages_returns_the_channel_backlog_newest_first() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        let expected = seed_backlog(&ctx, channel, user, 3).await;

        let page = list(&ctx, user, channel, None).await;

        assert_eq!(page.count, 3);
        assert_eq!(page.results.len(), 3);
        assert_eq!(page.cursor, None, "a partial page has nothing to page to");

        let ids: Vec<Uuid> = page.results.iter().map(|m| m.id).collect();
        assert_eq!(ids, expected);
        assert_eq!(page.results[0].content, "message 2");
        assert_eq!(page.results[0].user_id, Some(user));
        assert_eq!(page.results[0].channel_id, channel);
    }

    #[actix_web::test]
    async fn list_messages_returns_an_empty_page_for_an_empty_channel() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;

        let page = list(&ctx, user, channel, None).await;

        assert_eq!(page.count, 0);
        assert!(page.results.is_empty());
        assert_eq!(page.cursor, None);
    }

    #[actix_web::test]
    async fn list_messages_omits_soft_deleted_messages() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        let ids = seed_backlog(&ctx, channel, user, 3).await;

        ctx.soft_delete_message(ids[1]).await;

        let page = list(&ctx, user, channel, None).await;

        let returned: Vec<Uuid> = page.results.iter().map(|m| m.id).collect();
        assert_eq!(returned, vec![ids[0], ids[2]]);
        assert_eq!(page.count, 2);
    }

    #[actix_web::test]
    async fn list_messages_is_scoped_to_the_channel_in_the_path() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        let other = ctx.seed_channel(None, "text", "other").await;

        let mine = seed_backlog(&ctx, channel, user, 2).await;
        seed_backlog(&ctx, other, user, 2).await;

        let page = list(&ctx, user, channel, None).await;

        let returned: Vec<Uuid> = page.results.iter().map(|m| m.id).collect();
        assert_eq!(returned, mine);
    }

    #[actix_web::test]
    async fn list_messages_caps_a_page_at_50_and_returns_a_cursor() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        let all = seed_backlog(&ctx, channel, user, 60).await;

        let page = list(&ctx, user, channel, None).await;

        assert_eq!(page.results.len(), 50, "page size is 50, not the 51 fetched");
        assert_eq!(page.count, 50);

        let returned: Vec<Uuid> = page.results.iter().map(|m| m.id).collect();
        assert_eq!(returned, all[..50], "the 51st row is a lookahead, not data");
        assert!(page.cursor.is_some(), "more rows exist, so a cursor is due");
    }

    #[actix_web::test]
    async fn list_messages_cursor_walks_the_backlog_without_gaps_or_overlap() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        let all = seed_backlog(&ctx, channel, user, 60).await;

        let first = list(&ctx, user, channel, None).await;
        let cursor = first.cursor.clone().expect("first page yields a cursor");

        let second = list(&ctx, user, channel, Some(&cursor)).await;

        assert_eq!(second.results.len(), 10, "60 messages, 50 already seen");
        assert_eq!(second.count, 10);
        assert_eq!(second.cursor, None, "the last page has no cursor");

        let walked: Vec<Uuid> = first
            .results
            .iter()
            .chain(second.results.iter())
            .map(|m| m.id)
            .collect();
        assert_eq!(walked, all, "every message exactly once, newest first");
    }

    /// A cursor that resolves past the oldest message yields an empty page, not
    /// an error and not a wrap-around to the top.
    #[actix_web::test]
    async fn list_messages_cursor_past_the_end_is_empty() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        seed_backlog(&ctx, channel, user, 3).await;

        let cursor = encode_cursor(
            OffsetDateTime::now_utc() - Duration::days(7),
            Uuid::from_u128(0),
        );

        let page = list(&ctx, user, channel, Some(&cursor)).await;

        assert!(page.results.is_empty());
        assert_eq!(page.count, 0);
        assert_eq!(page.cursor, None);
    }

    #[actix_web::test]
    async fn list_messages_rejects_a_malformed_cursor() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;

        let now = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .expect("format timestamp");

        // (case, raw cursor, expected `details[0].code`)
        let cases = [
            ("not base64", "not-valid-base64!!".to_string(), 4001),
            (
                "not utf-8",
                base64::prelude::BASE64_URL_SAFE.encode([0xff, 0xfe, 0xfd]),
                4002,
            ),
            (
                "no comma",
                base64::prelude::BASE64_URL_SAFE.encode(format!("{now}{}", Uuid::now_v7())),
                4003,
            ),
            (
                "timestamp is not rfc3339",
                base64::prelude::BASE64_URL_SAFE.encode(format!("yesterday,{}", Uuid::now_v7())),
                4004,
            ),
            (
                "id is not a uuid",
                base64::prelude::BASE64_URL_SAFE.encode(format!("{now},not-a-uuid")),
                4005,
            ),
        ];

        for (case, cursor, code) in cases {
            let resp = ctx
                .as_user(user)
                .call(
                    test::TestRequest::get()
                        .uri(&format!("/api/v1/channel/{channel}/messages?cursor={cursor}")),
                )
                .await;

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "case: {case}");

            let body: serde_json::Value = test::read_body_json(resp).await;
            assert_eq!(body["target"], "query", "case: {case}");
            assert_eq!(body["details"][0]["target"], "cursor", "case: {case}");
            assert_eq!(body["details"][0]["code"], code, "case: {case}");
        }
    }

    #[actix_web::test]
    async fn list_messages_requires_a_bearer_token() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let channel = ctx.seed_channel(None, "text", "example").await;
        let uri = format!("/api/v1/channel/{channel}/messages");

        let anonymous = test::TestRequest::get().uri(&uri);
        let resp = ctx.authenticated().call(anonymous).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let authorized = test::TestRequest::get()
            .uri(&uri)
            .insert_header((AUTHORIZATION, format!("Bearer {}", ctx.token(user))));
        let resp = ctx.authenticated().call(authorized).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
