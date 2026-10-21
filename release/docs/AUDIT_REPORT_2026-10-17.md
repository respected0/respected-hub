# Respected Ecosystem — Independent Audit Report (2026-10-17)

Scope: Hub (Rust/React), Orchestrator, SecondBrain, installers, MCP/RPP/SIL integrations.
Method: 5 independent read-only AI reviewers (Opus x4, Sonnet x1) over disjoint areas; the Master then re-checked the highest-impact claims directly in code. No tests were re-run, no real user data/config touched, no git operations, no code changed.

Status tags: **VERIFIED** = Master re-read the code and confirmed; **REVIEWER** = reviewer's claim from code reading/experiment in a scratch dir, not re-checked by Master; **RISK** = needs conditions; **UNVERIFIED** = never driven for real.
Several issues were found independently by 2-3 reviewers (marked xN).

## P0 — fix before any release / real-user use

| # | Finding | Evidence | Status | Fix |
|---|---|---|---|---|
| 1 | **Uninstall / repair / update / integration switch-off break once a client rewrites its own config** (x2). Ownership = byte-identical whole file (`~/.claude.json`, `settings.json`, `config.toml`). Claude Code rewrites `.claude.json` every launch; Codex adds `[projects.*]`. Uninstall records the conflict but STILL deletes app files, leaving hooks + MCP entry pointing at a deleted exe in every session. E2E never changes the file between install and uninstall. | `respected-secondbrain/src/respectedbrain/installation/operations.py:74,93,100` (raises "User changed integration"), `uninstall.py:27-44`, `rendering.py:341,435` | VERIFIED (raise sites); end-to-end consequence REVIEWER | Own entries, not files: record JSON pointer/digest of each owned hook/MCP entry; on removal re-read, remove only unchanged owned entries, compare-and-swap write. Never delete app files while external registrations remain. Add test: client mutates config between install and uninstall. |
| 2 | **Orchestrator command execution via worker `.cmd` shims on Windows** (x3). `gemini` resolves to `gemini.CMD`; prompt/profile go through cmd.exe: multi-line brief truncated to first line, `&` / `|` in prompt, profile or diff executes (reproduced with stub `.cmd`: file created). Gemini reviewer/master cannot work on Windows at all (exit 255). | `runner.py:48-85,676`, `processes.py:151`, `roles.py` normalize (profile only length-checked) | REVIEWER (reproduced by 2 reviewers independently) | Refuse `.cmd/.bat` argv prompts or call `node script.js`; pass prompts by stdin/file; restrict profile to `^[A-Za-z0-9._:/-]{1,100}$`. |
| 3 | **Orchestrator MCP gives a calling model host code execution / arbitrary apply**, contradicting `docs/records/2026-10-15-mcp-access.md`. `worker:"custom"`, free `acceptance` commands, `apply_root` (any dir), `engine_source` (any dir with campaign.py → run as supervisor), `final_review.command`; bypass denylist is exact-match for 3 strings only; client can read the patch hash and call `orchestrator_apply` itself. | `mcp_server.py:33-37,83-89`, `runner.py:259-262,496-498,710,855`, `project.py:87-110` | VERIFIED (denylist = 3 exact strings); rest REVIEWER | Over MCP refuse custom worker / free acceptance / apply_root / engine_source / final_review.command unless registered with explicit opt-in; allowlist worker flags; restrict project roots to user-approved ones; correct the record. |
| 4 | **Acceptance checks run worker-written code unsandboxed as the user**; it can read `data_root/.engine-key` and forge signed `maf-approval.json` (bypasses human gate) or read client credentials. | `runner.py:209,710-715`, `processes.py:106-116` | REVIEWER | Run acceptance in the worker sandbox/restricted user; keep signing key out of worker-reachable tree (separate ACL/process). |
| 5 | **SecondBrain MCP `respected_remember` / `quick_capture` write without grant, revert or inbox restriction**; default `confidence: verified` into `500-Knowledge` / `300-Projects`. Handoff claims "authorised write, inbox only, revert" — true only for RPP `memory.write`. Persistent prompt-injection channel across sessions/clients. | `integrations/mcp/server.py:75,145-165,297-301` | VERIFIED | Route MCP writes through `memory.write` path (grant, ownership front-matter, revert), default `unverified`, per-client write permission in wizard. |
| 6 | **Raw transcripts copied unredacted into the vault on every PreCompact**, regardless of `capture_mode=manual`; no retention, no size cap. Template `.gitignore` excludes `Session-Logs/` (partial mitigation for git publishing; sync tools/OneDrive still copy it). | `memory/lifecycle.py:373-392` | VERIFIED | Gate on capture_mode, store in data root (not vault), add retention, redact with existing patterns. |
| 7 | **Launcher for Orchestrator MCP loads modules from the client's working directory** (no `-I`/`-P`); a repo opened in Claude Code/Codex containing `json.py` etc. executes as the user (reproduced). | `respected-hub/src-tauri/src/products/mcp_access.rs:29-34` | VERIFIED (no `-I` in launcher text); exploit REVIEWER (reproduced) | `-I` (or `-P` + `PYTHONSAFEPATH=1`) and `cd` to a fixed dir; same for `.sh`. Note: PYTHONPATH is set explicitly, so use `-s -P` rather than `-I` if PYTHONPATH must survive. |
| 8 | **Package authenticity is not enforced**: Hub never verifies signatures (`signed = signature != null`), UI shows "imzalı"; digest lives inside the same writable `bundle.json`; `--allow-unattested` hardcoded for every SecondBrain setup/update. | `lifecycle.rs:147-172`, `brain.rs:288,303`, `Discover.tsx:19` | VERIFIED (flag); rest REVIEWER | Embedded public key + Ed25519 over digest; pass `--allow-unattested` only for unsigned dev bundles. (Release feature, already known as unsigned.) |

## P1 — high/medium, fix in bug-fixing phase

| # | Finding | Evidence | Status |
|---|---|---|---|
| 9 | No single-instance guard; a 2nd window runs startup recovery that deletes staging and rolls back a live install of window 1. Product installers launch Hub with `--install=` so this is a normal path. Also startup recovery runs synchronously (<=15 min, no window). Fix: single-instance plugin + OS lock on journal; recover on a background thread. | `lib.rs:536`, `lifecycle.rs:503-523`, `brain.rs:350-355` | REVIEWER x2 |
| 10 | Stale writer lock never reclaimed: a killed supervisor makes every later task on that data root fail (`state="stale"` is only a label in the error). | `antigravity_orchestrator.py:160-169` | VERIFIED |
| 11 | Runtime update (2nd) / uninstall breaks the Orchestrator MCP launcher (hardcoded versioned python path; launcher refreshed only for orchestrator; prune deletes it; runtime uninstall has no dependency guard). | `lib.rs:292`, `mcp_access.rs:31`, `lifecycle.rs:402-409`, `MyApps.tsx:26` | REVIEWER x2 |
| 12 | Webview can set the executable Hub spawns (`save_settings python/brain_exe`) and `memory_command` via bridge-allowed `orchestrator settings.apply`; `memory.*` block is only a UX guard. `lifecycle_rollback` op id unvalidated (path traversal into journal/undo paths). | `lib.rs:59-77,346-353`, `rpp.rs:43-57`, `lifecycle.rs:312-315` | REVIEWER x3 |
| 13 | Engine process control: reader-thread joins have no deadline (inherited handles hang forever holding `ENGINE_GATE.write()`), timeout kills only the direct child (no Job Object), stderr uncapped; long semantic pull (65 min) under `ENGINE_GATE.read` blocks all engine calls once an update is queued. | `lib.rs:96-118`, `rpp.rs:117-122,299`, `lib.rs:453` | REVIEWER |
| 14 | Managed Python is verified with `-I` but run without isolation; `PYTHONHOME/PYTHONPATH`/user site leak in (Anaconda users: install passes, every poll fails). | `lib.rs:245`, `lib.rs:~149`, `rpp.rs` | REVIEWER |
| 15 | Settings form overwrites real settings with schema defaults when `settings.readback` fails; Connections form unsavable on fresh install (extra null `brain_*` keys render as required inputs). | `Products.tsx:71-76`, `productsModel.ts:170-181`, `Connections.tsx:22` | REVIEWER |
| 16 | Consent gaps: "Yedekleri temizle" deletes permanently with one click; "Bekleyen oturumları şimdi yakala" sends past sessions to an AI provider without confirm (Rust hardcodes `confirm:true`); semantic "Yeniden dene" re-downloads ~460 MB without consent text (mentions Ollama); SecondBrain uninstall copy says "reversible"/"program copy only" but removes hooks/MCP/tasks and "Geri al" always fails; integrations preview and apply can differ (no plan digest, apply not disabled while loading); wizard applies capture_mode after install (hooks live in auto meanwhile, failure strands user). | `LifecycleControls.tsx:48-68`, `MemoryIndex.tsx:17,66`, `BrainIntegrations.tsx:64`, `BrainInstall.tsx:~58` | REVIEWER |
| 17 | Orchestrator MCP registration/removal blocked for all clients by ONE unparsable client config (Gemini JSON with comments); `disable_all` swallows errors so uninstall leaves entries; launcher never deleted; no in-progress receipt (kill → entry looks foreign forever). | `integrations.py:94-98,179-180,274-276`, `mcp_access.rs:128-138` | REVIEWER |
| 18 | Worktrees/branches (`orchestrator/<run>`) never removed from the user's repo; long task briefs over 32 K chars fail (`WinError 206`; task limit 60 000). | `antigravity_orchestrator.py:259-268`, `runner.py:437,676` | REVIEWER |
| 19 | Evidence integrity: worker can plant `__pycache__/*.pyc` / `node_modules` (excluded from patch, present during acceptance) so checks pass on code not in the patch; verify command runs applied code with full env and no child marker; final master `Bash(python -m respected_orchestrator:*)` allows `create/start` with injected reviewer text. | `antigravity_orchestrator.py:672-685`, `project.py:177-182`, `campaign.py:326-327` | REVIEWER |
| 20 | Secrets/PII flow: unredacted transcripts → summariser → daily/Last-Session → injected at every SessionStart (fixed delimiter, no "untrusted" framing; Codex summariser can read files). Client-config before-images (may contain MCP env keys) kept forever in manifest/journals, survive uninstall. | `memory/flush.py:302-322`, `providers/runner.py:103-115`, `installation/ownership.py:71` | REVIEWER |
| 21 | Orchestrator memory provider pinned to a versioned `dist/<ver>/respectedbrain.exe` path, never refreshed after SecondBrain update/uninstall; old dist folders never pruned; RPP served from newest *staged* catalog dist rather than installed version. | `memory_access.rs:87-89`, `brain.rs:104-113` | REVIEWER x2 |
| 22 | Update detection version-only (digest ignored; `0.1.0-rc1` ranks above `0.1.0`); `[InstallDelete]` missing for `{app}\catalog` (stale bundles after upgrade/downgrade); installers share AppId/AppName so Add/Remove shows "Respected Hub" for all. | `lifecycle.rs:253`, `hub_setup.iss` | REVIEWER |
| 23 | Hub uninstall leaves SecondBrain hooks/MCP/scheduled task/shortcut live with no UI left to remove them (documented in .iss comment, not surfaced to the user). | `hub_setup.iss` | REVIEWER |
| 24 | Dev paths and test overrides active in release: `CARGO_MANIFEST_DIR` default roots and catalog fallback, `RESPECTED_HUB_TEST_HOME/MANAGED_ROOT/CATALOG_ROOT/BRAIN_*`; `product_status` always `development:true`. Masks E2E results on the dev machine. | `lib.rs:50-56,206,215,218` | REVIEWER x2 |
| 25 | Concurrent ops: `is_idle` and journal `begin` not atomic (runtime uninstall vs Orchestrator install; plan vs apply); polling every 8 s spawns ~8 engine processes with no in-flight/sequence guard (stale "Kurulu değil" overwrites; plausible cause of the old "reset to Home"/stale-state flakes). | `lifecycle.rs:357-360`, `useEcosystem.ts:36` | REVIEWER x2 |

## P2 — medium/low

- Backup: no restore path at all, no `restic init`, no password source, verification checks only exit code (`maintenance/backup/backup_restic.py`). Handoff lists backup/restore as "present"; it is effectively unusable. REVIEWER.
- Semantic runtime: exe not re-hashed after install, model checked by size only, interrupted extraction leaves permanently broken runtime, port probed-then-released race, loopback server unauthenticated. REVIEWER.
- Interrupted uninstall/repair journal says "unchanged" although files moved; failed op id retry returns success; bundle size checked after copy; `.pyc` excluded from integrity check. REVIEWER.
- Companion notes (`Threads.md`, `Last-Session.md`) rewritten without cross-session exclusive lock; idempotency record written after note (duplicate on crash); revert unavailable after 500 writes. REVIEWER.
- OneDrive Files-On-Demand / junctioned vault root is rejected as UNSAFE_PATH (UNVERIFIED on this machine: no OneDrive folder).
- SecondBrain MCP tools all take the exclusive writer lease (read tools fail with generic error while install runs; 10 s wait). RPP version/payload validation differs between Orchestrator (lenient) and SecondBrain (strict). JSON configs are re-serialised (BOM, escapes) not byte-preserved; TOML inline-table/CRLF edge cases; config read-modify-write races with the owning client; `Path.home()` vs stored `user_home` in preview.
- Orchestrator: `retry` drops `memory` opt-in; `blocked` projects can't resume; restart accepts any concurrent tree edits; pid reuse; narrow secret redaction; non-git mirror copies `id_rsa/*.pfx/.npmrc`; token budget only checked between stages; `tester`/`coordinator` roles accepted but silently unused (should error); target repo's `.orchestration/templates` and git hooks run in lane. REVIEWER.
- RPP grants are consent, not authentication: any same-user process can mint `memory.grant {write:true}`. Document it.
- Build hygiene: unpinned runtime build, `RECORD` path join unchecked, egg-info shipped, `--skip-app` can reuse stale exe, `%` unescaped in `.cmd` launcher paths, hardcoded `0.1.0`, English errors in Turkish UI, no React component tests, WebView2/MSVC runtime on a clean machine unverified, no SmartScreen note.
- Test blind spots: `real_env_guard.py` does not watch APPDATA/LOCALAPPDATA/Task Scheduler/registry/Start Menu; fake home covers only USERPROFILE/HOME; E2E never enables schedule/shortcut, never edits a client config mid-life, never drives Orchestrator update/uninstall, memory grant/write, rollback, purge, Tasks/Campaigns; `--intent` runs `process.exit` inside `try` (skips finally/guard/cleanup); fixed port 9333; "foreign config intact" check rewrites the file just before uninstall (vacuous); `run_brain_regression.py` reports order-dependent failures as FLAKY with exit 0. `respected-hub/tools/hub_native_e2e.mjs` is a 0-byte stub (the real script is `tools/hub_native_e2e.mjs`; `native_verify.py` exists only in old campaign copies — dead, low).
- **Test-environment hazard (VERIFIED)**: `%TEMP%\hubaccept-0t638rk2\hub\node_modules` is a junction/symlink to the REAL `respected-hub\node_modules`. A recursive delete of the "6.9 GB residue" can destroy the real folder. Remove such junctions with `rmdir` first. Do not clean without checking.

## Not verified at all (missing verification)
Real sessions for Codex, Gemini, Cursor, Antigravity (hook formats, event names, SIL) and `cursor-agent` worker flags; Gemini worker on Windows; macOS/Linux; clean-machine install (WebView2/VC runtime); Orchestrator update/uninstall through the UI; combined E2E (SecondBrain MCP + Orchestrator MCP on one `~/.claude.json` with a simulated Claude rewrite, then uninstall both); OneDrive vault. Backup/restore (code reviewed: not functional as described).

## Deferred release features (already known; not bugs)
Code signing and attestation, remote catalog/auto-update, macOS/Linux, single-instance, history-import UI, model-priority editing, `bounded_recall`/graphrag/session_viz unwired.

## Checked and sound
No XSS path in the webview (no innerHTML/markdown); CSP/capabilities tight; bundle-path traversal rejected; staged copies hash-verified; SecondBrain note-path traversal/ADS/reserved names blocked; `memory.write` fixed inbox + exclusive create; grant token constant-time compare; UTF-8 stdio on Windows; hook program path quoted; signed state uses fsync+replace; uninstall cannot touch vault paths; settings.apply allow-listed.

## Reviewer coverage gaps
Not read in depth: Orchestrator `rpp.py`/`lifecycle.py` internals, MAF correction loop, SecondBrain `compile.py`/`vault_linter`/Task XML, `memory_index.rs`/`memory_semantic.rs`, `BrainSetup.tsx`/`Campaigns.tsx`, Rust tests.

## Recommended fix order (for approval)
1. P0 #1, #6, #7, #5 (user data/config safety + simplest high-impact).
2. P0 #2, #3, #4 (Orchestrator trust boundary) — design decision needed: MCP opt-in model.
3. P1 #9-#14 (lifecycle/process robustness), then UI consent fixes #15-#16.
4. P0 #8 with the signing/release work.
5. Add the combined E2E (client rewrite between install and uninstall) before fixing #1 so it proves the fix.


---

# Resolution status (updated 2026-10-18, end of the bug-fixing phase)

Legend: **FIXED+PROVEN** = changed, with targeted adversarial/regression tests, full suites green. **PARTIAL** = mitigated, a named part remains. **OPEN** = not done. Evidence files: `MASTER_HANDOFF.md` §18.1–18.8. Decision records: `respected-orchestrator/docs/decisions/{ACCEPTANCE_SANDBOX,MCP_PERMISSIONS}.md`, `respected-hub/docs/decisions/{BUNDLE_TRUST,DEV_OVERRIDES_AND_UNINSTALL}.md`, `respected-secondbrain/docs/decisions/PRIVACY_AND_CLIENT_CONFIG.md`.

## P0
| # | Status | What now holds | Evidence |
|---|---|---|---|
| 1 uninstall/repair break after a client rewrites its config | FIXED+PROVEN | Entry-level undo (JSON, TOML, Markdown) via `integrations/delta.py`; only an edit of our own entry is a conflict | `tests/test_config_delta.py` (9), real install→client rewrite→uninstall in `brain_e2e` and `cleanup_e2e` |
| 2 `.cmd` worker shims | FIXED | `.cmd/.bat` workers unavailable; profile charset restricted. Gemini workers stay unavailable on Windows until a direct `node` launch is written | runner/roles tests |
| 3 MCP gives the caller code execution | FIXED+PROVEN | `--allow-privileged` switch; acceptance/verify only from the user's `mcp_acceptance_allow` prefix list (default empty); bypass-flag filter widened | `tests/test_mcp_server.py` |
| 4 acceptance code can read the engine key / forge approvals | FIXED+PROVEN (Windows) | Restricted token + deny ACEs + OWNER RIGHTS ACE; fail closed | `tests/test_sandbox.py` (5, with negative control) |
| 5 SecondBrain MCP writes without grant/inbox/revert | FIXED+PROVEN | `mcp_write` permission (off), tools hidden when off, one inbox, ownership front matter, revert tool, `verified` impossible | `tests/mcp_and_features_test.py` |
| 6 raw transcripts copied into the vault | FIXED+PROVEN | No copy by default; opt-in redacted archive in the data root with retention; summariser input and summary redacted | `tests/test_redact.py`, `f7_transcript_integrity_test.py`, `lifecycle_test.py` |
| 7 MCP launcher module hijack | FIXED | `PYTHONSAFEPATH=1` + own cwd | `mcp_access_e2e` |
| 8 bundle authenticity | FIXED+PROVEN (dev); PRODUCTION KEY OPEN | Ed25519 over product/kind/version/digest, fail closed; dev trust and unsigned opt-in compiled out of production | `trust.rs` tests incl. `cargo test --release --lib trust`, `trust_catalog_e2e` |

## P1
Fixed: #9 single instance (+ forwards `--install`), #10 stale writer lock, #11 runtime/launcher, #12 journal ids and webview exec limits (production `save_settings` refused; `memory_*` settings blocked on the bridge), #13 process-tree kill / pipe grace / gate timeout, #14 Python isolation, #15/#16 UI consent fixes, #17 per-client MCP read errors, #18 command-line limit, #19 bytecode and verify environment, #20 prompt delimiter / redaction / minimal retention, #21 memory provider refresh, #22 `[InstallDelete]`, #24 dev overrides, #25 single-flight refresh.
Still open: worktree/branch cleanup in the user's repo (#18 second half); `blocked` projects cannot be resumed; digest-based update detection and the `0.1.0-rc1` ordering; the three installers share one AppId/name; startup recovery misreports the result of an interrupted uninstall/repair; `is_idle` race between operations; final-master allowlist `Bash(python -m respected_orchestrator:*)` is still broad.

## P2 / category B
Fixed: backup password source + `init`, content-verified backups, **restore into a new folder** (`--restore-to`), semantic runtime staged extraction, MCP busy message, retry keeps memory choice, `tester`/`coordinator` refused, **Hub uninstall cleanup** (`--uninstall-products`, tested with the real exe and with a real Inno uninstall), second launch focuses the open window.
Open: OneDrive placeholder/junction vaults, JSON byte-preservation of re-serialised configs (semantically equal), RPP strictness parity between products, semantic runtime re-hash and loopback port race, companion-note cross-session lock, idempotency record order, Gemini/Codex/Cursor/Antigravity real-session verification, E2E blind spots (APPDATA/Task Scheduler/registry, schedule/shortcut never enabled).

## Release blockers (must be done in the release phase, not here)
1. Production signing key: generate offline, put the public key in `PRODUCTION_KEYS`, sign with `build_hub_installer.py --release-key`, Authenticode for exe and installers. A production build trusts no bundle until then (fail closed).
2. Build and run a **production** installer once (only its compile-time behaviour is proven so far).
3. Real restic backup/restore run (proven only against a functional stand-in).
4. Interactive Inno uninstall prompt ("remove the products too?") exercised by a person (silent paths are proven).
5. Clean-machine prerequisites (WebView2, VC runtime), SmartScreen note, macOS/Linux, remote catalog/auto-update.
6. Real sessions for Codex/Gemini/Cursor/Antigravity (hook formats, workers) — never driven.

## Remaining risks to know about
* The acceptance sandbox protects the engine's key/evidence/settings only; sandboxed code still reads the user's other profile files and the network. Worker CLIs themselves are not in that token. A listed MCP prefix allows any arguments of that command.
* Redaction is best effort; a password written as prose is not recognised, and the summariser CLI keeps its own read-only sandbox.
* JSON configs undone by the delta mechanism are rewritten with two-space indentation.
* Test-environment cleanup that needs the user's authorisation is still not done (see handoff §16: `%TEMP%` residue, Claude/Codex test sessions, the `hubaccept-*` junction that points at the real `node_modules`).

**Release-phase update (2026-10-18):** blocker 1 is partly done (key plumbing, tooling and a production rehearsal; the real key and Authenticode, deferred for budget, remain). See MASTER_HANDOFF.md section 19 and RELEASE_PHASE_PLAN.md.

