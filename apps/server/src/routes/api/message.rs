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
    models::channel::{ChannelKind, Message},
    state::{
        api_errors::{ApplicationError, ErrorDetail, InnerError},
        oauth::jwt::Claims,
        permission::{ChannelScope, MANAGE_MESSAGES, SEND_MESSAGES, channel_permissions},
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

    let channel_id = params.into_inner();

    let result = channel_permissions(&db, &claims.sub, &channel_id).await?;
    if result.kind == ChannelKind::Voice {
        return Err(ApplicationError::bad_request(
            "invalid channel type",
            "request",
            Vec::default(),
        ));
    }
    result.require(SEND_MESSAGES)?;

    let message_id = Uuid::now_v7();
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
        (status = 404, description = "no such message in this channel", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/channel/{channel}/message/{message}")]
pub async fn get_message(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    params: web::Path<(Uuid, Uuid)>,
) -> Result<web::Json<Message>, ApplicationError> {
    let (channel_id, message_id) = params.into_inner();
    let result = channel_permissions(&db, &claims.sub, &channel_id).await?;
    if result.kind == ChannelKind::Voice {
        return Err(ApplicationError::bad_request(
            "invalid channel type",
            "request",
            Vec::default(),
        ));
    }

    let message = query_as!(
        Message,
        "SELECT * FROM messages WHERE id = ? AND channel_id = ?",
        message_id,
        channel_id
    )
    .fetch_optional(db.get_ref())
    .await?
    .ok_or_else(|| ApplicationError::not_found("message"))?;

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
    claims: web::ReqData<Claims>,
    body: web::Json<PatchMessagePayload>,
    params: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, ApplicationError> {
    let (channel_id, message_id) = params.into_inner();

    let result = channel_permissions(&db, &claims.sub, &channel_id).await?;
    if result.kind == ChannelKind::Voice {
        return Err(ApplicationError::bad_request(
            "invalid channel type",
            "request",
            Vec::default(),
        ));
    }

    let edited_at = time::OffsetDateTime::now_utc();

    // Author-only, with no moderator override: rewriting someone else's words
    // is forgery, not moderation, so MANAGE_MESSAGES deliberately buys nothing
    // here. `user_id` is nullable, so a message whose author was deleted has
    // no one who can edit it.
    let r = query!(
        "UPDATE messages SET content = ?, edited_at = ? WHERE id = ? AND channel_id = ? AND user_id = ?",
        body.content,
        edited_at,
        message_id,
        channel_id,
        &claims.sub
    )
    .execute(db.get_ref())
    .await?;

    // Without this the route answers 202 for a message that doesn't exist, or
    // that the caller doesn't own — reporting an edit that never happened.
    if r.rows_affected() != 1 {
        return Err(ApplicationError::not_found("message"));
    }

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
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, ApplicationError> {
    let (channel_id, message_id) = params.into_inner();

    let result = channel_permissions(&db, &claims.sub, &channel_id).await?;
    if result.kind == ChannelKind::Voice {
        return Err(ApplicationError::bad_request(
            "invalid channel type",
            "request",
            Vec::default(),
        ));
    }
    let deleted_at = time::OffsetDateTime::now_utc();

    // Authors may always delete their own message; removing anyone else's is
    // moderation and takes MANAGE_MESSAGES. A bare `require(MANAGE_MESSAGES)`
    // can't express that from either side — it would refuse a plain member
    // their own message, and in a DM it always passes because no roles exist
    // there. So the moderator case widens the filter rather than gating it.
    let can_moderate = matches!(
        result.scope,
        ChannelScope::Server { perms, .. } if perms & MANAGE_MESSAGES == MANAGE_MESSAGES
    );

    let r = if can_moderate {
        query!(
            "UPDATE messages SET deleted_at = ? WHERE id = ? AND channel_id = ?",
            deleted_at,
            message_id,
            channel_id,
        )
    } else {
        query!(
            "UPDATE messages SET deleted_at = ? WHERE id = ? AND channel_id = ? AND user_id = ?",
            deleted_at,
            message_id,
            channel_id,
            &claims.sub
        )
    }
    .execute(db.get_ref())
    .await?;

    if r.rows_affected() != 1 {
        return Err(ApplicationError::not_found("message"));
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
    description = "fetch a cursor paginated list of messages sorted by newest",
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
    claims: web::ReqData<Claims>,
) -> Result<web::Json<MessageQueryResult>, ApplicationError> {
    // validate user can fetch messages from this channel
    let channel_id = params.into_inner();

    let result = channel_permissions(&db, &claims.sub, &channel_id).await?;
    if result.kind == ChannelKind::Voice {
        return Err(ApplicationError::bad_request(
            "invalid channel type",
            "request",
            Vec::default(),
        ));
    }

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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;

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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;

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

    /// Unknown id and "exists, but in another channel" are the same 404 — the
    /// query is scoped by `channel_id`, and both used to be a 500.
    #[actix_web::test]
    async fn get_message_for_an_unknown_or_foreign_message_is_not_found() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
        let other_channel = ctx.seed_channel(Some(_server), "text", "other").await;

        let elsewhere = ctx.seed_message(other_channel, user, "not here").await;

        for message in [Uuid::now_v7(), elsewhere] {
            let req = test::TestRequest::get()
                .uri(&format!("/api/v1/channel/{channel}/message/{message}"))
                .set_payload(Vec::default());

            let resp = ctx.as_user(user).call(req).await;

            assert_eq!(resp.status(), StatusCode::NOT_FOUND, "message={message}");
        }
    }

    #[actix_web::test]
    async fn patch_message() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;

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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;

        let page = list(&ctx, user, channel, None).await;

        assert_eq!(page.count, 0);
        assert!(page.results.is_empty());
        assert_eq!(page.cursor, None);
    }

    #[actix_web::test]
    async fn list_messages_omits_soft_deleted_messages() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
        let other = ctx.seed_channel(Some(_server), "text", "other").await;

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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
        let all = seed_backlog(&ctx, channel, user, 60).await;

        let page = list(&ctx, user, channel, None).await;

        assert_eq!(
            page.results.len(),
            50,
            "page size is 50, not the 51 fetched"
        );
        assert_eq!(page.count, 50);

        let returned: Vec<Uuid> = page.results.iter().map(|m| m.id).collect();
        assert_eq!(returned, all[..50], "the 51st row is a lookahead, not data");
        assert!(page.cursor.is_some(), "more rows exist, so a cursor is due");
    }

    #[actix_web::test]
    async fn list_messages_cursor_walks_the_backlog_without_gaps_or_overlap() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;

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
                .call(test::TestRequest::get().uri(&format!(
                    "/api/v1/channel/{channel}/messages?cursor={cursor}"
                )))
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
        let (_server, channel) = ctx.seed_text_channel_for(user).await;
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

    // -- channel access ----------------------------------------------------
    //
    // Every route resolves the channel through `channel_permissions` before it
    // touches a message. These drive that resolver from the outside: an
    // outsider on a server channel, a stranger on a DM, and channel ids that
    // belong to nobody.

    /// One request per route, so a gate missing from a single handler shows up
    /// instead of hiding behind its neighbours. Returns `(label, status)`.
    async fn all_routes(
        ctx: &TestCtx,
        user: Uuid,
        channel: Uuid,
        message: Uuid,
    ) -> Vec<(&'static str, StatusCode)> {
        let calls: Vec<(&'static str, test::TestRequest)> = vec![
            (
                "list",
                test::TestRequest::get().uri(&format!("/api/v1/channel/{channel}/messages")),
            ),
            (
                "get",
                test::TestRequest::get()
                    .uri(&format!("/api/v1/channel/{channel}/message/{message}")),
            ),
            (
                "create",
                test::TestRequest::post()
                    .uri(&format!("/api/v1/channel/{channel}/message"))
                    .set_json(json!({ "content": "intruding" })),
            ),
            (
                "patch",
                test::TestRequest::patch()
                    .uri(&format!("/api/v1/channel/{channel}/message/{message}"))
                    .set_json(json!({ "content": "rewritten" })),
            ),
            (
                "delete",
                test::TestRequest::delete()
                    .uri(&format!("/api/v1/channel/{channel}/message/{message}")),
            ),
        ];

        let mut out = Vec::new();
        for (label, req) in calls {
            out.push((label, ctx.as_user(user).call(req).await.status()));
        }
        out
    }

    /// A non-member resolves to zero permissions, so `VIEW_CHANNELS` fails and
    /// every route refuses before it can read or write a message.
    #[actix_web::test]
    async fn message_routes_reject_a_non_member_of_the_server() {
        let ctx = TestCtx::new().await;
        let author = ctx.seed_user("author").await;
        let (_server, channel) = ctx.seed_text_channel_for(author).await;
        let message = ctx.seed_message(channel, author, "members only").await;

        let outsider = ctx.seed_user("outsider").await;

        for (label, status) in all_routes(&ctx, outsider, channel, message).await {
            assert_eq!(status, StatusCode::FORBIDDEN, "route={label}");
        }

        assert_eq!(content_of(&ctx, message).await, "members only");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE channel_id = ?")
            .bind(channel)
            .fetch_one(&ctx.pool)
            .await
            .expect("count messages");
        assert_eq!(count, 1, "the outsider must not have posted");
    }

    /// A DM has no server and no roles, so the only thing guarding it is the
    /// `dm_participants` row. This is the check that makes a leaked DM channel
    /// id useless to anyone outside the conversation.
    #[actix_web::test]
    async fn message_routes_reject_a_stranger_to_a_dm() {
        let ctx = TestCtx::new().await;
        let alice = ctx.seed_user("alice").await;
        let bob = ctx.seed_user("bob").await;
        let channel = ctx.seed_dm_channel(alice, bob).await;
        let message = ctx.seed_message(channel, alice, "private").await;

        let stranger = ctx.seed_user("stranger").await;

        for (label, status) in all_routes(&ctx, stranger, channel, message).await {
            assert_eq!(status, StatusCode::NOT_FOUND, "route={label}");
        }

        assert_eq!(content_of(&ctx, message).await, "private");
    }

    #[actix_web::test]
    async fn message_routes_reject_an_unknown_channel() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;

        for (label, status) in all_routes(&ctx, user, Uuid::now_v7(), Uuid::now_v7()).await {
            assert_eq!(status, StatusCode::NOT_FOUND, "route={label}");
        }
    }

    /// A channel with no server *and* no participants belongs to nobody.
    #[actix_web::test]
    async fn message_routes_reject_an_orphan_channel() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;
        let orphan = ctx.seed_channel(None, "text", "orphan").await;

        for (label, status) in all_routes(&ctx, user, orphan, Uuid::now_v7()).await {
            assert_eq!(status, StatusCode::NOT_FOUND, "route={label}");
        }
    }

    #[actix_web::test]
    async fn message_routes_reject_a_voice_channel() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let user = ctx.seed_user("user").await;
        ctx.seed_member(server, user).await;

        let voice = ctx.seed_channel(Some(server), "voice", "general").await;
        let message = ctx.seed_message(voice, user, "should not be here").await;

        for (label, status) in all_routes(&ctx, user, voice, message).await {
            assert_eq!(status, StatusCode::BAD_REQUEST, "route={label}");
        }
    }

    // -- authorship --------------------------------------------------------

    async fn patch_as(ctx: &TestCtx, user: Uuid, channel: Uuid, message: Uuid) -> StatusCode {
        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/channel/{channel}/message/{message}"))
            .set_json(json!({ "content": "rewritten" }));

        ctx.as_user(user).call(req).await.status()
    }

    async fn delete_as(ctx: &TestCtx, user: Uuid, channel: Uuid, message: Uuid) -> StatusCode {
        let req =
            test::TestRequest::delete().uri(&format!("/api/v1/channel/{channel}/message/{message}"));

        ctx.as_user(user).call(req).await.status()
    }

    async fn content_of(ctx: &TestCtx, message: Uuid) -> String {
        sqlx::query_scalar("SELECT content FROM messages WHERE id = ?")
            .bind(message)
            .fetch_one(&ctx.pool)
            .await
            .expect("read content")
    }

    async fn is_deleted(ctx: &TestCtx, message: Uuid) -> bool {
        sqlx::query_scalar::<_, Option<OffsetDateTime>>(
            "SELECT deleted_at FROM messages WHERE id = ?",
        )
        .bind(message)
        .fetch_one(&ctx.pool)
        .await
        .expect("read deleted_at")
        .is_some()
    }

    /// Editing is author-only with no moderator override — rewriting someone
    /// else's words is forgery, not moderation. `MANAGE_MESSAGES` buys the
    /// power to *delete*, and this pins that it buys nothing here.
    #[actix_web::test]
    async fn patch_message_is_author_only_even_for_a_moderator() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let channel = ctx.seed_channel(Some(server), "text", "example").await;

        let author = ctx.seed_user("author").await;
        ctx.seed_member(server, author).await;
        let message = ctx.seed_message(channel, author, "original").await;

        let moderator = ctx
            .seed_member_with_role(server, "moderator", MANAGE_MESSAGES)
            .await;

        assert_eq!(
            patch_as(&ctx, moderator, channel, message).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(content_of(&ctx, message).await, "original");

        // The owner holds ALL_PERMS and still can't rewrite it.
        assert_eq!(
            patch_as(&ctx, owner, channel, message).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(content_of(&ctx, message).await, "original");

        // The author can.
        assert_eq!(
            patch_as(&ctx, author, channel, message).await,
            StatusCode::ACCEPTED
        );
        assert_eq!(content_of(&ctx, message).await, "rewritten");
    }

    /// Previously this answered 202 while changing nothing.
    #[actix_web::test]
    async fn patch_message_for_an_unknown_message_is_not_found() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;
        let (_server, channel) = ctx.seed_text_channel_for(user).await;

        assert_eq!(
            patch_as(&ctx, user, channel, Uuid::now_v7()).await,
            StatusCode::NOT_FOUND
        );
    }

    /// A plain member holds `BASE_PERMS` and no `MANAGE_MESSAGES`, and must
    /// still be able to take down what they wrote.
    #[actix_web::test]
    async fn delete_message_allows_the_author_without_manage_messages() {
        let ctx = TestCtx::new().await;
        let author = ctx.seed_user("author").await;
        let (_server, channel) = ctx.seed_text_channel_for(author).await;
        let message = ctx.seed_message(channel, author, "mine").await;

        assert_eq!(
            delete_as(&ctx, author, channel, message).await,
            StatusCode::ACCEPTED
        );
        assert!(is_deleted(&ctx, message).await);
    }

    #[actix_web::test]
    async fn delete_message_allows_a_moderator_to_remove_someone_elses() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let channel = ctx.seed_channel(Some(server), "text", "example").await;

        let author = ctx.seed_user("author").await;
        ctx.seed_member(server, author).await;
        let message = ctx.seed_message(channel, author, "spam").await;

        let moderator = ctx
            .seed_member_with_role(server, "moderator", MANAGE_MESSAGES)
            .await;

        assert_eq!(
            delete_as(&ctx, moderator, channel, message).await,
            StatusCode::ACCEPTED
        );
        assert!(is_deleted(&ctx, message).await);
    }

    /// Without `MANAGE_MESSAGES` the filter narrows to the caller's own rows,
    /// so someone else's message simply doesn't match.
    #[actix_web::test]
    async fn delete_message_refuses_a_plain_member_someone_elses() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let channel = ctx.seed_channel(Some(server), "text", "example").await;

        let author = ctx.seed_user("author").await;
        ctx.seed_member(server, author).await;
        let message = ctx.seed_message(channel, author, "not yours").await;

        let bystander = ctx.seed_user("bystander").await;
        ctx.seed_member(server, bystander).await;

        assert_eq!(
            delete_as(&ctx, bystander, channel, message).await,
            StatusCode::NOT_FOUND
        );
        assert!(!is_deleted(&ctx, message).await);
    }

    // -- dm participants ---------------------------------------------------

    /// A DM has no roles, so `require` passes unconditionally there. Both
    /// participants can read and post, but each may only edit and delete their
    /// own — there is no moderator path to widen it.
    #[actix_web::test]
    async fn dm_participants_can_read_and_post_but_only_touch_their_own() {
        let ctx = TestCtx::new().await;
        let alice = ctx.seed_user("alice").await;
        let bob = ctx.seed_user("bob").await;
        let channel = ctx.seed_dm_channel(alice, bob).await;

        let from_alice = ctx.seed_message(channel, alice, "hi bob").await;

        let req = test::TestRequest::get().uri(&format!("/api/v1/channel/{channel}/messages"));
        assert_eq!(
            ctx.as_user(bob).call(req).await.status(),
            StatusCode::OK,
            "a participant can read the conversation"
        );

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/channel/{channel}/message"))
            .set_json(json!({ "content": "hi alice" }));
        assert_eq!(ctx.as_user(bob).call(req).await.status(), StatusCode::OK);

        assert_eq!(
            patch_as(&ctx, bob, channel, from_alice).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(content_of(&ctx, from_alice).await, "hi bob");

        assert_eq!(
            delete_as(&ctx, bob, channel, from_alice).await,
            StatusCode::NOT_FOUND
        );
        assert!(!is_deleted(&ctx, from_alice).await);

        assert_eq!(
            delete_as(&ctx, alice, channel, from_alice).await,
            StatusCode::ACCEPTED
        );
        assert!(is_deleted(&ctx, from_alice).await);
    }
}
