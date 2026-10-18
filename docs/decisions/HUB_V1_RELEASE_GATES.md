# Hub V1 release gates

Matrix of the gates that a Respected Hub V1 release must pass, and their state in
this worktree. It complements `HUB_V1_READINESS.md` (feature readiness) with an
explicit release/compliance view. Every state below cites code or a recorded run in
this worktree; nothing is inferred.

State legend:

- **OPEN** - not satisfied. Either no run is recorded, or the capability does not
  exist yet. OPEN is the default whenever a recorded run cannot be pointed at.
- **DEV-VERIFIED** - a development-only run is recorded (exact command given). It
  was run **only on Windows 11 against this dev source tree**; it is not a signed,
  installed or per-OS release acceptance.
- **NOT APPLICABLE** - the gate cannot and need not be satisfied for the current
  deliverable.

**No gate in this matrix is closed.** DEV-VERIFIED is a development checkpoint, not
release acceptance; on its own it never marks a gate closed.

## Gate matrix

| Gate | State | Recorded command / basis | Evidence that would close it |
| --- | --- | --- | --- |
| Build (frontend) | OPEN | No run recorded. The command is defined in `package.json`: `npm run build` (`tsc --noEmit && vite build`). | A recorded `npm run build` exit 0 on a clean checkout, plus the built `dist/` artifact referenced from a dated record under `docs/records/`. |
| Unit tests (TypeScript) | DEV-VERIFIED | `node --test tests/*.test.ts` (`package.json` `test`). Recorded in `HUB_V1_READINESS.md` "Recorded dev evidence"; run only on Windows 11 dev source. | The same suite green in CI on each target OS, with the run output stored under `docs/records/`. |
| Rust tests | DEV-VERIFIED | `cargo test --manifest-path src-tauri/Cargo.toml --lib --tests`. Recorded in `HUB_V1_READINESS.md` "Recorded dev evidence"; run only on Windows 11 dev source. | The same run green on each target OS in CI, output stored under `docs/records/`. |
| RPP conformance | DEV-VERIFIED | `cargo test --manifest-path src-tauri/Cargo.toml --test rpp_e2e --test campaign_e2e` (real engines against temporary data roots: manifest, health, schema, validate, apply + readback + idempotent replay, unknown op, missing root, protocol mismatch). Run only on Windows 11 dev source. | A recorded conformance run against the pinned engine protocol version per target OS, with the protocol version and engine revision captured in the evidence record. |
| Native Tauri smoke | OPEN | No run recorded. `README.md` states "Native IPC and folder selection must be tested in the actual desktop build"; `HUB_V1_READINESS.md` ranks it as next work #1. | A recorded manual/automated run of `npm run tauri -- dev` in the real desktop build: open app, connect a live engine, load settings, verify readback, then cancel a task and confirm engine-owned state survives Hub close. |
| Per-OS validation (Windows/macOS/Linux) | OPEN | No Windows/macOS/Linux release validation exists. `HUB_NATIVE_RPP.md` "Windows is the verified platform" is a dev check, not release acceptance; `README.md` forbids claiming OS acceptance from a TypeScript build. | A pass/fail result recorded once per OS on the release candidate build, stored under `docs/records/`. |
| Signing (code signing / notarization) | OPEN | No signing config exists: `src-tauri/tauri.conf.json` has `bundle.active: false`, empty `icon`, no signing identity; `src-tauri/Cargo.toml` has no signing/updater plugin. `README.md` calls signing/notarization a later work. | Signed (and, for macOS, notarized) release artifacts from the release pipeline, with the certificate/identity and verification command recorded under `docs/records/`. |
| Installer (bundled artifact) | OPEN | `src-tauri/tauri.conf.json` `bundle.active: false`; `README.md`: "Bundling is disabled for this development stage." No installer is produced. | A produced, signed installer per target OS, its checksum recorded, and a clean-machine install/uninstall run stored under `docs/records/`. |
| Update | OPEN | No updater exists: no updater plugin in `src-tauri/Cargo.toml` (`tauri` + `tauri-plugin-dialog` only); `README.md` lists product update as later work. | An update from an installed older release to the candidate, verified end to end (feed + signature + applied version), recorded under `docs/records/`. |
| Upgrade from dev | OPEN | No installer/updater exists, so no dev-to-release upgrade path can run. Bundling is disabled (`bundle.active: false`). | A recorded upgrade from a dev-installed build to the signed release build that preserves Hub-local settings and independent engine state, stored under `docs/records/`. |

## Notes

- The three DEV-VERIFIED rows are the entirety of the recorded dev evidence; every
  other gate is OPEN because no run is recorded or the capability does not exist.
- `docs/records/` does not exist in this worktree; the README points extraction
  evidence at the sibling Brain repo. Closing any gate above depends on first
  recording its run locally.
- Items needing human authorization (bundling, icons, signing/notarization,
  updater, per-OS acceptance) are listed as next work 6-8 in `HUB_V1_READINESS.md`;
  they are not attempted here.
