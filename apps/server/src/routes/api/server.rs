use actix_web::{HttpResponse, Responder, delete, get, http::StatusCode, patch, post, web};
use actix_web_validation::Validated;
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool, query, query_as, query_scalar};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        channel::Channel,
        invite::Invite,
        server::{Server, ServerMember},
    },
    state::{
        api_errors::{ApplicationError, ErrorDetail},
        oauth::jwt::Claims,
        permission::{MANAGE_SERVER, MANAGE_USERS, required_permissions},
    },
};

#[utoipa::path(
    tag = "server", 
    description = "fetch a server",
    responses(
        (status = 200, description = "single server info", body = Server),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 404, description = "no such server", body = ApplicationError),
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
        .fetch_optional(db.get_ref())
        .await?
        .ok_or_else(|| ApplicationError::not_found("server"))?;

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

    required_permissions(&db, &user.sub, &server_id, MANAGE_SERVER).await?;

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

    let mut tx = db.begin().await?;

    let server = query_as!(
        Server,
        "INSERT INTO servers VALUES (?,?,?,?,?) RETURNING *",
        server_id,
        body.name,
        user.sub,
        now,
        body.icon
    )
    .fetch_one(&mut *tx)
    .await?;

    let member_id = Uuid::now_v7();

    query!(
        "INSERT INTO server_members VALUES (?,?,?)",
        member_id,
        server_id,
        user.sub
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

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
    claims: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    if body.name.is_none() || body.icon.is_none() {
        return Err(ApplicationError::new(
            StatusCode::BAD_REQUEST,
            "bad request",
            "body",
            vec![ErrorDetail::new(4001, "body", "no values to update")],
            None,
        ));
    }
    let server_id = server.into_inner();
    required_permissions(&db, &claims.sub, &server_id, MANAGE_SERVER).await?;

    let mut builder = QueryBuilder::<Sqlite>::new("UPDATE servers SET ");
    let mut separated = builder.separated(", ");

    if let Some(name) = body.name {
        separated.push("name = ").push_bind_unseparated(name);
    }
    if let Some(icon) = body.icon {
        separated.push("icon = ").push_bind_unseparated(icon);
    }

    builder.push("WHERE id = ").push_bind(server_id);

    let query = builder.build();

    query.execute(db.get_ref()).await?;

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
    claims: web::ReqData<Claims>,
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
    cliams: web::ReqData<Claims>,
) -> Result<web::Json<Vec<Channel>>, ApplicationError> {
    let server_id = params.into_inner();

    //TODO: fetch channels that user can see based on roles

    let channels = query_as!(
        Channel,
        "SELECT * FROM channels WHERE server_id = ?",
        &server_id
    )
    .fetch_all(db.get_ref())
    .await?;

    Ok(web::Json(channels))
}

#[utoipa::path(
    tags = ["server","users"],
    responses(
        (status = 204, description = "left server"),
        (status = 400, description = "owner can not leave their own server", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 404, description = "no such server", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server/{server}/leave")]
pub async fn leave_server(
    db: web::Data<SqlitePool>,
    params: web::Path<uuid::Uuid>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, ApplicationError> {
    let server_id = params.into_inner();

    remove_user_from_server(&db, claims.sub, server_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, ToSchema)]
struct PostKickUserPayload {
    user: Uuid,
}

#[utoipa::path(
    tags = ["server","users"],
    request_body = PostKickUserPayload,
    responses(
        (status = 204, description = "kicked user from server"),
        (status = 400, description = "owner can not be kicked", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing MANAGE_USERS", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server/{server}/kick")]
pub async fn kick_user_from_server(
    db: web::Data<SqlitePool>,
    params: web::Path<uuid::Uuid>,
    claims: web::ReqData<Claims>,
    body: web::Json<PostKickUserPayload>,
) -> Result<HttpResponse, ApplicationError> {
    let server_id = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, MANAGE_USERS).await?;

    remove_user_from_server(&db, body.user, server_id).await?;

    return Ok(HttpResponse::NoContent().finish());
}

async fn remove_user_from_server(
    db: &SqlitePool,
    user: Uuid,
    server_id: Uuid,
) -> Result<(), ApplicationError> {
    let mut tx = db.begin().await?;

    let owner_id = query_scalar!("SELECT owner_id FROM servers WHERE id = ?", &server_id)
        .fetch_optional(&mut *tx)
        .await?;

    let Some(owner_id) = owner_id else {
        tx.rollback().await?;
        return Err(ApplicationError::not_found("server"));
    };

    if owner_id == user {
        tx.rollback().await?;
        return Err(ApplicationError::new(
            StatusCode::BAD_REQUEST,
            "can not remove owner from server",
            "request",
            vec![ErrorDetail::new(4000, "user", "invalid user id")],
            None,
        ));
    }

    query!("DELETE FROM role_members WHERE member_id = ? AND role_id IN (SELECT id FROM roles WHERE server_id = ?)", &user, &server_id)
        .execute(&mut *tx)
        .await?;
    query!(
        "DELETE FROM server_members WHERE user_id = ? AND server_id = ?",
        user,
        server_id
    )
    .execute(&mut *tx)
    .await?;

    query!(
        "DELETE FROM invites WHERE created_by = ? AND server_id = ?",
        user,
        &server_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
struct PostJoinServerPayload {
    #[validate(length(equal = 21))]
    invite_code: String,
}

/// Revoked, expired, and spent are one answer to the caller: the code is real
/// but no longer usable. Distinct from the 404 an unknown code gets.
fn invite_no_longer_usable() -> ApplicationError {
    ApplicationError::new(
        StatusCode::GONE,
        "invite was revoked, is at max uses, or has expired",
        "invite",
        Vec::default(),
        None,
    )
}

#[utoipa::path(
    tags = ["server","invite"],
    description = "join a server by redeeming an invite code",
    request_body = PostJoinServerPayload,
    responses(
        (status = 202, description = "joined the server, or was already a member"),
        (status = 400, description = "malformed invite code", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 404, description = "no such invite", body = ApplicationError),
        (status = 410, description = "invite has been revoked, expired, or is at max uses", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server/join")]
pub async fn join_server(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    Validated(web::Json(body)): Validated<web::Json<PostJoinServerPayload>>,
) -> Result<HttpResponse, ApplicationError> {
    let result = query_as!(Invite, "SELECT * FROM invites WHERE id = ?", &body.invite_code)
        .fetch_optional(db.get_ref())
        .await?
        .ok_or_else(|| ApplicationError::not_found("invite"))?;

    if result.revoked
        || result.uses >= result.max_uses
        || result
            .expires_at
            .map(|exp| exp < time::UtcDateTime::now())
            .unwrap_or_default()
    {
        return Err(invite_no_longer_usable());
    }

    let mut tx = db.begin().await?;

    // Joining is idempotent. `server_members` has only a non-unique index on
    // (server_id, user_id), so nothing below the handler stops a repeat join
    // from creating a second row — and since `role_members.member_id` points at
    // `server_members.id`, a duplicate splits one member's identity in two and
    // leaves role lookups free to pick either half.
    let already_member = query_scalar!(
        "SELECT COUNT(*) FROM server_members WHERE server_id = ? AND user_id = ?",
        result.server_id,
        claims.sub
    )
    .fetch_one(&mut *tx)
    .await?
        > 0;

    if already_member {
        tx.rollback().await?;
        return Ok(HttpResponse::Accepted().finish());
    }

    // Spend the use with a *relative* increment guarded by the same conditions
    // checked above. The pre-flight read happens outside this transaction, so
    // two joins racing on the last seat would otherwise both see `uses = 0` and
    // both write `1` — overshooting `max_uses` while the counter says otherwise.
    // No rows updated means someone else took the seat first.
    let spent = query!(
        "UPDATE invites SET uses = uses + 1 WHERE id = ? AND NOT revoked AND uses < max_uses",
        result.id
    )
    .execute(&mut *tx)
    .await?;

    if spent.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(invite_no_longer_usable());
    }

    query!(
        "INSERT INTO server_members (id,server_id,user_id) VALUES (?,?,?)",
        Uuid::now_v7(),
        result.server_id,
        &claims.sub
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Accepted().finish())
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;
    use nanoid::nanoid;
    use serde_json::json;
    use time::OffsetDateTime;

    use crate::state::permission::{BASE_PERMS, effective_permissions};
    use crate::test_support::{InviteState, TestCtx};

    use super::*;

    #[actix_web::test]
    pub async fn get_server() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let srv: Server = test::read_body_json(resp).await;

        assert_eq!(srv.owner_id, user);
        assert_eq!(srv.name, "test-server");
    }

    #[actix_web::test]
    pub async fn delete_server() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let server_r = sqlx::query("SELECT * FROM servers WHERE id = ?")
            .bind(server)
            .fetch_optional(&ctx.pool)
            .await
            .expect("failed to query");

        assert!(server_r.is_none());
    }

    #[actix_web::test]
    pub async fn create_server() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server"))
            .set_json(json!({"name":"example-server", "icon": "data:image/jpeg;base64,"}));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let srv: Server = test::read_body_json(resp).await;

        assert_eq!(srv.owner_id, user);
        assert_eq!(srv.name, "example-server");

        assert_eq!(srv.icon, Some("data:image/jpeg;base64,".to_string()));
    }

    #[actix_web::test]
    pub async fn patch_server() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}"))
            .set_json(json!({"name":"example-server", "icon": "data:image/jpeg;base64,AAAA"}));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let srv = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
            .bind(server)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to query");

        assert_eq!(srv.owner_id, user);
        assert_eq!(srv.name, "example-server");

        assert_eq!(srv.icon, Some("data:image/jpeg;base64,AAAA".to_string()));
    }

    #[actix_web::test]
    pub async fn list_servers() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server_a = ctx.seed_server(owner).await;
        let server_b = ctx.seed_server(owner).await;

        let user = ctx.seed_user("test").await;
        ctx.seed_member(server_a, user).await;
        ctx.seed_member(server_b, user).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/servers"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let srv: Vec<Server> = test::read_body_json(resp).await;

        assert_eq!(srv.len(), 2);
    }
    #[actix_web::test]
    pub async fn list_server_members() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let user = ctx.seed_user("test").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}/members"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let srv: Vec<ServerMember> = test::read_body_json(resp).await;

        assert_eq!(srv.len(), 2);
    }
    #[actix_web::test]
    pub async fn list_server_channels() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        ctx.seed_channel(Some(server), "text", "a").await;
        ctx.seed_channel(Some(server), "voice", "b").await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}/channels"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(owner).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let srv: Vec<Channel> = test::read_body_json(resp).await;

        assert_eq!(srv.len(), 2);
    }

    /// An unknown id is a 404, not the 500 `fetch_one` produced by surfacing
    /// `RowNotFound` through the `sqlx::Error` conversion.
    #[actix_web::test]
    pub async fn get_server_for_an_unknown_id_is_not_found() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{}", Uuid::now_v7()))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // -- join --------------------------------------------------------------

    async fn join(ctx: &TestCtx, user: Uuid, code: &str) -> StatusCode {
        let req = test::TestRequest::post()
            .uri("/api/v1/server/join")
            .set_json(json!({ "invite_code": code }));

        ctx.as_user(user).call(req).await.status()
    }

    async fn member_count(ctx: &TestCtx, server: Uuid, user: Uuid) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM server_members WHERE server_id = ? AND user_id = ?",
        )
        .bind(server)
        .bind(user)
        .fetch_one(&ctx.pool)
        .await
        .expect("count server members")
    }

    #[actix_web::test]
    pub async fn join_server_adds_the_caller_and_spends_a_use() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let code = nanoid!();
        ctx.seed_invite(&code, server, owner).await;

        let joiner = ctx.seed_user("joiner").await;

        assert_eq!(join(&ctx, joiner, &code).await, StatusCode::ACCEPTED);

        assert_eq!(member_count(&ctx, server, joiner).await, 1);
        assert_eq!(ctx.invite_uses(&code).await, 1);
    }

    /// A joiner is a plain member: `BASE_PERMS` only, no roles.
    #[actix_web::test]
    pub async fn join_server_grants_only_base_permissions() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let code = nanoid!();
        ctx.seed_invite(&code, server, owner).await;

        let joiner = ctx.seed_user("joiner").await;
        assert_eq!(join(&ctx, joiner, &code).await, StatusCode::ACCEPTED);

        let perms = effective_permissions(&ctx.pool, &joiner, &server)
            .await
            .expect("effective permissions");

        assert_eq!(perms, BASE_PERMS);
    }

    #[actix_web::test]
    pub async fn join_server_with_an_unknown_code_is_not_found() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;

        assert_eq!(join(&ctx, user, &nanoid!()).await, StatusCode::NOT_FOUND);
    }

    /// Revoked, expired, and spent all mean the same thing to a caller: the
    /// invite is no longer usable. None of them may add a member or move the
    /// use counter.
    #[actix_web::test]
    pub async fn join_server_refuses_a_dead_invite() {
        let cases: Vec<(&str, InviteState)> = vec![
            (
                "revoked",
                InviteState {
                    revoked: true,
                    ..Default::default()
                },
            ),
            (
                "expired",
                InviteState {
                    expires_at: Some(OffsetDateTime::now_utc() - time::Duration::hours(1)),
                    ..Default::default()
                },
            ),
            (
                "spent",
                InviteState {
                    max_uses: 2,
                    uses: 2,
                    ..Default::default()
                },
            ),
        ];

        for (label, state) in cases {
            let ctx = TestCtx::new().await;
            let owner = ctx.seed_user("owner").await;
            let server = ctx.seed_server(owner).await;

            let code = nanoid!();
            let before = state.uses;
            ctx.seed_invite_with(&code, server, owner, state).await;

            let joiner = ctx.seed_user("joiner").await;

            assert_eq!(join(&ctx, joiner, &code).await, StatusCode::GONE, "{label}");
            assert_eq!(member_count(&ctx, server, joiner).await, 0, "{label}");
            assert_eq!(ctx.invite_uses(&code).await, before, "{label}");
        }
    }

    /// A null `expires_at` means the invite never expires, so it must still work.
    #[actix_web::test]
    pub async fn join_server_accepts_an_invite_that_never_expires() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let code = nanoid!();
        ctx.seed_invite_with(
            &code,
            server,
            owner,
            InviteState {
                expires_at: None,
                ..Default::default()
            },
        )
        .await;

        let joiner = ctx.seed_user("joiner").await;

        assert_eq!(join(&ctx, joiner, &code).await, StatusCode::ACCEPTED);
        assert_eq!(member_count(&ctx, server, joiner).await, 1);
    }

    /// `max_uses` is the whole point of the counter: the last permitted join
    /// succeeds and the next one is turned away.
    #[actix_web::test]
    pub async fn join_server_exhausts_an_invite_at_max_uses() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let code = nanoid!();
        ctx.seed_invite_with(
            &code,
            server,
            owner,
            InviteState {
                max_uses: 1,
                ..Default::default()
            },
        )
        .await;

        let first = ctx.seed_user("first").await;
        let second = ctx.seed_user("second").await;

        assert_eq!(join(&ctx, first, &code).await, StatusCode::ACCEPTED);
        assert_eq!(ctx.invite_uses(&code).await, 1);

        assert_eq!(join(&ctx, second, &code).await, StatusCode::GONE);
        assert_eq!(member_count(&ctx, server, second).await, 0);
        assert_eq!(ctx.invite_uses(&code).await, 1, "a refused join spends nothing");
    }

    /// Joining twice must not produce two `server_members` rows for one user.
    /// `role_members.member_id` points at `server_members.id`, so a duplicate
    /// splits one member's identity in two and lets role lookups pick either.
    #[actix_web::test]
    pub async fn join_server_twice_does_not_duplicate_membership() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let code = nanoid!();
        ctx.seed_invite_with(
            &code,
            server,
            owner,
            InviteState {
                max_uses: 10,
                ..Default::default()
            },
        )
        .await;

        let joiner = ctx.seed_user("joiner").await;

        assert_eq!(join(&ctx, joiner, &code).await, StatusCode::ACCEPTED);
        let after_first = ctx.invite_uses(&code).await;

        join(&ctx, joiner, &code).await;

        assert_eq!(
            member_count(&ctx, server, joiner).await,
            1,
            "a second join must not add a second membership row"
        );
        assert_eq!(
            ctx.invite_uses(&code).await,
            after_first,
            "re-joining must not burn another use"
        );
    }

    /// The owner is already a member; redeeming their own invite must not add
    /// them a second time.
    #[actix_web::test]
    pub async fn join_server_by_an_existing_member_is_a_no_op() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let code = nanoid!();
        ctx.seed_invite(&code, server, owner).await;

        join(&ctx, owner, &code).await;

        assert_eq!(member_count(&ctx, server, owner).await, 1);
    }

    /// `invite_code` is a 21-char nanoid, and the payload is `Validated`, so a
    /// wrong-length code is turned away before the lookup runs.
    #[actix_web::test]
    pub async fn join_server_rejects_a_malformed_code() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;

        for code in ["", "too-short", &"x".repeat(22)] {
            assert_eq!(
                join(&ctx, user, code).await,
                StatusCode::BAD_REQUEST,
                "code={code:?}"
            );
        }
    }

    /// A user who leaves can come back through a fresh invite — the no-op guard
    /// keys on current membership, not on ever having been a member.
    #[actix_web::test]
    pub async fn join_server_after_leaving_re_adds_the_member() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let code = nanoid!();
        ctx.seed_invite_with(
            &code,
            server,
            owner,
            InviteState {
                max_uses: 10,
                ..Default::default()
            },
        )
        .await;

        let joiner = ctx.seed_user("joiner").await;
        assert_eq!(join(&ctx, joiner, &code).await, StatusCode::ACCEPTED);

        let leave = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/leave"))
            .set_payload(Vec::default());
        assert_eq!(
            ctx.as_user(joiner).call(leave).await.status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(member_count(&ctx, server, joiner).await, 0);

        assert_eq!(join(&ctx, joiner, &code).await, StatusCode::ACCEPTED);
        assert_eq!(member_count(&ctx, server, joiner).await, 1);
        assert_eq!(ctx.invite_uses(&code).await, 2, "the return trip spends a use");
    }

    // -- leave / kick ------------------------------------------------------

    async fn is_member(ctx: &TestCtx, server: Uuid, user: Uuid) -> bool {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM server_members WHERE server_id = ? AND user_id = ?",
        )
        .bind(server)
        .bind(user)
        .fetch_one(&ctx.pool)
        .await
        .expect("count server members");

        count > 0
    }

    async fn count_role_members(ctx: &TestCtx, member: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM role_members WHERE member_id = ?")
            .bind(member)
            .fetch_one(&ctx.pool)
            .await
            .expect("count role members")
    }

    /// Leaving is self-service: the route removes the *caller*, so it carries
    /// no permission gate on purpose. This member holds only a mask-0 role
    /// (i.e. nothing beyond `BASE_PERMS`) — if a `required_permissions` check
    /// ever gets added here, it would lock ordinary members into the server and
    /// this test is what catches it.
    #[actix_web::test]
    pub async fn leave_server_drops_membership_and_role_links() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let user = ctx.seed_user("test").await;
        let member = ctx.seed_member(server, user).await;
        let role = ctx.seed_role(server, "example").await;
        ctx.seed_role_member(role, member).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/leave"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        assert!(!is_member(&ctx, server, user).await);
        assert_eq!(count_role_members(&ctx, member).await, 0);
        assert!(
            is_member(&ctx, server, owner).await,
            "leaving must not touch anyone else's membership"
        );
    }

    /// `servers.owner_id` is `ON DELETE RESTRICT`, and an ownerless server has
    /// no way back — so the owner is refused before anything is deleted.
    #[actix_web::test]
    pub async fn leave_server_rejects_the_owner() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/leave"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(owner).call(req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert!(is_member(&ctx, server, owner).await);
    }

    /// `leave_server` has no permission gate in front of it — leaving is
    /// self-service — so it's the one path that reaches the owner lookup with
    /// an arbitrary server id. That used to be a 500.
    #[actix_web::test]
    pub async fn leave_server_for_an_unknown_server_is_not_found() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{}/leave", Uuid::now_v7()))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    /// Invites the leaver created here go with them; their invites to other
    /// servers, and other people's invites here, must survive.
    #[actix_web::test]
    pub async fn leave_server_revokes_only_the_leavers_invites_for_that_server() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let other_server = ctx.seed_server(owner).await;

        let user = ctx.seed_user("test").await;
        ctx.seed_member(server, user).await;
        ctx.seed_member(other_server, user).await;

        ctx.seed_invite("aaaaaaaaaaaaaaaaaaaaa", server, user).await;
        ctx.seed_invite("bbbbbbbbbbbbbbbbbbbbb", other_server, user)
            .await;
        ctx.seed_invite("ccccccccccccccccccccc", server, owner)
            .await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/leave"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let surviving: Vec<String> = sqlx::query_scalar("SELECT id FROM invites ORDER BY id")
            .fetch_all(&ctx.pool)
            .await
            .expect("list invites");

        assert_eq!(
            surviving,
            vec![
                "bbbbbbbbbbbbbbbbbbbbb".to_string(),
                "ccccccccccccccccccccc".to_string()
            ]
        );
    }

    #[actix_web::test]
    pub async fn kick_user_from_server_removes_the_target() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let target = ctx.seed_user("target").await;
        let member = ctx.seed_member(server, target).await;
        let role = ctx.seed_role(server, "example").await;
        ctx.seed_role_member(role, member).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/kick"))
            .set_json(json!({ "user": target }));

        let resp = ctx.as_user(owner).call(req).await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        assert!(!is_member(&ctx, server, target).await);
        assert_eq!(count_role_members(&ctx, member).await, 0);
    }

    /// A role carrying `MANAGE_USERS` is enough — kicking isn't owner-only.
    #[actix_web::test]
    pub async fn kick_user_from_server_accepts_a_role_with_manage_users() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let moderator = ctx.seed_user("moderator").await;
        let moderator_member = ctx.seed_member(server, moderator).await;
        let role = ctx
            .seed_role_with_mask(server, "moderator", MANAGE_USERS)
            .await;
        ctx.seed_role_member(role, moderator_member).await;

        let target = ctx.seed_user("target").await;
        ctx.seed_member(server, target).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/kick"))
            .set_json(json!({ "user": target }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(!is_member(&ctx, server, target).await);
    }

    /// Kicking is gated on `MANAGE_USERS` specifically, not on the broader
    /// administer-this-server bit. Holding `MANAGE_SERVER` alone — enough to
    /// rename or even delete the server — must not confer the power to remove
    /// people, or the two constants collapse into one.
    #[actix_web::test]
    pub async fn kick_user_from_server_rejects_manage_server_alone() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let admin = ctx.seed_user("admin").await;
        let admin_member = ctx.seed_member(server, admin).await;
        let role = ctx
            .seed_role_with_mask(server, "admin", MANAGE_SERVER)
            .await;
        ctx.seed_role_member(role, admin_member).await;

        let target = ctx.seed_user("target").await;
        ctx.seed_member(server, target).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/kick"))
            .set_json(json!({ "user": target }));

        let resp = ctx.as_user(admin).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert!(is_member(&ctx, server, target).await);
    }

    /// A plain member has only `BASE_PERMS`, which does not include
    /// `MANAGE_USERS` — and the target must still be a member afterwards.
    #[actix_web::test]
    pub async fn kick_user_from_server_requires_manage_users() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let user = ctx.seed_user("test").await;
        ctx.seed_member(server, user).await;

        let target = ctx.seed_user("target").await;
        ctx.seed_member(server, target).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/kick"))
            .set_json(json!({ "user": target }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert!(is_member(&ctx, server, target).await);
    }

    /// A non-member has no permissions at all, so they can't kick either.
    #[actix_web::test]
    pub async fn kick_user_from_server_rejects_a_non_member() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let outsider = ctx.seed_user("outsider").await;

        let target = ctx.seed_user("target").await;
        ctx.seed_member(server, target).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/kick"))
            .set_json(json!({ "user": target }));

        let resp = ctx.as_user(outsider).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert!(is_member(&ctx, server, target).await);
    }

    /// `MANAGE_USERS` doesn't extend to removing the owner.
    #[actix_web::test]
    pub async fn kick_user_from_server_rejects_the_owner() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let moderator = ctx.seed_user("moderator").await;
        let moderator_member = ctx.seed_member(server, moderator).await;
        let role = ctx
            .seed_role_with_mask(server, "moderator", MANAGE_USERS)
            .await;
        ctx.seed_role_member(role, moderator_member).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/kick"))
            .set_json(json!({ "user": owner }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert!(is_member(&ctx, server, owner).await);
    }
}
