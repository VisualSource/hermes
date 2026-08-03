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
        api_errors::ApplicationError,
        oauth::jwt::Claims,
        permission::{
            PERM_CHANNEL, PERM_CREATE, PERM_DELETE, PERM_READ, PERM_WRITE, has_permissions,
        },
    },
};

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
    if !has_permissions(claims.sub, server_id, PERM_CREATE | PERM_CHANNEL).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

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

    if !has_permissions(claims.sub, server_id, PERM_WRITE | PERM_CHANNEL).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

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
    description = "fetch a channel attached to the given server",
    responses(
        (status = 200, description = "accepted changes", body = Channel),
        (status = 401, description = "unauthorized", body = ApplicationError),
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

    if !has_permissions(claims.sub, server_id, PERM_READ | PERM_CHANNEL).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

    let channel = query_as!(Channel, "SELECT * FROM channels WHERE id = ?", &channel_id)
        .fetch_one(db.get_ref())
        .await?;

    Ok(web::Json(channel))
}

#[utoipa::path(
    tag = "channel", 
    description = "delete a channel attached to the given server",
    responses(
        (status = 201, description = "accepted deletion"),
        (status = 401, description = "unauthorized", body = ApplicationError),
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

    if !has_permissions(claims.sub, server_id, PERM_DELETE | PERM_CHANNEL).await? {
        return Err(ApplicationError::new(
            StatusCode::FORBIDDEN,
            "user does not have required permissions",
            "user",
            Vec::default(),
            None,
        ));
    }

    query!("DELETE FROM channels WHERE id = ?", &channel_id)
        .execute(db.get_ref())
        .await?;

    Ok(HttpResponse::Accepted().finish())
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
}
