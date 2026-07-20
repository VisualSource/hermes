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
cargo test                                       # unit tests (JWT + PKCE)
cargo test <pattern>                             # single test
sqlx migrate run                                 # apply pending migrations
sqlx migrate add <name>                          # scaffold a new migration
cargo sqlx prepare -- --lib                      # regenerate .sqlx offline data
```

Logging is configured in [log4rs.yaml](log4rs.yaml).

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
