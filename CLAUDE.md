# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Hermes is a Discord-like clone: text chat + voice channels using WebRTC (peer mesh) with the server acting as the WebSocket signaling relay and the HTTP+OAuth2 identity provider. The desktop client is Tauri (React + Rust); an overlay-client is a separate Rust binary that draws into other windows via `asdf-overlay-client`.

## Repo layout

Monorepo with three apps under `apps/` and shared API contracts under `api/`:

- [apps/server](apps/server/) — Rust / Actix-web signaling + HTTP API. SQLite via `sqlx` with migrations in [apps/server/migrations/](apps/server/migrations/). OAuth2 authorization-code + PKCE lives in [apps/server/src/state/oauth/](apps/server/src/state/oauth/) and [apps/server/src/routes/auth/oauth.rs](apps/server/src/routes/auth/oauth.rs). HTTP routes are split: `/auth/*` (login/signup/oauth), `/api/v1/*` (REST), `/api/ws` (WebSocket signaling). Binds `0.0.0.0:7433`.
- [apps/desktop-client](apps/desktop-client/) — Tauri 2 shell wrapping a Vite + React 19 SPA. TypeScript UI in [src/](apps/desktop-client/src/), Rust side in [src-tauri/](apps/desktop-client/src-tauri/). Routing via TanStack Router with generated [routeTree.gen.ts](apps/desktop-client/src/routeTree.gen.ts); data with TanStack Query. Client OAuth flow in [src/lib/auth.ts](apps/desktop-client/src/lib/auth.ts) uses `@badgateway/oauth2-client` and either the `hermes://oauth` deep link (Tauri) or a popup + `BroadcastChannel` (dev browser).
- [apps/overlay-client](apps/overlay-client/) — Standalone Rust binary; in-game/window overlay UI (egui + asdf-overlay-client).
- [api/proto/hermes.proto](api/proto/hermes.proto) — Protobuf schema for the WebSocket `Envelope` (voice channel join/leave + WebRTC signaling: SDP offer/answer, ICE candidates).
- [api/openapi.yaml](api/openapi.yaml) — REST contract (currently minimal; server declares its OpenAPI via `utoipa`).
- [configs/dev/Caddyfile](configs/dev/Caddyfile) — dev Caddy reverse-proxies `localhost` → `host.docker.internal:7433`.
- [depolyments/](depolyments/) — [sic] docker-compose files. The local one runs Caddy for TLS termination in dev.

Note: [pnpm-workspace.yaml](pnpm-workspace.yaml) and the root [package.json](package.json) still reference the pre-rename `packages/` paths. The active tree is `apps/`. The `dev:server` script in root `package.json` (`cd ./packages/server && cargo run`) is stale — run the server directly from [apps/server](apps/server/).

## Commands

### Desktop client — [apps/desktop-client](apps/desktop-client/)

```
pnpm install
pnpm dev          # Vite dev server on :1420 (Tauri expects fixed port)
pnpm tauri dev    # Launch Tauri shell with dev server
pnpm build        # tsc && vite build
pnpm test         # Vitest
pnpm protoc       # Regenerate src/lib/proto/hermes.ts from ../../api/proto/hermes.proto
pnpm openapi-ts   # Regenerate src/api/ from ../../api/openapi.yaml (via @hey-api/openapi-ts + biome format)
```

Vitest single test: `pnpm test -- <pattern>` or `pnpm exec vitest run path/to/file.test.ts`.

Requires env vars: `VITE_SERVER_URL`, `VITE_CLIENT_ID` (OAuth client). The client hits `${VITE_SERVER_URL}/api/ws?token=...` with WSS.

### Server — [apps/server](apps/server/)

```
cargo run                                  # Runs on :7433, reads .env (dotenvy)
cargo test
cargo test <test_name>
sqlx migrate run                           # If schema changes; migrations in ./migrations
```

Requires env: `DATABASE_URL` (SQLite, e.g. `sqlite://hermes.db`), `SERVER_ORIGIN` (used for OAuth `issuer`). Logging config in [log4rs.yaml](apps/server/log4rs.yaml). CORS is currently pinned to `http://localhost:1420` in [src/main.rs](apps/server/src/main.rs).

### Overlay client — [apps/overlay-client](apps/overlay-client/)

```
cargo run
```

### Dev infrastructure

```
pnpm dev:caddy    # sudo docker compose -f depolyments/docker-compose-local.yml up (Caddy TLS front)
```

## Architecture notes

### Signaling and voice

Voice is a peer mesh, not SFU. The server only relays signaling: [src/routes/websocket.rs](apps/server/src/routes/websocket.rs) authenticates the WS via the `token` query param, then brokers `Envelope` protobuf frames. Voice-channel join/leave events fan out to peers, who then negotiate RTCPeerConnection offers/answers directly via the same relay. Client-side orchestration is in [apps/desktop-client/src/lib/core/app.ts](apps/desktop-client/src/lib/core/app.ts) (singleton `App` class, extends `EventTarget`) and [rtc.ts](apps/desktop-client/src/lib/core/rtc.ts). Microphone capture uses the Web Audio API plus `@sapphi-red/web-noise-suppressor`; Opus is available on the Tauri side via the `opus` crate + `cpal` (see [src-tauri/src/cmd/audio.rs](apps/desktop-client/src-tauri/src/cmd/audio.rs)).

If you touch the wire format, regenerate both sides:
- TS: `pnpm protoc` in `apps/desktop-client` (uses `ts-proto`, browser env, no JSON methods).
- Rust: `prost-build` runs from [apps/server/build.rs](apps/server/build.rs).

### Auth

The server is its own OAuth2 provider (see the `/.well-known/oauth2-authorization-server` handler in [src/routes/mod.rs](apps/server/src/routes/mod.rs)). Flow is authorization-code + PKCE + `offline_access`. Passwords are Argon2. The `grants` and `refresh_tokens` tables in [migrations/000_init.sql](apps/server/migrations/000_init.sql) back this; JWT signing lives in [src/state/oauth/jwt.rs](apps/server/src/state/oauth/jwt.rs).

Desktop client OAuth completes via one of two paths:
- Tauri build: OS opens the browser to the authorize URL; the `hermes://oauth` deep link comes back through `tauri-plugin-single-instance` → `tauri-plugin-deep-link`, which emits `hermes://auth` to the JS side (see [src-tauri/src/lib.rs](apps/desktop-client/src-tauri/src/lib.rs)).
- Dev browser (`import.meta.env.DEV`): popup window + `BroadcastChannel("oauth")`.

### Data layer

Server uses `sqlx` with compile-time-checked queries against SQLite (`sqlx-macros` is `opt-level=3` even in dev). Models in [src/models/](apps/server/src/models/). Actix `web::Data<SqlitePool>` is the injection surface. Migrations run at startup ([db.rs](apps/server/src/db.rs)).

Client data uses TanStack Query with a generated fetch client (`@hey-api/openapi-ts` → [src/api/](apps/desktop-client/src/api/)) and hand-rolled query wrappers in [src/lib/api/queries.ts](apps/desktop-client/src/lib/api/queries.ts).

## Tooling / conventions

- Formatter and linter: **Biome** (config in each package's `biome.json`). Tabs for indent, double quotes for JS/TS. `organizeImports` is disabled — don't let editor plugins reorder imports automatically.
- TS path alias `@/*` → `src/*` in [apps/desktop-client/vite.config.ts](apps/desktop-client/vite.config.ts).
- UI stack: TanStack Router (file-based routes under [src/routes/](apps/desktop-client/src/routes/), auto-generated `routeTree.gen.ts` — don't hand-edit), TanStack Query, Base UI, Tailwind v4, shadcn scaffolding, Lexical for the chat composer.
- Rust: edition 2024 on server and overlay, edition 2021 on the Tauri crate.
