use actix_web::{HttpResponse, delete, get, http::StatusCode, post, put, web};
use serde::Deserialize;
use sqlx::{SqlitePool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    models::channel::{ChannelKind, DmParticipant, FriendRequest},
    state::{api_errors::ApplicationError, oauth::jwt::Claims},
};

#[utoipa::path(
    tags = ["dm","channel"],
     responses(
        (status = 200, description = "new channel", body = Vec<DmParticipant>),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/dm-channels")]
pub async fn get_dms(
    db: web::Data<SqlitePool>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Vec<DmParticipant>>, ApplicationError> {
    let channels = query_as!(
        DmParticipant,
        "SELECT * FROM dm_participants WHERE user_a_id = ? OR user_b_id = ?",
        user.sub,
        user.sub
    )
    .fetch_all(db.get_ref())
    .await?;

    Ok(web::Json(channels))
}

#[derive(Debug, Deserialize, ToSchema)]
struct CreateFriendRequestPayload {
    user: Uuid,
}

#[utoipa::path(
    tags = ["friend-requests"],
    request_body = CreateFriendRequestPayload,
     responses(
        (status = 200, description = "new channel", body = FriendRequest),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/friend-request")]
pub async fn create_friend_request(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    body: web::Json<CreateFriendRequestPayload>,
) -> Result<web::Json<FriendRequest>, ApplicationError> {
    let id = Uuid::now_v7();

    let req = query_as!(
        FriendRequest,
        "INSERT INTO friend_requests (id,from_user,to_user) VALUES (?,?,?) RETURNING *",
        id,
        claims.sub,
        body.user
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(req))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum FriendRequestType {
    Incoming,
    Outgoing,
}

#[derive(Debug, Deserialize, ToSchema)]

struct FriendRequestQuery {
    request: FriendRequestType,
}

#[utoipa::path(
    tags = ["friend-requests"],
     responses(
        (status = 200, description = "new channel", body = Vec<FriendRequest>),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/friend-request")]
pub async fn list_friend_requests(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    query: web::Query<FriendRequestQuery>,
) -> Result<web::Json<Vec<FriendRequest>>, ApplicationError> {
    let results = match query.request {
        FriendRequestType::Incoming => {
            query_as!(
                FriendRequest,
                "SELECT * FROM friend_requests WHERE to_user = ?",
                claims.sub
            )
            .fetch_all(db.get_ref())
            .await?
        }
        FriendRequestType::Outgoing => {
            query_as!(
                FriendRequest,
                "SELECT * FROM friend_requests WHERE from_user = ?",
                claims.sub
            )
            .fetch_all(db.get_ref())
            .await?
        }
    };

    Ok(web::Json(results))
}

#[utoipa::path(
    tags = ["friend-requests"],
     responses(
        (status = 202, description = "new channel"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/friend-request/{id}")]
pub async fn delete_friend_request(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    params: web::Path<Uuid>,
) -> Result<HttpResponse, ApplicationError> {
    let req_id = params.into_inner();

    query!(
        "DELETE FROM friend_requests WHERE id = ? AND from_user = ?",
        req_id,
        claims.sub
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum RequestAction {
    Reject,
    Accept,
}

#[derive(Debug, Deserialize, ToSchema)]
struct PutFriendRequest {
    action: RequestAction,
}

#[utoipa::path(
    tags = ["friend-requests"],
    request_body = PutFriendRequest,
    responses(
        (status = 201, description = "action accepted"),
        (status = 400, description = "bad request", body = ApplicationError),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[put("/friend-request/{id}")]
pub async fn put_friend_request(
    db: web::Data<SqlitePool>,
    claims: web::ReqData<Claims>,
    params: web::Path<Uuid>,
    body: web::Json<PutFriendRequest>,
) -> Result<HttpResponse, ApplicationError> {
    let req_id = params.into_inner();

    match body.action {
        RequestAction::Reject => {
            query!(
                "UPDATE friend_requests SET rejected = TRUE WHERE id = ? AND to_user = ?",
                req_id,
                claims.sub
            )
            .execute(db.get_ref())
            .await?;
        }
        RequestAction::Accept => {
            let mut tx = db.begin().await?;

            let request = query_as!(
                FriendRequest,
                "DELETE FROM friend_requests WHERE id = ? AND to_user = ? RETURNING *",
                req_id,
                claims.sub
            )
            .fetch_one(&mut *tx)
            .await?;

            if request.rejected {
                tx.rollback().await?;

                return Err(ApplicationError::new(
                    StatusCode::BAD_REQUEST,
                    "can not operate on a rejected request",
                    "request",
                    Vec::default(),
                    None,
                ));
            }

            let channel_id = Uuid::now_v7();

            query!(
                "INSERT INTO channels (id,kind,name) VALUES (?,?,?)",
                &channel_id,
                ChannelKind::Dm,
                ""
            )
            .execute(&mut *tx)
            .await?;

            let canonical = DmParticipant::canonical_pair(request.from_user, request.to_user);

            query!(
                "INSERT INTO dm_participants VALUES (?,?,?)",
                channel_id,
                canonical.0,
                canonical.1
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;

    use crate::test_support::TestCtx;

    use super::*;

    #[actix_web::test]
    async fn get_dms() {
        let ctx = TestCtx::new().await;
        let user_a = ctx.seed_user("user_a").await;
        let user_b = ctx.seed_user("user_b").await;
        let channel = ctx.seed_channel(None, "dm", "").await;

        let users = ctx.seed_dm_participant(channel, user_a, user_b).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/dm-channels"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(user_a).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let dms: Vec<DmParticipant> = test::read_body_json(resp).await;

        assert!(dms.len() == 1);

        assert_eq!(dms[0].user_a_id, users.0);
        assert_eq!(dms[0].user_b_id, users.1);
    }
}
