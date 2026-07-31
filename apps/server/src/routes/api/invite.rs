use actix_web::{HttpResponse, delete, post, web};
use actix_web_validation::Validated;
use nanoid::nanoid;
use serde::Deserialize;
use sqlx::{SqlitePool, query, query_as};
use time::SignedDuration;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::invite::Invite,
    state::{api_errors::ApplicationError, oauth::jwt::Claims},
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct CreateInvitePayload {
    #[validate(range(min = 1, max = 255))]
    max_uses: Option<i64>,
}

#[utoipa::path(
    tags = ["server","invite"],
    request_body = CreateInvitePayload,
    responses(
        (status = 200, description = "created server invite", body = Invite),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server/{server}/invite")]
pub async fn create_invite(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    params: web::Path<Uuid>,
    body: Validated<web::Json<CreateInvitePayload>>,
) -> Result<web::Json<Invite>, ApplicationError> {
    //TODO: validate user can create invite

    let server_id = params.into_inner();
    let id = nanoid!();

    let expires_at = time::OffsetDateTime::now_utc().checked_add(SignedDuration::hours(1));
    let max_uses = body.max_uses.unwrap_or(2);

    let invite = query_as!(
        Invite,
        "INSERT INTO invites (id,server_id,created_by,expires_at,max_uses) VALUES (?,?,?,?,?) RETURNING *",
        id,
        server_id,
        claims.sub,
        expires_at,
        max_uses,
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(invite))
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
struct DeleteInviteQuery {
    #[validate(length(equal = 21))]
    id: String,
}

#[utoipa::path(
    tags = ["server","invite"],
    responses(
        (status = 201, description = "accepted revoked"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/server/{server}/invite-revoke")]
pub async fn revoke_invite(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    params: web::Path<Uuid>,
    Validated(web::Query(query)): Validated<web::Query<DeleteInviteQuery>>,
) -> Result<HttpResponse, ApplicationError> {
    //TODO: validate user can revoke invite
    let server_id = params.into_inner();

    query!(
        "UPDATE invites SET revoked = TRUE WHERE server_id = ? AND id = ?",
        server_id,
        query.id
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[cfg(test)]
mod test {
    use actix_web::http::StatusCode;
    use actix_web::http::header::AUTHORIZATION;
    use actix_web::test;
    use serde_json::json;

    use crate::test_support::TestCtx;

    use super::*;

    #[actix_web::test]
    async fn create_invite_returns_the_new_invite() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/invite"))
            .set_json(json!({ "max_uses": 5 }));

        let resp = ctx.as_user(user).call(req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let invite: Invite = test::read_body_json(resp).await;
        assert_eq!(invite.server_id, server);
        assert_eq!(invite.created_by, Some(user));
        assert_eq!(invite.max_uses, 5);
        assert_eq!(invite.uses, 0);
        assert!(!invite.revoked);
        assert_eq!(invite.id.len(), 21, "id should be a nanoid");
        assert!(invite.expires_at.is_some());
    }

    #[actix_web::test]
    async fn create_invite_defaults_max_uses() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{server}/invite"))
            .set_json(json!({}));

        let resp = ctx.as_user(user).call(req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let invite: Invite = test::read_body_json(resp).await;
        assert_eq!(invite.max_uses, 2);
    }

    #[actix_web::test]
    async fn create_invite_rejects_out_of_range_max_uses() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;
        let server = ctx.seed_server(user).await;

        for max_uses in [0, 256] {
            let req = test::TestRequest::post()
                .uri(&format!("/api/v1/server/{server}/invite"))
                .set_json(json!({ "max_uses": max_uses }));

            let resp = ctx.as_user(user).call(req).await;
            assert_eq!(
                resp.status(),
                StatusCode::BAD_REQUEST,
                "max_uses={max_uses} should fail validation"
            );
        }
    }

    /// The `invites.server_id` FK has nothing to point at, so sqlx surfaces a
    /// database error — which `ApplicationError` maps to 400, not 404.
    #[actix_web::test]
    async fn create_invite_for_unknown_server_is_rejected() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/server/{}/invite", Uuid::now_v7()))
            .set_json(json!({}));

        let resp = ctx.as_user(user).call(req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn create_invite_requires_a_bearer_token() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;
        let server = ctx.seed_server(user).await;
        let uri = format!("/api/v1/server/{server}/invite");

        let anonymous = test::TestRequest::post().uri(&uri).set_json(json!({}));

        let resp = ctx.authenticated().call(anonymous).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let authorized = test::TestRequest::post()
            .uri(&uri)
            .insert_header((AUTHORIZATION, format!("Bearer {}", ctx.token(user))))
            .set_json(json!({}));

        let resp = ctx.authenticated().call(authorized).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn revoke_invite_marks_the_row_revoked() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;
        let server = ctx.seed_server(user).await;
        let invite_id = nanoid!();
        ctx.seed_invite(&invite_id, server, user).await;

        let req = test::TestRequest::delete().uri(&format!(
            "/api/v1/server/{server}/invite-revoke?id={invite_id}"
        ));

        let resp = ctx.as_user(user).call(req).await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let revoked: bool = sqlx::query_scalar("SELECT revoked FROM invites WHERE id = ?")
            .bind(&invite_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("read back invite");
        assert!(revoked);
    }

    #[actix_web::test]
    async fn revoke_invite_rejects_a_malformed_id() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;
        let server = ctx.seed_server(user).await;

        let req = test::TestRequest::delete().uri(&format!(
            "/api/v1/server/{server}/invite-revoke?id=too-short"
        ));

        let resp = ctx.as_user(user).call(req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Revoking an id that belongs to a different server must not touch it.
    #[actix_web::test]
    async fn revoke_invite_is_scoped_to_the_server_in_the_path() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("alice").await;
        let owning_server = ctx.seed_server(user).await;
        let other_server = ctx.seed_server(user).await;
        let invite_id = nanoid!();
        ctx.seed_invite(&invite_id, owning_server, user).await;

        let req = test::TestRequest::delete().uri(&format!(
            "/api/v1/server/{other_server}/invite-revoke?id={invite_id}"
        ));

        let resp = ctx.as_user(user).call(req).await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let revoked: bool = sqlx::query_scalar("SELECT revoked FROM invites WHERE id = ?")
            .bind(&invite_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("read back invite");
        assert!(!revoked, "invite belongs to another server");
    }
}
