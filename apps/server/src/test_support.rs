//! Shared harness for route-level tests.
//!
//! A [`TestCtx`] owns an isolated in-memory SQLite database with the real
//! migrations applied, so handlers run against the same schema (and the same
//! compile-time-checked queries) as the live server. Nothing here is mocked —
//! only the pieces `main.rs` wires up that would fight the tests are omitted:
//! rate limiting (a 5-request burst would fail every third test), CORS,
//! sessions, and TLS.
//!
//! Two auth modes are available:
//!
//! - [`TestCtx::as_user`] injects [`Claims`] straight into request extensions,
//!   the same slot [`crate::middleware::require_jwt`] fills. Use this for
//!   handler behaviour — it's the fast path and doesn't care about tokens.
//! - [`TestCtx::authenticated`] mounts the real `require_jwt` middleware, so
//!   the test supplies its own `Authorization` header (see [`TestCtx::token`]).
//!   Use this for the auth boundary itself.
//!
//! ```ignore
//! let ctx = TestCtx::new().await;
//! let user = ctx.seed_user("alice").await;
//! let server = ctx.seed_server(user).await;
//!
//! let req = test::TestRequest::post()
//!     .uri(&format!("/api/v1/server/{server}/invite"))
//!     .set_json(json!({ "max_uses": 5 }));
//!
//! let resp = ctx.as_user(user).call(req).await;
//! assert_eq!(resp.status(), StatusCode::OK);
//! ```

use std::str::FromStr;
use std::sync::Once;

use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Next, from_fn};
use actix_web::{App, Error, HttpMessage, test, web};
use ed25519_dalek::pkcs8::EncodePrivateKey;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use time::OffsetDateTime;
use utoipa_actix_web::service_config::ServiceConfig;
use uuid::Uuid;

use crate::models::channel::DmParticipant;
use crate::routes::api;
use crate::state::oauth::OAUTH_CLIENT_ID;
use crate::state::oauth::jwt::{Claims, create_jwt, init_keys_from_pem};

/// Issuer/audience base used by every test. `SERVER_ORIGIN` is process-wide and
/// JWT keys live in a `OnceLock`, so all tests in the binary must agree on this
/// value — see [`init_test_env`].
pub const TEST_ORIGIN: &str = "http://localhost:5000";

static INIT: Once = Once::new();

/// Set `SERVER_ORIGIN` and load a deterministic Ed25519 signing key. Both are
/// process-global (env var + `OnceLock`), so this runs exactly once per test
/// binary and every test shares the result. Never use this key anywhere real.
pub fn init_test_env() {
    INIT.call_once(|| {
        unsafe {
            std::env::set_var("SERVER_ORIGIN", TEST_ORIGIN);
        }

        let signing = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let pem = signing
            .to_pkcs8_pem(LineEnding::LF)
            .expect("encode test key as PKCS#8 PEM");
        // Ignore the result: another test binary path may have initialized the
        // OnceLock already, which is fine — the key is the same either way.
        let _ = init_keys_from_pem(&pem);
    });
}

pub struct TestCtx {
    pub pool: SqlitePool,
}

/// The four columns `join_server` gates on. [`Default`] is a live invite —
/// override one field to build the case under test.
pub struct InviteState {
    pub expires_at: Option<OffsetDateTime>,
    pub max_uses: i64,
    pub uses: i64,
    pub revoked: bool,
}

impl Default for InviteState {
    fn default() -> Self {
        Self {
            expires_at: Some(OffsetDateTime::now_utc() + time::Duration::hours(1)),
            max_uses: 2,
            uses: 0,
            revoked: false,
        }
    }
}

impl TestCtx {
    /// Fresh in-memory database with migrations applied. Each call is fully
    /// isolated from every other, so tests can run in parallel.
    pub async fn new() -> Self {
        init_test_env();

        // `sqlite::memory:` gives every *new* connection its own blank
        // database, so the pool has to hand back the same one forever or the
        // migrated schema disappears — hence exactly one connection, never
        // recycled.
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("parse sqlite::memory:")
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .min_connections(1)
            .max_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options)
            .await
            .expect("open in-memory sqlite");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        Self { pool }
    }

    // -- fixtures ----------------------------------------------------------
    //
    // These use runtime `sqlx::query` rather than the `query!` macro on
    // purpose: macro queries need a matching entry in `.sqlx`, so test-only
    // inserts would force a `cargo sqlx prepare` every time a fixture changes.

    pub async fn seed_user(&self, username: &str) -> Uuid {
        let id = Uuid::now_v7();

        sqlx::query(
            "INSERT INTO users (id, username, psd_hash, created_at, email) VALUES (?,?,?,?,?)",
        )
        .bind(id)
        .bind(username)
        .bind("$argon2id$test-only-not-a-real-hash")
        .bind(OffsetDateTime::now_utc())
        .bind(format!("{username}@example.test"))
        .execute(&self.pool)
        .await
        .expect("seed user");

        id
    }

    /// A role granting nothing. Members still get [`BASE_PERMS`] on top of it —
    /// use [`Self::seed_role_with_mask`] when the test needs a real grant.
    ///
    /// [`BASE_PERMS`]: crate::state::permission::BASE_PERMS
    pub async fn seed_role(&self, server: Uuid, name: &str) -> Uuid {
        self.seed_role_with_mask(server, name, 0).await
    }

    pub async fn seed_role_with_mask(&self, server: Uuid, name: &str, mask: u64) -> Uuid {
        let id = Uuid::now_v7();

        sqlx::query("INSERT INTO roles (id,server_id,name,mask) VALUES (?,?,?,?)")
            .bind(id)
            .bind(server)
            .bind(name)
            .bind(mask as i64)
            .execute(&self.pool)
            .await
            .expect("failed to seed role");

        id
    }

    /// Insert a server owned by `owner`, and add `owner` as a member.
    pub async fn seed_server(&self, owner: Uuid) -> Uuid {
        let id = Uuid::now_v7();

        sqlx::query("INSERT INTO servers (id, name, owner_id, created_at) VALUES (?,?,?,?)")
            .bind(id)
            .bind("test-server")
            .bind(owner)
            .bind(OffsetDateTime::now_utc())
            .execute(&self.pool)
            .await
            .expect("seed server");

        self.seed_member(id, owner).await;

        id
    }

    /// Returns the `server_members.id`, which is what `role_members` links to.
    pub async fn seed_member(&self, server: Uuid, user: Uuid) -> Uuid {
        let id = Uuid::now_v7();

        sqlx::query("INSERT INTO server_members (id, server_id, user_id) VALUES (?,?,?)")
            .bind(id)
            .bind(server)
            .bind(user)
            .execute(&self.pool)
            .await
            .expect("seed server member");

        id
    }

    /// A member of `server` holding one role with `mask`, returned as a
    /// `users.id` ready for [`Self::as_user`].
    ///
    /// Most route tests seed the owner, who short-circuits to `ALL_PERMS` in
    /// `effective_permissions` and so never exercises a permission gate. This
    /// is the shorthand for the non-owner case that does. Remember the caller's
    /// effective permissions are `BASE_PERMS | mask`.
    pub async fn seed_member_with_role(&self, server: Uuid, name: &str, mask: u64) -> Uuid {
        let user = self.seed_user(name).await;
        let member = self.seed_member(server, user).await;
        let role = self.seed_role_with_mask(server, name, mask).await;
        self.seed_role_member(role, member).await;

        user
    }

    /// `member` is a `server_members.id` (what [`Self::seed_member`] returns),
    /// not a `users.id` — that's what `role_members.member_id` references.
    pub async fn seed_role_member(&self, role: Uuid, member: Uuid) {
        sqlx::query("INSERT INTO role_members (role_id, member_id) VALUES (?,?)")
            .bind(role)
            .bind(member)
            .execute(&self.pool)
            .await
            .expect("seed role member");
    }

    /// `kind` must be one of `text`, `voice`, `dm` (CHECK constraint).
    pub async fn seed_channel(&self, server: Option<Uuid>, kind: &str, name: &str) -> Uuid {
        let id = Uuid::now_v7();

        sqlx::query("INSERT INTO channels (id, kind, server_id, name) VALUES (?,?,?,?)")
            .bind(id)
            .bind(kind)
            .bind(server)
            .bind(name)
            .execute(&self.pool)
            .await
            .expect("seed channel");

        id
    }

    /// A text channel in a server where `user` is a **plain member** — i.e.
    /// `BASE_PERMS` and nothing else. Returns `(server, channel)`.
    ///
    /// The server is owned by a separate seeded user on purpose: an owner
    /// short-circuits to `ALL_PERMS` in `effective_permissions`, which would
    /// mask every gate the message routes apply.
    pub async fn seed_text_channel_for(&self, user: Uuid) -> (Uuid, Uuid) {
        let owner = self.seed_user(&format!("owner-of-{user}")).await;
        let server = self.seed_server(owner).await;
        self.seed_member(server, user).await;
        let channel = self.seed_channel(Some(server), "text", "example").await;

        (server, channel)
    }

    /// A dm channel with both users joined as participants — the shape
    /// `channel_permissions` accepts for [`ChannelScope::Dm`].
    ///
    /// [`ChannelScope::Dm`]: crate::state::permission::ChannelScope::Dm
    pub async fn seed_dm_channel(&self, a: Uuid, b: Uuid) -> Uuid {
        let channel = self.seed_channel(None, "dm", "").await;
        self.seed_dm_participant(channel, a, b).await;

        channel
    }

    pub async fn seed_dm_participant(
        &self,
        channel: Uuid,
        user_a: Uuid,
        user_b: Uuid,
    ) -> (Uuid, Uuid) {
        let users = DmParticipant::canonical_pair(user_a, user_b);

        sqlx::query("INSERT INTO dm_participants VALUES (?,?,?)")
            .bind(channel)
            .bind(users.0)
            .bind(users.1)
            .execute(&self.pool)
            .await
            .expect("failed to insert row");

        users
    }

    /// A pending (not yet rejected) friend request from `from` to `to`.
    pub async fn seed_friend_request(&self, from: Uuid, to: Uuid) -> Uuid {
        let id = Uuid::now_v7();

        sqlx::query("INSERT INTO friend_requests (id, from_user, to_user) VALUES (?,?,?)")
            .bind(id)
            .bind(from)
            .bind(to)
            .execute(&self.pool)
            .await
            .expect("seed friend request");

        id
    }

    pub async fn seed_message(&self, channel: Uuid, user: Uuid, content: &str) -> Uuid {
        self.seed_message_at(channel, user, content, OffsetDateTime::now_utc())
            .await
    }

    /// [`Self::seed_message`] with an explicit `created_at`. Pagination and
    /// ordering tests need distinct, known timestamps — `now_utc()` called in a
    /// tight loop can collide, and the keyset cursor is `(created_at, id)`.
    pub async fn seed_message_at(
        &self,
        channel: Uuid,
        user: Uuid,
        content: &str,
        created_at: OffsetDateTime,
    ) -> Uuid {
        let id = Uuid::now_v7();

        sqlx::query(
            "INSERT INTO messages (id, channel_id, user_id, content, created_at) VALUES (?,?,?,?,?)",
        )
        .bind(id)
        .bind(channel)
        .bind(user)
        .bind(content)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .expect("seed message");

        id
    }

    /// Soft-delete a message the way `DELETE /channel/{c}/message/{m}` does.
    pub async fn soft_delete_message(&self, message: Uuid) {
        sqlx::query("UPDATE messages SET deleted_at = ? WHERE id = ?")
            .bind(OffsetDateTime::now_utc())
            .bind(message)
            .execute(&self.pool)
            .await
            .expect("soft delete message");
    }

    /// Invite ids are 21-char nanoids; pass one the route's validator accepts.
    ///
    /// Live by default: expires in an hour, two uses, not revoked. Use
    /// [`Self::seed_invite_with`] to build one the join route should turn away.
    pub async fn seed_invite(&self, id: &str, server: Uuid, created_by: Uuid) {
        self.seed_invite_with(id, server, created_by, InviteState::default())
            .await;
    }

    pub async fn seed_invite_with(
        &self,
        id: &str,
        server: Uuid,
        created_by: Uuid,
        state: InviteState,
    ) {
        sqlx::query(
            "INSERT INTO invites (id, server_id, created_by, expires_at, max_uses, uses, revoked) \
             VALUES (?,?,?,?,?,?,?)",
        )
        .bind(id)
        .bind(server)
        .bind(created_by)
        .bind(state.expires_at)
        .bind(state.max_uses)
        .bind(state.uses)
        .bind(state.revoked)
        .execute(&self.pool)
        .await
        .expect("seed invite");
    }

    /// Reads back the counter the join route increments.
    pub async fn invite_uses(&self, id: &str) -> i64 {
        sqlx::query_scalar("SELECT uses FROM invites WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .expect("read invite uses")
    }

    // -- auth --------------------------------------------------------------

    /// Claims equivalent to what `require_jwt` would extract from a valid
    /// access token for `user`.
    pub fn claims_for(&self, user: Uuid) -> Claims {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Claims {
            iss: TEST_ORIGIN.to_string(),
            aud: format!("{TEST_ORIGIN}/api"),
            sub: user,
            client_id: OAUTH_CLIENT_ID,
            jti: Uuid::now_v7(),
            iat: now,
            exp: now + 3600,
            scope: Some("profile".to_string()),
        }
    }

    /// A real, signed access token for `user` — pass as `Bearer {token}` to a
    /// client built with [`Self::authenticated`].
    pub fn token(&self, user: Uuid) -> String {
        create_jwt(user, OAUTH_CLIENT_ID, Some("profile".to_string()))
            .expect("sign test access token")
    }

    /// Client that skips token parsing and injects `Claims` for `user`.
    pub fn as_user(&self, user: Uuid) -> Client {
        Client {
            pool: self.pool.clone(),
            auth: Auth::Claims(self.claims_for(user)),
        }
    }

    /// Client that mounts the real `require_jwt` middleware. The test owns the
    /// `Authorization` header — omit it to assert the 401 path.
    pub fn authenticated(&self) -> Client {
        Client {
            pool: self.pool.clone(),
            auth: Auth::Jwt,
        }
    }
}

enum Auth {
    /// Inject these claims directly, bypassing token validation.
    Claims(Claims),
    /// Run the real `require_jwt` middleware.
    Jwt,
}

pub struct Client {
    pool: SqlitePool,
    auth: Auth,
}

impl Client {
    /// Mount the full `/api/v1` route set and run one request against it.
    ///
    /// Takes the [`test::TestRequest`] rather than the built request: actix
    /// doesn't re-export `actix_http::Request` anywhere public, so calling
    /// `.to_request()` in here keeps that type out of the signature.
    ///
    /// The app is rebuilt per call rather than held in the struct — actix's
    /// initialized-service type is likewise unnameable without leaking a dozen
    /// generic parameters, and building it is cheap next to the request itself.
    pub async fn call(&self, req: test::TestRequest) -> ServiceResponse<BoxBody> {
        let pool = web::Data::new(self.pool.clone());
        let req = req.to_request();

        match &self.auth {
            Auth::Claims(claims) => {
                let app = test::init_service(
                    App::new()
                        .app_data(pool)
                        .app_data(web::Data::new(claims.clone()))
                        .service(
                            web::scope("/api/v1")
                                .wrap(from_fn(inject_claims))
                                .configure(|c| api::configure_v1(&mut ServiceConfig::new(c))),
                        ),
                )
                .await;

                run(&app, req).await
            }
            Auth::Jwt => {
                let app = test::init_service(
                    App::new().app_data(pool).service(
                        web::scope("/api/v1")
                            .wrap(from_fn(crate::middleware::require_jwt))
                            .configure(|c| api::configure_v1(&mut ServiceConfig::new(c))),
                    ),
                )
                .await;

                run(&app, req).await
            }
        }
    }
}

/// `test::call_service` panics on `Err`, but middleware rejections (e.g. a
/// missing bearer token) *are* `Err` — the HTTP dispatcher turns them into
/// responses in production, and `init_service` has no dispatcher. Do that
/// conversion here so tests can assert on 4xx from middleware.
async fn run<S, R, B>(app: &S, req: R) -> ServiceResponse<BoxBody>
where
    S: Service<R, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static,
{
    match test::try_call_service(app, req).await {
        Ok(resp) => resp.map_into_boxed_body(),
        Err(err) => ServiceResponse::new(
            test::TestRequest::default().to_http_request(),
            err.error_response(),
        ),
    }
}

/// Test-only stand-in for the claims half of `require_jwt`: copies the `Claims`
/// registered as app data into request extensions, where `web::ReqData<Claims>`
/// reads them from.
async fn inject_claims(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if let Some(claims) = req.app_data::<web::Data<Claims>>() {
        let claims = claims.as_ref().clone();
        req.extensions_mut().insert(claims);
    }

    next.call(req).await
}
