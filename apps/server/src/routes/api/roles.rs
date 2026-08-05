use actix_web::{HttpResponse, Responder, delete, get, http::StatusCode, patch, post, put, web};
use actix_web_validation::Validated;
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool, query, query_as, query_scalar};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::roles::Role,
    state::{
        api_errors::{ApplicationError, ErrorDetail},
        oauth::jwt::Claims,
        permission::{
            ADD_ROLE, MANAGE_ROLES, REMOVE_ROLE, effective_permissions, required_permissions,
        },
    },
};

#[derive(Debug, Deserialize, ToSchema, Validate)]
struct CreateRolePayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: String,

    // #123456
    #[validate(length(equal = 7))]
    fg_color: Option<String>,

    // #123456
    #[validate(length(equal = 7))]
    bg_color: Option<String>,

    mask: Option<u64>,
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "create a role",
    request_body = CreateRolePayload,
    responses(
        (status = 200, description = "role", body = Role),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server/{server}/role")]
pub async fn create_role(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    Validated(web::Json(body)): Validated<web::Json<CreateRolePayload>>,
    claims: web::ReqData<Claims>,
) -> Result<web::Json<Role>, ApplicationError> {
    let server_id = params.into_inner();

    let caller = effective_permissions(&db, &claims.sub, &server_id).await?;
    if caller & MANAGE_ROLES != MANAGE_ROLES {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

    let mask = body.mask.unwrap_or_default();
    if mask & !caller != 0 {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user can not set permissions they do no have",
            "user",
            Vec::default(),
            None,
        ));
    }

    let id = Uuid::now_v7();

    let role = query_as!(
        Role,
        "INSERT INTO roles VALUES (?,?,?,?,?,?) RETURNING *",
        id,
        server_id,
        body.name,
        body.fg_color,
        body.bg_color,
        mask as i64
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(role))
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "delete role",
    responses(
        (status = 201, description = "role"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/server/{server}/role/{role}")]
pub async fn delete_role(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    let (server_id, role_id) = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, MANAGE_ROLES).await?;

    query!(
        "DELETE FROM roles WHERE id = ? AND server_id = ?",
        &role_id,
        server_id
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct PatchRolePayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: Option<String>,
    #[validate(length(equal = 7))]
    bg_color: Option<String>,
    #[validate(length(equal = 7))]
    fg_color: Option<String>,

    mask: Option<u64>,
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "update a role",
    request_body = PatchRolePayload,
    responses(
        (status = 201, description = "role"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[patch("/server/{server}/role/{role}")]
pub async fn patch_role(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    Validated(web::Json(body)): Validated<web::Json<PatchRolePayload>>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    if body.mask.is_none()
        && body.name.is_none()
        && body.bg_color.is_none()
        && body.fg_color.is_none()
    {
        return Err(ApplicationError::new(
            StatusCode::BAD_REQUEST,
            "patch body is empty",
            "body",
            Vec::default(),
            None,
        ));
    }

    let (server_id, role_id) = params.into_inner();
    let caller = effective_permissions(&db, &claims.sub, &server_id).await?;
    if caller & MANAGE_ROLES != MANAGE_ROLES {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

    let mut builder = QueryBuilder::<Sqlite>::new("UPDATE roles SET ");
    let mut separated = builder.separated(", ");

    if let Some(name) = body.name {
        separated.push("name = ").push_bind_unseparated(name);
    }

    if let Some(fg) = body.fg_color {
        separated
            .push("fg_color = ")
            .push_bind_unseparated(Some(fg));
    }

    if let Some(bg) = body.bg_color {
        separated
            .push("bg_color = ")
            .push_bind_unseparated(Some(bg));
    }

    if let Some(mask) = body.mask {
        if mask & !caller != 0 {
            return Err(ApplicationError::new(
                StatusCode::FORBIDDEN,
                "user can not set permissions they do no have",
                "user",
                Vec::default(),
                None,
            ));
        }

        separated.push("mask = ").push_bind_unseparated(mask as i64);
    }

    builder.push(" WHERE id = ").push_bind(role_id);
    builder.push(" AND server_id = ").push_bind(server_id);

    let query = builder.build();

    query.execute(db.get_ref()).await?;

    Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "get role",
    responses(
        (status = 200, description = "role", body = Role),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing MANAGE_ROLES", body = ApplicationError),
        (status = 404, description = "no such role in this server", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/server/{server}/role/{role}")]
pub async fn get_role(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    claims: web::ReqData<Claims>,
) -> Result<web::Json<Role>, ApplicationError> {
    let (server_id, role_id) = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, MANAGE_ROLES).await?;

    let role = query_as!(
        Role,
        "SELECT * FROM roles WHERE id = ? AND server_id = ?",
        role_id,
        &server_id
    )
    .fetch_optional(db.get_ref())
    .await?
    .ok_or_else(|| ApplicationError::not_found("role"))?;

    Ok(web::Json(role))
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
struct PutRolePayload {
    role_id: Uuid,
    target: Uuid,
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "add role to user",
    request_body = PutRolePayload,
    responses(
        (status = 201, description = "adding accepted"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 403, description = "missing ADD_ROLE, or the role grants permissions the caller lacks", body = ApplicationError),
        (status = 404, description = "no such role or member in this server", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[put("/server/{server}/role")]
pub async fn add_role_to_user(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    Validated(web::Json(body)): Validated<web::Json<PutRolePayload>>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    let server_id = params.into_inner();

    let caller = effective_permissions(&db, &claims.sub, &server_id).await?;
    if caller & ADD_ROLE != ADD_ROLE {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

    let mut tx = db.begin().await?;

    // Granting is the third way to confer permissions, next to creating and
    // editing a role, so it honours the same rule those two do: nobody hands
    // out more than they hold. Without this check `ADD_ROLE` on its own would
    // be enough to grant yourself any role that already exists in the server —
    // a `MANAGE_SERVER` one included — which turns the weakest role-related
    // permission into full control of the server.
    //
    // The read shares a transaction with the insert below so a concurrent
    // `patch_role` can't widen the mask in between.
    let mask = query_scalar!(
        "SELECT mask FROM roles WHERE id = ? AND server_id = ?",
        body.role_id,
        server_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(role_member_not_found)?;

    if mask as u64 & !caller != 0 {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user can not grant permissions they do no have",
            "user",
            Vec::default(),
            None,
        ));
    }

    // `target` is a user id, but `role_members.member_id` points at
    // `server_members.id` — resolve one to the other here. The joins also do
    // the scope checking: no row comes out unless the role belongs to this
    // server *and* the target is a member of it.
    //
    // PUT is idempotent, so an existing link must not surface the primary-key
    // conflict as an error. `DO UPDATE` rather than `DO NOTHING` is what keeps
    // that distinguishable: the assignment is a no-op, but it makes `RETURNING`
    // fire on the conflicting row, so an empty result means only one thing —
    // the SELECT matched nothing, and the triple is bad.
    let linked = query_scalar!(
        "INSERT INTO role_members (role_id, member_id) \
         SELECT r.id, sm.id \
         FROM roles r \
         JOIN server_members sm ON sm.server_id = r.server_id \
         WHERE r.id = ? AND r.server_id = ? AND sm.user_id = ? \
         ON CONFLICT(role_id, member_id) DO UPDATE SET member_id = excluded.member_id \
         RETURNING member_id",
        body.role_id,
        server_id,
        body.target
    )
    .fetch_optional(&mut *tx)
    .await?;

    if linked.is_none() {
        return Err(role_member_not_found());
    }

    tx.commit().await?;

    Ok(HttpResponse::Accepted().finish())
}

/// Nothing matched the (server, role, target) triple — which of the three is
/// missing is deliberately not distinguished, since a caller who can't see the
/// server shouldn't learn whether a role or user id exists.
fn role_member_not_found() -> ApplicationError {
    ApplicationError::new(
        StatusCode::NOT_FOUND,
        "no such role or member in this server",
        "body",
        vec![ErrorDetail::new(
            4001,
            "target",
            "role and target must both belong to the server in the path",
        )],
        None,
    )
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "remove role from user",
    request_body = PutRolePayload,
    responses(
        (status = 201, description = "remove accepted"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/server/{server}/role")]
pub async fn remove_role_from_user(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    Validated(web::Json(body)): Validated<web::Json<PutRolePayload>>,
    claims: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    let server_id = params.into_inner();

    required_permissions(&db, &claims.sub, &server_id, REMOVE_ROLE).await?;

    // Same user-id -> member-id resolution as `add_role_to_user`, and the
    // `server_id` filter keeps one server from stripping another's roles.
    query!(
        "DELETE FROM role_members \
         WHERE role_id = ? \
           AND member_id IN ( \
               SELECT id FROM server_members WHERE user_id = ? AND server_id = ? \
           )",
        body.role_id,
        body.target,
        server_id
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[cfg(test)]
mod test {
    use actix_web::http::StatusCode;
    use actix_web::test;
    use serde_json::json;

    use crate::models::roles::RoleMember;
    use crate::state::permission::{CREATE_INVITE, MANAGE_SERVER};
    use crate::test_support::TestCtx;

    use super::*;

    #[actix_web::test]
    async fn create_role() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({"name":"role_name" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let role: Role = test::read_body_json(resp).await;

        assert_eq!(role.name, "role_name");
    }

    #[actix_web::test]
    async fn get_role() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let role = ctx.seed_role(server, "role_name").await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}/role/{role}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let role: Role = test::read_body_json(resp).await;

        assert_eq!(role.name, "role_name");
    }

    #[actix_web::test]
    async fn delete_role() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let role = ctx.seed_role(server, "example").await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}/role/{role}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = ?")
            .bind(&role)
            .fetch_optional(&ctx.pool)
            .await
            .expect("failed to get channel");

        assert!(result.is_none())
    }

    #[actix_web::test]
    async fn patch_role() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("test").await;
        let server = ctx.seed_server(user).await;
        let role = ctx.seed_role(server, "example").await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/role/{role}"))
            .set_json(json!({
                "name": "hello",
                "bg_color": "#AABBCC",
                "fg_color": "#AABBCC",
            }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = ?")
            .bind(&role)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to get role");

        assert_eq!(result.name, "hello");
        assert_eq!(result.fg_color, Some("#AABBCC".to_string()));
        assert_eq!(result.bg_color, Some("#AABBCC".to_string()))
    }

    /// `target` in the payload is a *user* id; the row that lands in
    /// `role_members` must hold the matching `server_members.id`.
    #[actix_web::test]
    async fn put_role() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let target = ctx.seed_user("target").await;
        let member = ctx.seed_member(server, target).await;
        let role = ctx.seed_role(server, "example").await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({
                "role_id": role,
                "target": target,
            }));

        let resp = ctx.as_user(owner).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result =
            sqlx::query_as::<_, RoleMember>("SELECT * FROM role_members WHERE role_id = ?")
                .bind(&role)
                .fetch_one(&ctx.pool)
                .await
                .expect("failed to get role member");

        assert_eq!(result.role_id, role);
        assert_eq!(
            result.member_id, member,
            "the link stores the server_members id, not the user id"
        );
    }

    /// PUT is idempotent: granting a role the member already holds is a 202,
    /// not a primary-key conflict, and leaves exactly one link behind.
    #[actix_web::test]
    async fn put_role_twice_is_idempotent() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let target = ctx.seed_user("target").await;
        let member = ctx.seed_member(server, target).await;
        let role = ctx.seed_role(server, "example").await;

        let payload = json!({ "role_id": role, "target": target });

        for attempt in 1..=2 {
            let req = test::TestRequest::put()
                .uri(&format!("/api/v1/server/{server}/role"))
                .set_json(&payload);

            let resp = ctx.as_user(owner).call(req).await;
            assert_eq!(resp.status(), StatusCode::ACCEPTED, "attempt {attempt}");
        }

        let links: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM role_members WHERE role_id = ? AND member_id = ?",
        )
        .bind(&role)
        .bind(&member)
        .fetch_one(&ctx.pool)
        .await
        .expect("count role members");

        assert_eq!(links, 1);
    }

    #[actix_web::test]
    async fn put_role_rejects_a_target_who_is_not_a_member() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let outsider = ctx.seed_user("outsider").await;
        let role = ctx.seed_role(server, "example").await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({
                "role_id": role,
                "target": outsider,
            }));

        let resp = ctx.as_user(owner).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_members")
            .fetch_one(&ctx.pool)
            .await
            .expect("count role members");
        assert_eq!(links, 0);
    }

    /// A role id from another server must not be grantable through this
    /// server's endpoint, even when the target is a legitimate member here.
    #[actix_web::test]
    async fn put_role_rejects_a_role_from_another_server() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let other_server = ctx.seed_server(owner).await;
        let foreign_role = ctx.seed_role(other_server, "elsewhere").await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({
                "role_id": foreign_role,
                "target": owner,
            }));

        let resp = ctx.as_user(owner).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_members")
            .fetch_one(&ctx.pool)
            .await
            .expect("count role members");
        assert_eq!(links, 0);
    }

    #[actix_web::test]
    async fn remove_role() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let target = ctx.seed_user("target").await;
        let member = ctx.seed_member(server, target).await;
        let role = ctx.seed_role(server, "example").await;

        // Without this the assertion below passes vacuously.
        ctx.seed_role_member(role, member).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({
                "role_id": role,
                "target": target,
            }));

        let resp = ctx.as_user(owner).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let result = sqlx::query_as::<_, RoleMember>(
            "SELECT * FROM role_members WHERE role_id = ? AND member_id = ?",
        )
        .bind(&role)
        .bind(&member)
        .fetch_optional(&ctx.pool)
        .await
        .expect("failed to get member role");

        assert!(result.is_none())
    }

    /// The same (role, user) pair in a different server must survive.
    #[actix_web::test]
    async fn remove_role_is_scoped_to_the_server_in_the_path() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let other_server = ctx.seed_server(owner).await;

        let target = ctx.seed_user("target").await;
        ctx.seed_member(server, target).await;
        let other_member = ctx.seed_member(other_server, target).await;

        let role = ctx.seed_role(other_server, "example").await;
        ctx.seed_role_member(role, other_member).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({
                "role_id": role,
                "target": target,
            }));

        let resp = ctx.as_user(owner).call(req).await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let survivor = sqlx::query_as::<_, RoleMember>(
            "SELECT * FROM role_members WHERE role_id = ? AND member_id = ?",
        )
        .bind(&role)
        .bind(&other_member)
        .fetch_optional(&ctx.pool)
        .await
        .expect("failed to get member role");

        assert!(
            survivor.is_some(),
            "deleting through server A must not touch server B's link"
        );
    }

    // -- permission gates --------------------------------------------------
    //
    // Every test above acts as the server owner, who short-circuits to
    // `ALL_PERMS`. That makes the gates unreachable — and, worse, makes the
    // `mask & !caller` escalation checks in `create_role`/`patch_role` dead
    // code under test, since `!ALL_PERMS` is zero. These drive both through a
    // non-owner.

    async fn role_mask(ctx: &TestCtx, role: Uuid) -> i64 {
        sqlx::query_scalar("SELECT mask FROM roles WHERE id = ?")
            .bind(role)
            .fetch_one(&ctx.pool)
            .await
            .expect("read role mask")
    }

    async fn count_roles(ctx: &TestCtx) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM roles")
            .fetch_one(&ctx.pool)
            .await
            .expect("count roles")
    }

    #[actix_web::test]
    async fn create_role_requires_manage_roles() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({ "name": "role_name" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert_eq!(count_roles(&ctx).await, 0);
    }

    /// The escalation guard: `MANAGE_ROLES` lets you mint roles, but only out
    /// of permissions you already hold. Otherwise any role manager could write
    /// themselves a `MANAGE_SERVER` role and take the server.
    #[actix_web::test]
    async fn create_role_rejects_granting_permissions_the_caller_lacks() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let moderator = ctx.seed_member_with_role(server, "moderator", MANAGE_ROLES).await;
        let before = count_roles(&ctx).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({ "name": "escalated", "mask": MANAGE_ROLES | MANAGE_SERVER }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            count_roles(&ctx).await,
            before,
            "the role must not be created at all"
        );
    }

    /// The other half of the guard — permissions the caller *does* hold are
    /// grantable, so the check isn't just refusing every non-owner.
    #[actix_web::test]
    async fn create_role_allows_granting_permissions_the_caller_holds() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let moderator = ctx
            .seed_member_with_role(server, "moderator", MANAGE_ROLES | CREATE_INVITE)
            .await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({ "name": "inviter", "mask": CREATE_INVITE }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let role: Role = test::read_body_json(resp).await;
        assert_eq!(role.mask, CREATE_INVITE as i64);
    }

    /// Unknown id and "exists, but in another server" are the same 404 — the
    /// query is scoped by `server_id`, and both used to be a 500.
    #[actix_web::test]
    async fn get_role_for_an_unknown_or_foreign_role_is_not_found() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let other_server = ctx.seed_server(owner).await;
        let foreign_role = ctx.seed_role(other_server, "elsewhere").await;

        for role in [Uuid::now_v7(), foreign_role] {
            let req = test::TestRequest::get()
                .uri(&format!("/api/v1/server/{server}/role/{role}"))
                .set_payload(Vec::default());

            let resp = ctx.as_user(owner).call(req).await;

            assert_eq!(resp.status(), StatusCode::NOT_FOUND, "role={role}");
        }
    }

    #[actix_web::test]
    async fn get_role_requires_manage_roles() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let role = ctx.seed_role(server, "example").await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/server/{server}/role/{role}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn delete_role_requires_manage_roles() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let role = ctx.seed_role(server, "example").await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}/role/{role}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let survivor = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = ?")
            .bind(&role)
            .fetch_optional(&ctx.pool)
            .await
            .expect("failed to get role");

        assert!(survivor.is_some());
    }

    #[actix_web::test]
    async fn patch_role_requires_manage_roles() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let role = ctx.seed_role(server, "example").await;

        let user = ctx.seed_user("plain").await;
        ctx.seed_member(server, user).await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/role/{role}"))
            .set_json(json!({ "name": "renamed" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let result = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = ?")
            .bind(&role)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to get role");

        assert_eq!(result.name, "example");
    }

    /// Same escalation guard as `create_role`, on the edit path — a role
    /// manager must not be able to widen an existing role past their own
    /// permissions, and the mask must be left untouched when they try.
    #[actix_web::test]
    async fn patch_role_rejects_granting_permissions_the_caller_lacks() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let role = ctx.seed_role_with_mask(server, "example", CREATE_INVITE).await;

        let moderator = ctx.seed_member_with_role(server, "moderator", MANAGE_ROLES).await;

        let req = test::TestRequest::patch()
            .uri(&format!("/api/v1/server/{server}/role/{role}"))
            .set_json(json!({ "mask": MANAGE_SERVER }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert_eq!(role_mask(&ctx, role).await, CREATE_INVITE as i64);
    }

    /// Granting a role is gated on `ADD_ROLE`, which is a separate bit from
    /// `MANAGE_ROLES` — holding only the latter must not let you hand roles out.
    #[actix_web::test]
    async fn put_role_requires_add_role() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let role = ctx.seed_role(server, "example").await;

        let moderator = ctx.seed_member_with_role(server, "moderator", MANAGE_ROLES).await;

        let target = ctx.seed_user("target").await;
        ctx.seed_member(server, target).await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({ "role_id": role, "target": target }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_members WHERE role_id = ?")
            .bind(&role)
            .fetch_one(&ctx.pool)
            .await
            .expect("count role members");
        assert_eq!(links, 0);
    }

    /// `ADD_ROLE` is the weakest role permission, and granting is the third way
    /// to confer permissions — so it obeys the same rule `create_role` and
    /// `patch_role` do. Before this was enforced, a member holding only
    /// `ADD_ROLE` could grant themselves any existing role in the server and
    /// walk away with `MANAGE_SERVER`.
    #[actix_web::test]
    async fn put_role_rejects_granting_permissions_the_caller_lacks() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        // An admin role already exists, well above the grantor.
        let admin_role = ctx
            .seed_role_with_mask(server, "admin", MANAGE_SERVER)
            .await;

        let moderator = ctx.seed_member_with_role(server, "moderator", ADD_ROLE).await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({ "role_id": admin_role, "target": moderator }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_members WHERE role_id = ?")
            .bind(&admin_role)
            .fetch_one(&ctx.pool)
            .await
            .expect("count role members");
        assert_eq!(links, 0);

        let perms = effective_permissions(&ctx.pool, &moderator, &server)
            .await
            .expect("effective permissions");
        assert_eq!(
            perms & MANAGE_SERVER,
            0,
            "the grantor must not have escalated themselves"
        );
    }

    /// The positive case — a role whose mask the caller fully holds is
    /// grantable, so the guard isn't refusing every non-owner.
    #[actix_web::test]
    async fn put_role_allows_granting_permissions_the_caller_holds() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;

        let inviter_role = ctx
            .seed_role_with_mask(server, "inviter", CREATE_INVITE)
            .await;

        let moderator = ctx.seed_member_with_role(server, "moderator", ADD_ROLE | CREATE_INVITE).await;

        let target = ctx.seed_user("target").await;
        let member = ctx.seed_member(server, target).await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({ "role_id": inviter_role, "target": target }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let link = sqlx::query_as::<_, RoleMember>(
            "SELECT * FROM role_members WHERE role_id = ? AND member_id = ?",
        )
        .bind(&inviter_role)
        .bind(&member)
        .fetch_optional(&ctx.pool)
        .await
        .expect("failed to get member role");

        assert!(link.is_some());
    }

    /// Mirror of `put_role_requires_add_role` for `REMOVE_ROLE`, and the
    /// existing link must survive the refusal.
    #[actix_web::test]
    async fn remove_role_requires_remove_role() {
        let ctx = TestCtx::new().await;
        let owner = ctx.seed_user("owner").await;
        let server = ctx.seed_server(owner).await;
        let role = ctx.seed_role(server, "example").await;

        let moderator = ctx.seed_member_with_role(server, "moderator", MANAGE_ROLES | ADD_ROLE).await;

        let target = ctx.seed_user("target").await;
        let member = ctx.seed_member(server, target).await;
        ctx.seed_role_member(role, member).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/server/{server}/role"))
            .set_json(json!({ "role_id": role, "target": target }));

        let resp = ctx.as_user(moderator).call(req).await;

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let survivor = sqlx::query_as::<_, RoleMember>(
            "SELECT * FROM role_members WHERE role_id = ? AND member_id = ?",
        )
        .bind(&role)
        .bind(&member)
        .fetch_optional(&ctx.pool)
        .await
        .expect("failed to get member role");

        assert!(survivor.is_some());
    }
}
