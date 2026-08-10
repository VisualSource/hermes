use actix_web::{HttpResponse, Responder, delete, get, http::StatusCode, patch, post, web};
use actix_web_validation::Validated;
use serde::Deserialize;
use sqlx::{SqlitePool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::channel::{Channel, ChannelKind},
    state::{
        api_errors::{ApplicationError, ErrorDetail},
        oauth::jwt::Claims,
        permission::{MANAGE_CHANNELS, VIEW_CHANNELS, channel_permissions, required_permissions},
        socket::session::SessionRegistry,
    },
};

/// The channel id in the path doesn't belong to the server in the path.
///
/// `required_permissions` only proves the caller has rights on *that server*,
/// so every query here must be scoped by `server_id` as well — otherwise owning
/// any server at all would be enough to address another server's channels.
/// Whether the channel exists elsewhere is deliberately not distinguished.
fn channel_not_found() -> ApplicationError {
    ApplicationError::new(
        StatusCode::NOT_FOUND,
        "no such channel in this server",
        "path",
        vec![ErrorDetail::new(
            4004,
            "channel",
            "channel must belong to the server in the path",
        )],
        None,
    )
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct CreateChannelPayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: String,
    #[validate(length(min = 3, max = 255), non_control_character)]
    category: Option<String>,

    kind: ChannelKind,
}

#[utoipa::path(
    tag = "channel",
    description = "create a channel on the given server",
    request_body = CreateChannelPayload,
    responses(
        (status = 200, description = "new channel", body = Channel),
        (status = 400, description = "invalid payload", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing MANAGE_CHANNELS", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server/{server}/channel")]
pub async fn create_channel(
    db: web::Data<SqlitePool>,
    Validated(web::Json(body)): Validated<web::Json<CreateChannelPayload>>,
    claims: web::ReqData<Claims>,
    params: web::Path<Uuid>,
) -> Result<web::Json<Channel>, ApplicationError> {
    let server_id = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, MANAGE_CHANNELS).await?;

    let id = uuid::Uuid::now_v7();
    let channel = query_as!(
        Channel,
        "INSERT INTO channels VALUES (?,?,?,?,?) RETURNING *",
        id,
        body.kind,
        server_id,
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
    description = "update a channel attached to the given server",
    request_body = PatchChannelPayload,
    responses(
        (status = 201, description = "accepted changes"),
        (status = 400, description = "invalid payload", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing MANAGE_CHANNELS", body = ApplicationError),
        (status = 404, description = "no such channel in this server", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[patch("/server/{server}/channel/{channel}")]
pub async fn patch_channel(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    claims: web::ReqData<Claims>,
    Validated(web::Json(body)): Validated<web::Json<PatchChannelPayload>>,
) -> Result<HttpResponse, ApplicationError> {
    let (server_id, channel_id) = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, MANAGE_CHANNELS).await?;

    let result = match (body.name, body.category) {
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
                "UPDATE channels SET name = ?, category = ? WHERE id = ? AND server_id = ?",
                name,
                category,
                channel_id,
                server_id
            )
            .execute(db.get_ref())
            .await?
        }
        (Some(name), None) => {
            query!(
                "UPDATE channels SET name = ? WHERE id = ? AND server_id = ?",
                name,
                &channel_id,
                server_id
            )
            .execute(db.get_ref())
            .await?
        }
        (None, Some(category)) => {
            query!(
                "UPDATE channels SET category = ? WHERE id = ? AND server_id = ?",
                category,
                &channel_id,
                server_id
            )
            .execute(db.get_ref())
            .await?
        }
    };

    if result.rows_affected() == 0 {
        return Err(channel_not_found());
    }

    Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
    tag = "channel",
    description = "fetch a channel attached to the given server",
    responses(
        (status = 200, description = "accepted changes", body = Channel),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing VIEW_CHANNELS", body = ApplicationError),
        (status = 404, description = "no such channel in this server", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/server/{server}/channel/{channel}")]
pub async fn get_channel(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    claims: web::ReqData<Claims>,
) -> Result<web::Json<Channel>, ApplicationError> {
    let (server_id, channel_id) = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, VIEW_CHANNELS).await?;

    let channel = query_as!(
        Channel,
        "SELECT * FROM channels WHERE id = ? AND server_id = ?",
        &channel_id,
        server_id
    )
    .fetch_optional(db.get_ref())
    .await?
    .ok_or_else(channel_not_found)?;

    Ok(web::Json(channel))
}

#[utoipa::path(
    tag = "channel", 
    description = "delete a channel attached to the given server",
    responses(
        (status = 201, description = "accepted deletion"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing MANAGE_CHANNELS", body = ApplicationError),
        (status = 404, description = "no such channel in this server", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/server/{server}/channel/{channel}")]
pub async fn delete_channel(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    let (server_id, channel_id) = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, MANAGE_CHANNELS).await?;

    let result = query!(
        "DELETE FROM channels WHERE id = ? AND server_id = ?",
        &channel_id,
        server_id
    )
    .execute(db.get_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(channel_not_found());
    }

    Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
    tags = ["channel","voice"],
    description = "list active voice channel users",
    responses(
        (status = 200, description = "user list", body = Vec<Uuid>),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing permission", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/channel/{channel}/voice")]
pub async fn list_voice_members(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    claims: web::ReqData<Claims>,
    session: web::Data<SessionRegistry>,
) -> Result<web::Json<Vec<Uuid>>, ApplicationError> {
    let channel_id = params.into_inner();

    let result = channel_permissions(&db, &claims.sub, &channel_id).await?;
    if result.kind != ChannelKind::Voice {
        return Err(ApplicationError::bad_request(
            "invalid channel type",
            "request",
            Vec::default(),
        ));
    }

    let members = session
        .voice_members(&channel_id)
        .expect("failed to get data");

    Ok(web::Json(members))
}
#[cfg(test)]
mod test {
    use actix_web::http::StatusCode;
    use actix_web::test;
    use serde_json::json;

    use crate::test_support::TestCtx;

    use super::*;

    #[actix_web::test]
    async fn create_channel() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/channel"))
            .set_json(json!({"name":"example", "kind":"text" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let channel: Channel = test::read_body_json(resp).await;

        assert_eq!(channel.name, "example");
        assert_eq!(channel.kind, ChannelKind::Text);
        assert_eq!(channel.category, None);
    }

    #[actix_web::test]
    async fn patch_channel_name_only() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_json(json!({ "name":"new-name" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = ?")
            .bind(&channel)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to read channel");

        assert_eq!(result.name, "new-name");
    }

    #[actix_web::test]
    async fn patch_channel_category_only() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_json(json!({ "category": "Example" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = ?")
            .bind(&channel)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to read channel");

        assert_eq!(result.category, Some("Example".to_string()));
    }

    #[actix_web::test]
    async fn patch_channel() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_json(json!({ "category": "Example", "name":"new-name"  }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = ?")
            .bind(&channel)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to read channel");

        assert_eq!(result.category, Some("Example".to_string()));
        assert_eq!(result.name, "new-name");
    }

    #[actix_web::test]
    async fn patch_channel_none() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_json(json!({}));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn get_channel() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_json(json!({"name":"example", "kind":"text" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let channel: Channel = test::read_body_json(resp).await;

        assert_eq!(channel.name, "name");
        assert_eq!(channel.kind, ChannelKind::Text);
        assert_eq!(channel.server_id, Some(server));
        assert_eq!(channel.category, None);
    }

    #[actix_web::test]
    async fn delete_channel() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = ?")
            .bind(&channel)
            .fetch_optional(&ctx.pool)
            .await
            .expect("failed to get channel");

        assert!(result.is_none())
    }

    // -- permission gates --------------------------------------------------
    //
    // Every test above acts as the server owner, who short-circuits to
    // `ALL_PERMS` — so none of them would notice if the `required_permissions`
    // calls disappeared. These drive the gates through a non-owner.

    async fn channel_named(ctx: &TestCtx, channel: Uuid) -> Option<String> {
        sqlx::query_scalar("SELECT name FROM channels WHERE id = ?")
            .bind(channel)
            .fetch_optional(&ctx.pool)
            .await
            .expect("read channel name")
    }

    #[actix_web::test]
    async fn create_channel_requires_manage_channels() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/channel"))
            .set_json(json!({"name":"example", "kind":"text" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let channels: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM channels")
            .fetch_one(&ctx.pool)
            .await
            .expect("count channels");
        assert_eq!(channels, 0);
    }

    /// Creating channels isn't owner-only — a role carrying `MANAGE_CHANNELS`
    /// is enough.
    #[actix_web::test]
    async fn create_channel_accepts_a_role_with_manage_channels() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let user = ctx
            .seed_member_with_role(server, "moderator", MANAGE_CHANNELS)
            .await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/channel"))
            .set_json(json!({"name":"example", "kind":"text" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn patch_channel_requires_manage_channels() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_json(json!({ "name":"new-name" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert_eq!(channel_named(&ctx, channel).await.as_deref(), Some("name"));
    }

    #[actix_web::test]
    async fn delete_channel_requires_manage_channels() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert!(channel_named(&ctx, channel).await.is_some());
    }

    /// `VIEW_CHANNELS` is part of `BASE_PERMS`, so plain membership is enough
    /// to read a channel — no role required.
    #[actix_web::test]
    async fn get_channel_allows_a_plain_member() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// A non-member resolves to zero permissions, so `VIEW_CHANNELS` fails.
    #[actix_web::test]
    async fn get_channel_rejects_a_non_member() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let channel = ctx.seed_channel(Some(server), "text", "name").await;

        let outsider = ctx.seed_user("outsider").await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}/channel/{channel}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(outsider).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // -- server scoping ----------------------------------------------------
    //
    // The permission check reads the server from the path, but the queries
    // address the channel by id alone. Owning *any* server therefore must not
    // become a licence to touch another server's channels.

    #[actix_web::test]
    async fn patch_channel_is_scoped_to_the_server_in_the_path() {
        let ctx = TestCtx::new().await;
        let attacker = ctx.seed_user("attacker").await;
        let own_server = ctx.seed_server(attacker).await;

        let victim = ctx.seed_user("victim").await;
        let other_server = ctx.seed_server(victim).await;
        let foreign = ctx.seed_channel(Some(other_server), "text", "name").await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{own_server}/channel/{foreign}"))
            .set_json(json!({ "name":"pwned" }));

        let resp = ctx.as_user(attacker).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            channel_named(&ctx, foreign).await.as_deref(),
            Some("name"),
            "a channel in another server must not be renamed through this path"
        );
    }

    #[actix_web::test]
    async fn delete_channel_is_scoped_to_the_server_in_the_path() {
        let ctx = TestCtx::new().await;
        let attacker = ctx.seed_user("attacker").await;
        let own_server = ctx.seed_server(attacker).await;

        let victim = ctx.seed_user("victim").await;
        let other_server = ctx.seed_server(victim).await;
        let foreign = ctx.seed_channel(Some(other_server), "text", "name").await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{own_server}/channel/{foreign}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(attacker).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert!(
            channel_named(&ctx, foreign).await.is_some(),
            "a channel in another server must not be deleted through this path"
        );
    }

    #[actix_web::test]
    async fn get_channel_is_scoped_to_the_server_in_the_path() {
        let ctx = TestCtx::new().await;
        let attacker = ctx.seed_user("attacker").await;
        let own_server = ctx.seed_server(attacker).await;

        let victim = ctx.seed_user("victim").await;
        let other_server = ctx.seed_server(victim).await;
        let foreign = ctx.seed_channel(Some(other_server), "text", "secret").await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{own_server}/channel/{foreign}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(attacker).call(req).await;

        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "a channel in another server must not be readable through this path"
        );
    }
}
