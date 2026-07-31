# hermes-server

Rust / Actix-web signaling + HTTP API for Hermes. OAuth2 (auth-code + PKCE + refresh) with EdDSA-signed JWTs published via JWKS, SQLite via `sqlx`, and a WebSocket relay for WebRTC signaling.

Binds `0.0.0.0:7433`.

## Prerequisites

- **Rust toolchain** — edition 2024, so `rustc ≥ 1.85`.
- **NASM** — required by `aws-lc-sys` (pulled transitively by `jsonwebtoken`'s `aws_lc_rs` backend). Install once per machine:
  - Windows: `winget install nasm` then restart your shell so the new PATH is picked up.
  - macOS: `brew install nasm`.
  - Linux: `apt install nasm` / `dnf install nasm`.
- **sqlx-cli** — for creating the DB and applying migrations:
  ```sh
  cargo install sqlx-cli --no-default-features --features sqlite --locked
  ```
- **OpenSSL** — used once to generate the JWT signing key.

## First-time setup

### 1. Generate the Ed25519 signing key

The server signs access + refresh tokens with an Ed25519 keypair. The public key is served at `/.well-known/jwks.json`.

```sh
mkdir keys
openssl genpkey -algorithm ed25519 -out keys/ed25519.pem
```

Add `keys/` to `.gitignore` if it isn't already. **Never commit the private key.**

Rotating the key later is a matter of overwriting the file — the `kid` in the JWKS is derived from the public key via RFC 7638 JWK Thumbprint, so it changes automatically. All outstanding tokens signed by the old key become invalid at that point.

### 2. Create the SQLite database

```sh
DATABASE_URL="sqlite://sqlite.db?mode=rwc" sqlx database create
DATABASE_URL="sqlite://sqlite.db?mode=rwc" sqlx migrate run
```

Migrations live in [migrations/](migrations/). The server also runs `sqlx::migrate!` at startup, so once the DB file exists it stays in sync, but sqlx's compile-time query checks require the schema to be applied before `cargo check` / `cargo build`.

### 3. Create `.env`

The server refuses to boot if any required env var is missing or too weak. Copy the template below into `.env` at the repo root (dotenvy loads from CWD, and `cargo run` is typically invoked from `apps/server/`):

```dotenv
# Public origin the AS issues tokens from; used as JWT `iss` and access-token `aud`.
SERVER_ORIGIN=http://localhost:7433

# Path to a PKCS#8 PEM Ed25519 private key. Generate with `openssl genpkey -algorithm ed25519`.
JWT_PRIVATE_KEY_PATH=./keys/ed25519.pem

# ≥ 64-byte string used to key the cookie session store. Rotating invalidates
# every outstanding login. Generate one with e.g. `openssl rand -base64 64`.
SESSION_SECRET=<paste-64+-bytes-here>

# SQLite connection string.
DATABASE_URL=sqlite://sqlite.db?mode=rwc

# Optional: origin allowed to hit the API from a browser. Defaults to the Tauri dev origin.
CORS_ALLOWED_ORIGIN=http://localhost:1420

# CloudFlare Turnstile. Required — the login/signup pages fail if these aren't set.
CLOUDFLARE_TURNSTILE_SITE_KEY
CLOUDFLARE_TURNSTILE_API_KEY
```

Startup validation checks `SERVER_ORIGIN`, `JWT_PRIVATE_KEY_PATH`, and `SESSION_SECRET` (length included). Missing values print a clear error and exit before the HTTP listener binds.

### 4. Run

```sh
cargo run
```

## Common commands

```sh
cargo run                                        # boot server on :7433
cargo test                                       # unit + route tests
cargo test <pattern>                             # single test
sqlx migrate run                                 # apply pending migrations
sqlx migrate add <name>                          # scaffold a new migration
cargo sqlx prepare -- --lib                      # regenerate .sqlx offline data
```

Logging is configured in [log4rs.yaml](log4rs.yaml).

## Tests

Route tests live in a `#[cfg(test)] mod test` next to the handlers they cover, and share the harness in [src/test_support.rs](src/test_support.rs).

`TestCtx::new()` gives each test its own in-memory SQLite database with the real migrations applied — nothing is mocked, so handlers hit the same schema and the same compile-time-checked queries as production. The only things the harness leaves out of the app are the pieces `main.rs` wires up that would fight the tests: rate limiting (the default governor's 5-request burst would fail every third test), CORS, sessions, and TLS.

```rust
let ctx = TestCtx::new().await;
let user = ctx.seed_user("alice").await;
let server = ctx.seed_server(user).await;

let req = test::TestRequest::post()
    .uri(&format!("/api/v1/server/{server}/invite"))
    .set_json(json!({ "max_uses": 5 }));

let resp = ctx.as_user(user).call(req).await;
assert_eq!(resp.status(), StatusCode::OK);

let invite: Invite = test::read_body_json(resp).await;
```

`call` mounts the whole `/api/v1` route set via `routes::api::configure_v1`, so tests exercise real URI matching and extractors. It takes the `TestRequest` itself rather than `.to_request()` — actix doesn't re-export `actix_http::Request` publicly, so the harness converts internally.

Two auth modes:

- `ctx.as_user(id)` injects `Claims` straight into request extensions — the same slot `require_jwt` fills. Use for handler behaviour.
- `ctx.authenticated()` mounts the real `require_jwt` middleware; the test sets its own `Authorization` header (`ctx.token(id)` mints a valid one). Use for the auth boundary — omit the header to assert the 401.

Fixtures (`seed_user`, `seed_server`, `seed_member`, `seed_channel`, `seed_message`, `seed_invite`) deliberately use runtime `sqlx::query` rather than the `query!` macro, so adding or changing one never requires a `cargo sqlx prepare`.

`SERVER_ORIGIN` and the JWT key `OnceLock` are process-global, so every test in the binary shares `test_support::init_test_env()` — don't set them per-test.

> Note: `cargo test` compiles the `query!` macros against the schema at `DATABASE_URL`, not against `migrations/`. If a migration is edited after it was applied, the dev DB and the migration files drift, and the *generated* decode types come from the stale DB — which shows up as a 500 at runtime, not a compile error. When migrations change, recreate the DB rather than relying on `CREATE TABLE IF NOT EXISTS` to catch up.

## OAuth surface

Discovery: `GET /.well-known/oauth2-authorization-server` and `GET /.well-known/jwks.json`.

| Endpoint            | Method | Purpose                                             |
| ------------------- | ------ | --------------------------------------------------- |
| `/auth/authorize`   | GET    | Authorization-code + PKCE grant (RFC 6749 §4.1)     |
| `/auth/token`       | POST   | Token endpoint — auth-code + refresh grants         |
| `/auth/revoke`      | POST   | RFC 7009 token revocation                           |
| `/login`, `/signup` | GET/POST | Local password login (Argon2) + reCAPTCHA v3      |

Access tokens are `typ: at+jwt` per RFC 9068. Refresh tokens use `typ: rt+jwt` and rotate on every use, with family revocation on reuse per OAuth 2.1 §6.1.

## Troubleshooting

- **`error returned from database: unable to open database file`** on `cargo check` — the SQLite file at `DATABASE_URL` doesn't exist yet or the path is wrong. Re-run the `sqlx database create` + `sqlx migrate run` step from setup.
- **`NASM command not found`** during build — NASM isn't installed or isn't on `PATH`. See prereqs.
- **`JWT_PRIVATE_KEY_PATH is not set`** at startup — copy `.env` template, generate the key with `openssl genpkey`, restart.
- **Users get logged out on every restart** — you're likely still on the pre-hardening `Key::generate()` path or `SESSION_SECRET` is being rotated between boots. Set a stable `SESSION_SECRET` in `.env`.
