# Hermes v1 TODO

Target: **v1 ship to friends** — self-hosted single-instance Discord-like clone.

## Scope

**In v1:** core text, core voice (≤5 per channel, mesh + TURN), DMs, invites, presence + typing, overlay client (voice-status only), roles/permissions (server-wide, ~6-8 boolean perms).

**Out of v1:** attachments, notifications, E2EE, screenshare/video, PTT/VAD, custom status, categories-as-table.

## Architecture decisions

- Flat schema: `servers` → `channels(kind: text|voice|dm, category text nullable)` → `messages`. Categories are a metadata string, not a table.
- Roles: custom server-wide roles with a bitmask of ~6-8 booleans. No per-channel overrides.
- Text delivery: REST POST for send, WS fanout with full message payload (no refetch). Edits/deletes also as WS events. Soft-delete.
- Voice: peer mesh, up to ~5 per channel, coturn as TURN fallback.
- Overlay client: voice-status widget only (channel members + speaking indicator).
- Invites: codes with `expires_at`, `max_uses`, revocation.
- Presence: WS-connection-derived online/offline + typing indicator (5s server-side timeout).
- Deploy: single box, docker-compose (hermes-server + caddy + coturn + sqlite volume).
- Tests: smoke coverage only (OAuth flow, message round-trip, voice signaling handshake). No CI required.

---

## Phase 1 — Schema foundation (~1 day)

- [x] Migration `001_core.sql` adding: `servers`, `server_members`, `channels(kind text|voice|dm, category text nullable, server_id nullable)`, `dm_participants`, `messages(edited_at, deleted_at)`, `roles`, `role_members`, `invites(expires_at, max_uses, uses)`
- [x] Regenerate sqlx offline query cache

## Phase 2 — Fill the NotImplemented REST stubs (~4-5 days)

- [x] `servers` CRUD in [apps/server/src/routes/api/channel.rs](apps/server/src/routes/api/channel.rs) (owner check on write)
- [x] `channels` CRUD (kind text/voice, category string, scoped to server)
- [x] `roles` CRUD + assign/unassign; store perms as bitmask; server-side helper `has_perm(user, server, PERM)`
- [X] `messages`: `POST /channel/{id}/messages`, `PATCH` (edit), `DELETE` (soft), `GET` cursor-paginated on `(ts, id)`, page size 50
- [x] `dms`: `POST /dm` (find-or-create peer DM), reuse messages table
- [x] `invites`: create/list/revoke + `POST /invites/{code}/accept`
- [ ] Update [api/openapi.yaml](api/openapi.yaml) + regenerate client with `pnpm openapi-ts`

## Phase 3 — WS text fanout + presence (~3-4 days)

- [ ] Extend [api/proto/hermes.proto](api/proto/hermes.proto) with: `MessageEvent{new|edit|delete}`, `TypingEvent`, `PresenceEvent`
- [ ] Regenerate: `pnpm protoc` + rebuild server (prost)
- [ ] `SessionRegistry` in server state: `user_id -> Vec<Session>`; track `channel_id -> members` cache
- [ ] After message write: fan out full payload to online channel members
- [ ] Presence: mark online on WS connect, offline on disconnect, broadcast to server co-members
- [ ] Typing: WS event, 5s server-side expiry, throttle 1/sec client-side

## Phase 4 — Voice signaling actually relays (~2 days)

- [ ] Replace the `log::debug!` stubs in [apps/server/src/routes/websocket.rs](apps/server/src/routes/websocket.rs#L52-L65) with real fanout
- [ ] In-memory `voice_channel_id -> Vec<user_id>` state
- [ ] On `VoiceChannelRequest`: update state, emit `VoiceChannelUserEvent` to other members
- [ ] Route `RtcEvent` / `RtcNewCandidate` to the `target` user's session

## Phase 5 — Client wire-up (~5-6 days)

- [ ] Replace query stubs in [apps/desktop-client/src/lib/api/queries.ts](apps/desktop-client/src/lib/api/queries.ts) with real endpoints
- [ ] Handle new WS events in [apps/desktop-client/src/lib/core/app.ts](apps/desktop-client/src/lib/core/app.ts) — apply message events directly to TanStack Query cache (no refetch)
- [ ] Server + channel sidebar list from real data
- [ ] Message composer: optimistic insert + reconciliation on fanout echo
- [ ] Roles UI in server settings drawer — create/edit, assign to members
- [ ] Invites UI — generate/list/revoke; `/invite/:code` route to accept
- [ ] Typing indicator UI
- [ ] Presence dots on member list
- [ ] Edit/delete UI (hover controls on messages, "(edited)" marker)

## Phase 6 — Voice UI + overlay (~4-5 days)

- [ ] Voice channel join/leave, mute/deafen buttons in [apps/desktop-client/src/routes/voice.$roomId.tsx](apps/desktop-client/src/routes/voice.$roomId.tsx)
- [ ] Speaking indicators (RMS threshold from existing web audio graph)
- [ ] [apps/overlay-client](apps/overlay-client/): connect to WS using token passed from main client via IPC/env; subscribe to voice state only; render egui widget with members + speaking dots
- [ ] Tauri command to launch/kill overlay + pass token

## Phase 7 — Deploy story (~2 days)

- [ ] Add `coturn` service to [depolyments/docker-compose-local.yml](depolyments/docker-compose-local.yml); static shared-secret creds
- [ ] Expose STUN/TURN URLs from server config; client reads them for `RTCConfiguration.iceServers`
- [ ] Move CORS origin from hardcoded `http://localhost:1420` in [apps/server/src/main.rs](apps/server/src/main.rs) to env
- [ ] Fix stale root [package.json](package.json) scripts (`packages/` -> `apps/`)
- [ ] Prod-shaped compose file (persistent sqlite volume, TLS via caddy)

## Phase 8 — Smoke tests (~1 day)

- [ ] Server test: OAuth authorization-code + PKCE round-trip end to end
- [ ] Server test: two mock WS clients — one sends message via REST, other receives fanout
- [ ] Server test: RTC signaling relay — client A's `RtcEvent{target: B}` reaches client B

## Phase 9 — Reconcile + polish (~1-2 days)

- [ ] Decide fate of [apps/desktop-client/src/routes/settings/_settingsLayout/voice.tsx](apps/desktop-client/src/routes/settings/_settingsLayout/voice.tsx) (PTT/VAD is out of v1 — hide or leave stub?)
- [ ] Same for [notifications.tsx](apps/desktop-client/src/routes/settings/_settingsLayout/notifications.tsx) and [overlay-games.tsx](apps/desktop-client/src/routes/settings/_settingsLayout/overlay-games.tsx)
- [ ] Rewrite [README.md](README.md) with quickstart + arch blurb
- [ ] Rename `depolyments/` -> `deployments/` (cosmetic, low priority)

---

**Rough total: ~4-5 weeks solo**, assuming no rabbit holes. Most likely rabbit holes: (a) TURN/NAT reality for friends on real networks, (b) overlay-client IPC handoff.
