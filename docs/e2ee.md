# End-to-end encryption for Hermes messages

Research + implementation plan for adding E2EE to text messaging. Voice
is already E2EE via WebRTC DTLS/SRTP — no work needed there.

## Design decisions (locked)

- **Scope**: DMs *and* group channels from day one.
- **Trust boundary**: server never sees plaintext. No server-side
  search, moderation, or content-based push notifications. History
  decrypts only on user devices.
- **Multi-device**: supported from day one.

These three together rule out the "simple 1:1 ECDH + AES-GCM" recipe
(no forward secrecy, no group support, no key rotation) and point at
**MLS (RFC 9420) via OpenMLS**.

## Current state of the codebase

Text messaging isn't shipped yet:

- [apps/server/src/routes/api/message.rs](../apps/server/src/routes/api/message.rs)
  is empty.
- [apps/server/migrations/000_init.sql](../apps/server/migrations/000_init.sql)
  has `users`, `grants`, `refresh_tokens`, `keys` — but no `channels`,
  `servers`, `messages`, or `dm_participants`.
- [api/proto/hermes.proto](../api/proto/hermes.proto) defines voice/RTC
  envelopes only — no text message types.
- [use-channel-text-mutation.ts](../apps/desktop-client/src/hooks/mutations/use-channel-text-mutation.ts)
  is a 5-second `sleep()` mock; no real send path.

Dead scaffolding that hints at prior design intent:

- `keys` table with `(id, user_id, public_key TEXT)` — bare; no
  algorithm, no device_id, no expiry.
- [`UserPublicKey`](../apps/server/src/models/user.rs) struct with
  unused CRUD methods.
- Commit `29c6f88 Update TODO — E2E message encryption` and
  [TODO.md](../TODO.md) mark E2EE as post-v1. The desktop-client TODO
  links a Medium article on the simple ECDH recipe.
- Multi-device is already anticipated: refresh-token families
  ([apps/server/src/models/auth.rs](../apps/server/src/models/auth.rs))
  with `revoke_family`.

**Consequence**: this is greenfield for text — no plaintext protocol
to migrate. The plan below designs the message pipeline **with E2EE
baked in from day one**, so there's no v1-to-v2 crypto migration.

## Why MLS

### The simpler "1:1 ECDH + AES-GCM" recipe (rejected)

Referenced from a Medium walkthrough:

- ECDH on P-256, HKDF-SHA-256, AES-GCM-256, 96-bit random IV, browser
  Web Crypto API.
- **1:1 only. No forward secrecy. No group support. No key rotation.**
  Public keys fetched from server unauthenticated.

Rejected because the locked design decisions demand groups, FS, and
multi-device.

### MLS (RFC 9420)

- Tree-based key encapsulation → post-compromise security in O(log n)
  group ops vs O(n²) with sender-key approaches. Scales from 2 to
  thousands of members (§1, Abstract).
- Ciphersuite: HPKE + hash + MAC + signature. Typical choice: X25519 +
  SHA-256 + Ed25519 (§5.1).
- Two server roles (§3):
  - **Authentication Service (AS)** — trusted; verifies identity ↔
    signing-key bindings. Fits our OAuth server: it already knows
    `user_id` (uuid) and can bind an MLS identity key to it.
  - **Delivery Service (DS)** — untrusted; fans out MLS frames
    verbatim. Fits our existing WebSocket relay.
- Client state to persist (§3.1, §4.2, §6.3.1): per-group epoch,
  ratchet tree, key schedule secrets, generation counters.
  **Generation-counter loss risks catastrophic key reuse** —
  persistence must survive crashes.
- Mainstream Rust implementation: **[OpenMLS](https://github.com/openmls/openmls)**.
  C++: `mlspp`. No mature browser-JS implementation today.

**Why MLS fits Hermes**: the desktop client is Tauri (Rust + React).
OpenMLS runs directly in the Tauri Rust half — no WASM, no JS crypto
gymnastics. The React side calls Tauri commands (`encrypt_message`,
`decrypt_message`, `process_commit`) over IPC.

## Architecture

```
Tauri client                            Hermes server
────────────                            ─────────────
[React UI]                              [Actix-web]
    │                                       │
    │ IPC: encrypt("hi", channel_id)        │
    ▼                                       │
[Tauri Rust: OpenMLS state]                 │
    │  encrypt w/ current epoch key         │
    │  produce MLSApplicationMessage        │
    ▼                                       │
[POST /api/v1/messages] ────────────────► [key_packages + messages tables]
                                            │ ciphertext + metadata only
    ┌─────────────────────── WS fanout ◄────┤ Delivery Service role
    ▼                                       │
[Tauri Rust: OpenMLS decrypt]               │
    │                                       │
    ▼                                       │
[React UI shows plaintext]                  │
```

Server sees ciphertext + routing metadata (channel_id, sender
device_id, epoch, timestamp) only.

## Data model changes

New migration `apps/server/migrations/001_e2ee.sql` (don't edit
`000_init.sql`):

- **`key_packages`** — one row per device × validity window:
  `id, user_id, device_id, ciphersuite, key_package_bytes BLOB,
  created_at, expires_at, consumed_at NULL`.
- **`channels`** — `id, kind ('dm'|'text'), created_at, mls_group_id`.
- **`channel_members`** — `channel_id, user_id, added_at`.
- **`messages`** — ciphertext only: `id, channel_id, sender_user_id,
  sender_device_id, mls_epoch, wire_format ('application'|'commit'|
  'welcome'), ciphertext BLOB, sent_at`. **No `content` column, ever.**

## Proto changes

Extend [`hermes.proto`](../api/proto/hermes.proto) `Envelope` with:

- `MLSApplicationMessage { channel_id, sender_device_id, epoch, ciphertext }`
- `MLSCommit { channel_id, ciphertext }` — group state changes
- `MLSWelcome { channel_id, target_user_id, target_device_id, ciphertext }`
- `KeyPackageSupplyLow { remaining }` — server → client

## Client responsibilities (Tauri Rust)

New module at `apps/desktop-client/src-tauri/src/mls/`:

- Depend on `openmls`, `openmls_traits`, `openmls_rust_crypto`,
  `openmls_sqlite_storage` (or hand-rolled sqlite via `rusqlite`).
- Store MLS state in a local sqlite at `%APPDATA%/hermes/mls-state.db`
  via the Tauri path API — one row per group, one per own KeyPackage,
  one per pending Commit.
- Tauri commands:
  - `create_identity()` — generate identity keypair on first login per
    device.
  - `create_key_package()` — generate + return KeyPackage bytes.
  - `create_group(channel_id, member_user_ids)` — fetch KeyPackages,
    build group, return Welcome per member.
  - `process_welcome(welcome_bytes)` — accept a group invite.
  - `encrypt_message(channel_id, plaintext)` → ciphertext.
  - `decrypt_message(channel_id, ciphertext, sender_device_id, epoch)`
    → plaintext.
  - `process_commit(channel_id, commit_bytes)` — advance epoch.
- OAuth identity plugs in as the AS credential: identity keypair signs
  a request to `POST /api/v1/key_packages` authenticated by the OAuth
  access token; server records `user_id + device_id + identity_key`.

## Server responsibilities

**Authentication Service role:**
- `POST /api/v1/key_packages` — accept a KeyPackage from an
  authenticated device; verify the embedded credential names the
  caller's `user_id`. Reject on signature/user mismatch.
- `GET /api/v1/users/{id}/key_packages` — return one unconsumed
  KeyPackage per device the user has (mark consumed on read).
- `GET /api/v1/devices/{user_id}` — list device_ids for a user.

**Delivery Service role:**
- `POST /api/v1/channels/{id}/messages` — accept ciphertext, persist,
  fanout to member sessions via WS.
- `GET /api/v1/channels/{id}/messages` — return ciphertext history
  (auth-gated).
- WS relays MLS frames verbatim.

Server never calls into MLS crypto — it's a signed-blob storage +
router.

## Multi-device story

- Each device generates its own identity keypair on first login
  (bound to the OAuth `sub` claim).
- Each device uploads its own supply of KeyPackages.
- Each device is a separate MLS member — sees its own view of the
  ratchet tree.
- **Adding a new device**: user logs in on device B → device B uploads
  KeyPackage → device A issues a Commit + Welcome to add device B to
  every group the user is in.
- **Losing a device**: user revokes device B server-side → AS marks
  device B's KeyPackages consumed → other devices Commit to remove
  device B from every group.
- The refresh-token `family_id` on
  [`refresh_tokens`](../apps/server/src/models/auth.rs) becomes the
  natural `device_id`.

## Trust model

- The Hermes AS binds `user_id ↔ identity_key`. Users trust the AS to
  not maliciously swap keys — this is Signal's model.
- Recommend adding **safety numbers** (RFC 9420 §5.3 exporter secret
  → short human-verifiable string) in a later phase for out-of-band
  identity verification.

## What this rules out (product-level trade-offs)

- **No server-side message search.** Search is a device-local index
  problem.
- **No server-side moderation** based on content. Moderation must be
  behavior/report-driven.
- **No content-based push notifications.** Push tells "new message in
  channel X from user Y", never the content.
- **Message history loss on device state loss** (unless recovery is
  designed — MLS doesn't provide this natively).
- **No web/browser client** without a WASM build of OpenMLS, which
  isn't production-ready yet. Tauri-only for the foreseeable future.

## Phased implementation

**Phase 1 — plumbing.**
- New migration: `channels`, `channel_members`, `messages`,
  `key_packages`. Remove the dead `keys` table + model.
- Extend `hermes.proto` with MLS frame types; regen TS + Rust.
- Add OpenMLS deps to the Tauri side.
- Implement `create_identity` + `create_key_package` Tauri commands +
  local sqlite state.
- Implement `POST /api/v1/key_packages` +
  `GET /api/v1/users/{id}/key_packages`.
- Wire the OAuth `sub` claim into KeyPackage upload auth.

**Phase 2 — 1:1 DM E2EE.**
- `create_group(dm_channel_id, [peer_user_id])` — fetches peer
  KeyPackage, builds group, produces Welcome.
- `POST /api/v1/channels/{id}/messages` + WS fanout.
- Replace the 5-second mock in
  [use-channel-text-mutation.ts](../apps/desktop-client/src/hooks/mutations/use-channel-text-mutation.ts).
- Decrypt on receive; render plaintext in
  [message components](../apps/desktop-client/src/components/chat/message/).

**Phase 3 — group channels.**
- Multi-member `create_group` and `add_member` / `remove_member` flows
  (Commit + Welcome fanout).
- History gate: new members can't decrypt pre-join messages. UI should
  show "joined at" boundary in history.

**Phase 4 — multi-device.**
- Per-device KeyPackages (Phase 1 already supports this if written
  right).
- Add-device UI flow.
- Device-revocation tied to OAuth token-family revocation.

**Phase 5 — polish / harder problems.**
- Attachments: per-file random AES-GCM key, key delivered inside the
  encrypted message body; ciphertext stored as opaque blobs (in the
  garage/S3 bucket scaffolded in `deployments/`).
- Safety numbers UI.
- Backup/recovery design (open problem — likely a re-invite from
  another logged-in device).

## Critical files to touch

**Server:**
- New migration `apps/server/migrations/001_e2ee.sql`.
- `apps/server/src/models/{keys.rs, channel.rs, message.rs}` — new
  or renamed.
- `apps/server/src/routes/api/{message.rs (empty today), channel.rs,
  key_package.rs (new)}`.
- [`websocket.rs`](../apps/server/src/routes/websocket.rs) — MLS
  envelope handling.
- [`hermes.proto`](../api/proto/hermes.proto) — MLS types.

**Desktop client (Tauri Rust):**
- [Cargo.toml](../apps/desktop-client/src-tauri/Cargo.toml) —
  `openmls`, `openmls_traits`, `openmls_rust_crypto`,
  `openmls_sqlite_storage`.
- `apps/desktop-client/src-tauri/src/mls/` — new module.
- `apps/desktop-client/src-tauri/src/cmd/mls.rs` — Tauri commands.

**Desktop client (React):**
- `apps/desktop-client/src/lib/mls.ts` — thin wrapper over Tauri IPC.
- [use-channel-text-mutation.ts](../apps/desktop-client/src/hooks/mutations/use-channel-text-mutation.ts)
  — real send path.
- [components/chat/message/](../apps/desktop-client/src/components/chat/message/)
  — decrypt on receive.

## Verification

- **Rust unit tests** on the Tauri MLS module: encrypt/decrypt
  round-trip, welcome + join, commit + epoch advance. OpenMLS ships
  test vectors; wire those in.
- **Integration — two devices, one DM channel**: log in as user A on
  two Tauri windows, send messages, verify decrypt on both.
- **Integration — group of 3, then add a 4th**: verify the 4th can
  decrypt only post-join messages; earlier history reads as
  ciphertext-not-decryptable.
- **Server-side**: attempt to `GET /api/v1/channels/{id}/messages` and
  confirm the response body is opaque bytes, no readable text. Schema
  has no `content` column.
- **Manual**: intercept WS traffic in Tauri dev tools and confirm no
  plaintext appears in any frame.

## Open questions to answer before Phase 1

1. **OpenMLS storage backend.** `openmls_sqlite_storage` exists but
   is early-stage. Alternative: `openmls_memory_storage` + custom
   `serde` blob persistence.
2. **KeyPackage supply pump.** Server signals a device when its
   supply drops below threshold, or the device polls on connect?
   Simplest: WS event `KeyPackageSupplyLow`.
3. **Concurrent Commits.** MLS handles Commit ordering via the DS
   linearizing them; verify our WS fanout preserves single-writer
   semantics per group.
4. **Message retention.** New members can't decrypt old messages
   (forward-secrecy property). Store forever, or trim server-side
   once every subscribed device has ACK'd? Product decision.
5. **Attachments cross the "server never sees plaintext" line** unless
   we also encrypt uploaded blobs. The garage/S3 config in
   `deployments/` needs matching client-side encryption before Phase 5.

## Primary sources

- **RFC 9420 — The Messaging Layer Security (MLS) Protocol**:
  https://datatracker.ietf.org/doc/html/rfc9420
- **OpenMLS (Rust implementation)**: https://github.com/openmls/openmls
- **HPKE — RFC 9180** (used inside MLS ciphersuites):
  https://datatracker.ietf.org/doc/html/rfc9180
