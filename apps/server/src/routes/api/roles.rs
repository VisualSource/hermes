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
        permission::{PERM_CREATE, PERM_DELETE, PERM_ROLE, PERM_WRITE, has_permissions},
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
    if !has_permissions(claims.sub, server_id, PERM_CREATE | PERM_ROLE).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

    let id = Uuid::now_v7();

    let mask = 0;

    let role = query_as!(
        Role,
        "INSERT INTO roles VALUES (?,?,?,?,?,?) RETURNING *",
        id,
        server_id,
        body.name,
        body.fg_color,
        body.bg_color,
        mask
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

    if !has_permissions(claims.sub, server_id, PERM_DELETE | PERM_ROLE).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

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
    if body.name.is_none() && body.bg_color.is_none() && body.fg_color.is_none() {
        return Err(ApplicationError::new(
            StatusCode::BAD_REQUEST,
            "patch body is empty",
            "body",
            Vec::default(),
            None,
        ));
    }

    let (server_id, role_id) = params.into_inner();
    if !has_permissions(claims.sub, server_id, PERM_WRITE | PERM_ROLE).await? {
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

    if !has_permissions(claims.sub, server_id, PERM_DELETE | PERM_ROLE).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

    let role = query_as!(
        Role,
        "SELECT * FROM roles WHERE id = ? AND server_id = ?",
        role_id,
        &server_id
    )
    .fetch_one(db.get_ref())
    .await?;

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

    if !has_permissions(claims.sub, server_id, PERM_WRITE | PERM_ROLE).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
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
    .fetch_optional(db.get_ref())
    .await?;

    if linked.is_none() {
        return Err(role_member_not_found());
    }

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

    if !has_permissions(claims.sub, server_id, PERM_DELETE | PERM_ROLE).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

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
}
