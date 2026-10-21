# Hub native RPP integration

## Context

Respected Hub is a Tauri 2 desktop app (Rust backend + React/TypeScript client).
The Orchestrator and SecondBrain products already expose a private RPP v1.0 entry
point: one JSON request on stdin, one JSON response line on stdout, with no network
listener and no daemon. The Hub must reach both products from its own Rust backend
without opening HTTP ports, without a public CLI and without adding dependencies.

## Decision

- `src-tauri/src/rpp.rs` owns the native RPP client. `rpp_call(settings, product,
  op, payload, operation_id)` always returns a complete RPP response envelope
  (`rpp`, `ok`, `operation_id`, `result`, `error`) and never panics. `rpp_call_version`
  is the same call with an explicit protocol version, used to prove that a mismatch
  is surfaced by the engine as `PROTOCOL_UNSUPPORTED`.
- Products are allow-listed to `orchestrator` and `secondbrain`; operations to
  `manifest`, `health`, `settings.schema`, `settings.validate`, `settings.apply`
  and `settings.readback`. Anything else returns `INVALID_PRODUCT` / `INVALID_OP`
  before any process is started.
- `settings.apply` without a valid `operation_id` gets a generated id (8..64 chars
  from `[A-Za-z0-9_-]`). Other operations keep the id they were given, or `null`.
- The engine is spawned directly (`Command`, no shell) with explicit argv, `cwd`
  and `PYTHONPATH`, `stdin` piped, a 30 second timeout with kill, and output capped
  at 1 MiB. Secret-looking environment variables (names containing key/token/secret/
  passw/credential/auth/cookie/session) are not forwarded to the child.
- Native failures become envelopes: `ENGINE_NOT_FOUND` (root/module missing),
  `ENGINE_UNAVAILABLE` (cannot start the engine), `ENGINE_TIMEOUT`, `BAD_RESPONSE`
  (no/invalid JSON or an envelope that does not match the contract). The process
  pump is extracted from `call_engine` in `lib.rs`, so the existing Orchestrator
  path keeps identical behaviour and error strings.
- `lib.rs` exposes `#[tauri::command] async fn rpp_request(...)`, which loads
  settings, runs `rpp_call` in `spawn_blocking` and returns the envelope as JSON.
  `Settings` is public, gains `#[serde(default)] brain_data_root: Option<PathBuf>`
  (backwards compatible with an existing `connections.json`) and the default brain
  sibling folder is `respected-secondbrain`.
- The SecondBrain data root is passed with `--data-root` only when `brain_data_root`
  is set. Each call is request/response with a child process that ends by itself,
  so nothing is tied to the Hub window lifetime.
- `src/shared/productClient.ts` invokes `rpp_request` with `{product, op, payload,
  operationId}` and passes the reply through the strict `parseResponse` from
  `src/shared/rpp.ts`. `rppCallVia(transport, ...)` is the injectable seam used by the
  TypeScript unit test; `rppCall` is the production entry point. Outside Tauri the
  call still rejects with a clear message.

## Evidence

- Rust unit tests in `rpp.rs`: allow-lists, operation-id policy, request shape and
  secret-environment filtering (no process is spawned).
- Real end-to-end tests in `src-tauri/tests/rpp_e2e.rs` run the actual engines
  against temporary data roots: manifest, health, schema, validate (unknown key),
  apply + readback + idempotent replay, unknown op, missing engine root and
  protocol mismatch.
- TypeScript unit test `tests/productClient.test.ts` exercises transport forwarding
  and strict reply parsing.

## Scope and limits

Windows is the verified platform; `cfg(windows)` is used only for the
`CREATE_NO_WINDOW` creation flag. No new dependencies, no HTTP server, no daemon,
no plugin. The Hub UI entry point is unchanged. `docs/repository_inventory.json`
is maintained by the extraction tooling, not by this change.