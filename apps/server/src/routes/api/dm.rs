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
        (status = 404, description = "no such friend request addressed to the caller", body = ApplicationError),
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
            .fetch_optional(&mut *tx)
            .await?;

            // No row means the id is unknown *or* the caller isn't the
            // recipient — accepting is the recipient's move, and a sender must
            // not be able to self-approve into a DM.
            let Some(request) = request else {
                tx.rollback().await?;
                return Err(ApplicationError::not_found("friend request"));
            };

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
    use serde_json::json;

    use crate::models::channel::Channel;
    use crate::test_support::TestCtx;

    use super::*;

    // -- helpers -----------------------------------------------------------

    async fn friend_request(ctx: &TestCtx, id: Uuid) -> Option<FriendRequest> {
        sqlx::query_as::<_, FriendRequest>("SELECT * FROM friend_requests WHERE id = ?")
            .bind(id)
            .fetch_optional(&ctx.pool)
            .await
            .expect("read friend request")
    }

    async fn dm_participants(ctx: &TestCtx) -> Vec<DmParticipant> {
        sqlx::query_as::<_, DmParticipant>("SELECT * FROM dm_participants")
            .fetch_all(&ctx.pool)
            .await
            .expect("read dm participants")
    }

    async fn dm_channels(ctx: &TestCtx) -> Vec<Channel> {
        sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE kind = 'dm'")
            .fetch_all(&ctx.pool)
            .await
            .expect("read dm channels")
    }

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

    /// DM channels aren't attached to a server, so there's no role to consult —
    /// the boundary is row ownership. Every handler here filters on `claims.sub`
    /// instead, and these tests are what hold that in place.
    #[actix_web::test]
    async fn get_dms_returns_only_the_callers_conversations() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;
        let friend = ctx.seed_user("friend").await;
        let stranger_a = ctx.seed_user("stranger_a").await;
        let stranger_b = ctx.seed_user("stranger_b").await;

        let mine = ctx.seed_channel(None, "dm", "").await;
        ctx.seed_dm_participant(mine, user, friend).await;

        // A conversation the caller has nothing to do with.
        let theirs = ctx.seed_channel(None, "dm", "").await;
        ctx.seed_dm_participant(theirs, stranger_a, stranger_b).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/dm-channels")
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let dms: Vec<DmParticipant> = test::read_body_json(resp).await;

        assert_eq!(dms.len(), 1, "someone else's DM must not be listed");
        assert_eq!(dms[0].channel_id, mine);
    }

    // -- create_friend_request ---------------------------------------------

    /// The payload carries only the recipient — `from_user` comes from the
    /// token, so a caller can't open a request in someone else's name.
    #[actix_web::test]
    async fn create_friend_request_takes_the_sender_from_the_token() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;

        let req = test::TestRequest::post()
            .uri("/api/v1/friend-request")
            .set_json(json!({ "user": recipient }));

        let resp = ctx.as_user(sender).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let created: FriendRequest = test::read_body_json(resp).await;

        assert_eq!(created.from_user, sender);
        assert_eq!(created.to_user, recipient);
        assert!(!created.rejected, "a new request starts pending");
    }

    /// `friend_requests.to_user` is `ON DELETE RESTRICT` against `users`, so an
    /// unknown recipient trips the FK and `ApplicationError` maps it to 400.
    #[actix_web::test]
    async fn create_friend_request_for_an_unknown_user_is_rejected() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;

        let req = test::TestRequest::post()
            .uri("/api/v1/friend-request")
            .set_json(json!({ "user": Uuid::now_v7() }));

        let resp = ctx.as_user(sender).call(req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM friend_requests")
            .fetch_one(&ctx.pool)
            .await
            .expect("count friend requests");
        assert_eq!(count, 0);
    }

    // -- list_friend_requests ----------------------------------------------

    #[actix_web::test]
    async fn list_friend_requests_incoming_returns_only_requests_to_the_caller() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;
        let other = ctx.seed_user("other").await;
        let stranger = ctx.seed_user("stranger").await;

        let incoming = ctx.seed_friend_request(other, user).await;
        // Sent by the caller, and one between two other people entirely.
        ctx.seed_friend_request(user, other).await;
        ctx.seed_friend_request(stranger, other).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/friend-request?request=incoming")
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let results: Vec<FriendRequest> = test::read_body_json(resp).await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, incoming);
    }

    #[actix_web::test]
    async fn list_friend_requests_outgoing_returns_only_requests_from_the_caller() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;
        let other = ctx.seed_user("other").await;
        let stranger = ctx.seed_user("stranger").await;

        let outgoing = ctx.seed_friend_request(user, other).await;
        ctx.seed_friend_request(other, user).await;
        ctx.seed_friend_request(stranger, other).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/friend-request?request=outgoing")
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let results: Vec<FriendRequest> = test::read_body_json(resp).await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, outgoing);
    }

    #[actix_web::test]
    async fn list_friend_requests_rejects_an_unknown_direction() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;

        let req = test::TestRequest::get()
            .uri("/api/v1/friend-request?request=sideways")
            .set_payload(Vec::default());

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // -- delete_friend_request ---------------------------------------------

    #[actix_web::test]
    async fn delete_friend_request_cancels_the_callers_own_request() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;
        let request = ctx.seed_friend_request(sender, recipient).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/friend-request/{request}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(sender).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        assert!(friend_request(&ctx, request).await.is_none());
    }

    /// Cancelling is the sender's move — the recipient rejects instead. The
    /// handler doesn't check `rows_affected`, so this still answers 202; what
    /// matters is that the row is untouched.
    #[actix_web::test]
    async fn delete_friend_request_is_scoped_to_the_sender() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;
        let request = ctx.seed_friend_request(sender, recipient).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/friend-request/{request}"))
            .set_payload(Vec::default());

        let resp = ctx.as_user(recipient).call(req).await;

        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        assert!(
            friend_request(&ctx, request).await.is_some(),
            "only the sender may cancel a request"
        );
    }

    // -- put_friend_request ------------------------------------------------

    /// Accepting consumes the request and opens the DM: a `dm` channel plus a
    /// `dm_participants` row holding the canonical (lower, higher) pair, which
    /// is what the `CHECK(user_a_id < user_b_id)` constraint requires.
    #[actix_web::test]
    async fn put_friend_request_accept_opens_a_dm_channel() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;
        let request = ctx.seed_friend_request(sender, recipient).await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/friend-request/{request}"))
            .set_json(json!({ "action": "accept" }));

        let resp = ctx.as_user(recipient).call(req).await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(
            friend_request(&ctx, request).await.is_none(),
            "an accepted request is consumed"
        );

        let channels = dm_channels(&ctx).await;
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].server_id, None, "a DM belongs to no server");

        let participants = dm_participants(&ctx).await;
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0].channel_id, channels[0].id);

        let expected = DmParticipant::canonical_pair(sender, recipient);
        assert_eq!(participants[0].user_a_id, expected.0);
        assert_eq!(participants[0].user_b_id, expected.1);
        assert!(participants[0].user_a_id < participants[0].user_b_id);
    }

    #[actix_web::test]
    async fn put_friend_request_reject_marks_the_row_and_opens_nothing() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;
        let request = ctx.seed_friend_request(sender, recipient).await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/friend-request/{request}"))
            .set_json(json!({ "action": "reject" }));

        let resp = ctx.as_user(recipient).call(req).await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let stored = friend_request(&ctx, request)
            .await
            .expect("a rejected request stays on file");
        assert!(stored.rejected);

        assert!(dm_channels(&ctx).await.is_empty());
        assert!(dm_participants(&ctx).await.is_empty());
    }

    /// Accepting after a rejection is refused, and the `DELETE ... RETURNING`
    /// that reads the row is rolled back — so the request must survive the 400
    /// rather than vanishing.
    #[actix_web::test]
    async fn put_friend_request_accept_after_reject_rolls_back() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;
        let request = ctx.seed_friend_request(sender, recipient).await;

        for action in ["reject", "accept"] {
            let req = test::TestRequest::put()
                .uri(&format!("/api/v1/friend-request/{request}"))
                .set_json(json!({ "action": action }));

            let resp = ctx.as_user(recipient).call(req).await;

            let expected = match action {
                "reject" => StatusCode::NO_CONTENT,
                _ => StatusCode::BAD_REQUEST,
            };
            assert_eq!(resp.status(), expected, "action={action}");
        }

        assert!(
            friend_request(&ctx, request).await.is_some(),
            "the rollback must put the row back"
        );
        assert!(dm_channels(&ctx).await.is_empty());
        assert!(dm_participants(&ctx).await.is_empty());
    }

    /// Accepting is the recipient's move: the sender must not be able to
    /// self-approve into a DM. The `AND to_user = ?` filter means their own
    /// request simply doesn't match, so it reads as a 404 — same answer an
    /// unknown id gets, which is what keeps the endpoint from confirming that
    /// someone else's request exists.
    #[actix_web::test]
    async fn put_friend_request_accept_is_scoped_to_the_recipient() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;
        let request = ctx.seed_friend_request(sender, recipient).await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/friend-request/{request}"))
            .set_json(json!({ "action": "accept" }));

        let resp = ctx.as_user(sender).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        assert!(friend_request(&ctx, request).await.is_some());
        assert!(dm_channels(&ctx).await.is_empty());
        assert!(dm_participants(&ctx).await.is_empty());
    }

    /// An id that matches nothing is a 404, not the 500 `fetch_one` used to
    /// produce by surfacing `RowNotFound` through the `sqlx::Error` conversion.
    #[actix_web::test]
    async fn put_friend_request_accept_for_an_unknown_id_is_not_found() {
        let ctx = TestCtx::new().await;
        let user = ctx.seed_user("user").await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/friend-request/{}", Uuid::now_v7()))
            .set_json(json!({ "action": "accept" }));

        let resp = ctx.as_user(user).call(req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert!(dm_channels(&ctx).await.is_empty());
    }

    /// Rejecting is likewise the recipient's move. This path uses a plain
    /// `UPDATE`, so it answers 204 either way — the row is the assertion.
    #[actix_web::test]
    async fn put_friend_request_reject_is_scoped_to_the_recipient() {
        let ctx = TestCtx::new().await;
        let sender = ctx.seed_user("sender").await;
        let recipient = ctx.seed_user("recipient").await;
        let request = ctx.seed_friend_request(sender, recipient).await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/friend-request/{request}"))
            .set_json(json!({ "action": "reject" }));

        let resp = ctx.as_user(sender).call(req).await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let stored = friend_request(&ctx, request).await.expect("request exists");
        assert!(
            !stored.rejected,
            "the sender must not be able to reject on the recipient's behalf"
        );
    }
}
