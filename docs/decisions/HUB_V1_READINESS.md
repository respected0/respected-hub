# Hub V1 readiness audit

Readiness of Respected Hub against a notional V1 for ecosystem management. Every
claim below cites code in this worktree; nothing is inferred. The README labels
several areas "later work", and this audit does not upgrade any of them.

Status legend: **REAL** = implemented and reachable in code; **PARTIAL** =
implemented with a stated limit; **ABSENT** = no code exists.

## Readiness table

| Area | Status | What is real in code | What the README says is later |
| --- | --- | --- | --- |
| Installation / lifecycle | ABSENT | No install/update/repair/removal control exists. `src-tauri/tauri.conf.json` sets `bundle.active: false` with empty `icon`, so no packaged artifact is produced. The only lifecycle-adjacent action is "discover sibling product sources" via `Settings`/`get_settings` in `src-tauri/src/lib.rs` and the source-root form in `src/views/Connections.tsx`; the registry `APPS` in `src/apps.ts` only maps app id to RPP product. | "Product installation/update/repair/removal, integration lifecycle ... are later work. Their controls are not represented as working features. Bundling is disabled for this development stage." |
| First-run setup | PARTIAL | `src/views/BrainSetup.tsx` `BrainSetup` runs a 4-step wizard (`STEPS`) that writes only SecondBrain engine settings through RPP (`settings.schema`/`settings.readback`/`settings.apply`) and verifies with `verifyReadback`; it does not run product installers. `src/homeModel.ts` `homeMode` returns `first-launch`/`setup-incomplete` from live status, and `src/views/Home.tsx` emits the matching CTA. Off-Tauri it degrades to `preview` and states no data is shown. The assistant name is Hub-local only (`NAME_KEY = hub.secondbrain.assistantName` in localStorage). | "no user-facing CLI, permanent service or setup wizard is added" applies to the engine; the SecondBrain settings wizard is explicitly scoped to engine-supported settings only (the wizard states, in Turkish, that vault selection and product installation belong to SecondBrain's own installer and are not yet managed by Hub). |
| UX | REAL | Navigation driven by `src/apps.ts` `APPS` + `ORCHESTRATOR_PAGES`; `src/views/Home.tsx` hero per `homeMode`; `src/useEcosystem.ts` polls every 8 s and maps RPP `manifest`/`health`/`settings.readback` via `summarizeProduct` (`src/views/productsModel.ts`); `src/views/Products.tsx` `ProductCard`/`ProductPanel` show status, checks, capabilities and a schema-driven settings form with validate/apply + readback verification. Browser preview is labelled and fabricates no statuses (`native` guard). | Browser preview intentionally shows no product data; "Native IPC and folder selection must be tested in the actual desktop build." |
| Product integration | REAL (bounded) | `src-tauri/src/rpp.rs` implements the RPP v1.0 client: allow-listed products `["orchestrator","secondbrain"]` and ops `["manifest","health","settings.schema","settings.validate","settings.apply","settings.readback"]`; invalid product/op rejected before any spawn; secret-looking env vars stripped; 30 s timeout; 1 MiB output cap. `src-tauri/src/lib.rs` exposes `rpp_request`, `engine_request` (allow-listed actions incl. `campaign-start`/`campaign-status`), `campaign_list`, `product_status`; `valid_campaign_id` guards identities. `src/productClient.ts` `rppCall`/`rppTry` is the typed client; `src/rpp.ts` owns `parseResponse`/`negotiate`. | "The native Rust bridge invokes the engine's private JSON interface; no shell plugin, user-facing CLI, permanent service or setup wizard". Product install/lifecycle, visual feature toggles, signed catalog, RPP, SIL and finite Lifecycle Core are later work. |
| Testing | PARTIAL | TypeScript units under `tests/` (11 files: `productClient.test.ts`, `productsModel.test.ts`, `rpp.test.ts`, `campaigns.test.ts`, `homeModel.test.ts`, `tasks.test.ts`, orchestrator workspace/agents/evidence/overview, `appNav.test.ts`), run by `node --test tests/*.test.ts` (`package.json` `test`). Rust units in `src-tauri/src/lib.rs` (data-root) and `src-tauri/src/rpp.rs` (allow-lists, op-id policy, request shape, secret-env filtering). Real engine e2e in `src-tauri/tests/rpp_e2e.rs` and `src-tauri/tests/campaign_e2e.rs`. `npm run build` = `tsc --noEmit && vite build`. | Native IPC and folder selection require the real desktop build; no OS-level acceptance is claimed from a TypeScript build. |
| Release-readiness | ABSENT | `src-tauri/tauri.conf.json` `bundle.active: false`; no updater plugin in `src-tauri/Cargo.toml` (`tauri` + `tauri-plugin-dialog` only); no signing/notarization/installer config; window titled "Respected Hub - Development". | "product install/update ... independent gates"; "Bundling is disabled for this development stage. Do not claim Windows/macOS/Linux release acceptance from a TypeScript build alone." |

## Recorded dev evidence

No Windows/macOS/Linux release validation exists anywhere in this worktree. What
is recorded here is development-only: TypeScript unit tests, Rust unit tests, and
engine e2e tests that run the actual product engines against temporary data roots
(`src-tauri/tests/rpp_e2e.rs`, `campaign_e2e.rs`). `docs/decisions/HUB_NATIVE_RPP.md`
states "Windows is the verified platform; `cfg(windows)` is used only for the
`CREATE_NO_WINDOW` creation flag" - that is a dev check, not a signed or installed
release acceptance. `docs/records/` does not exist in this worktree; the README
points extraction evidence at `../secondbrain/docs/records/...` (outside this
worktree, not read here). **No Windows/macOS/Linux release validation exists beyond
these recorded development checks.**

## Ranked next work

1. Native desktop smoke check (open app, connect a real engine, load settings,
   verify readback) - within existing scope.
2. Keep browser-preview honesty guard and status polling correct as views grow -
   within existing scope.
3. SecondBrain settings-wizard edge cases (fresh install, partial readback, schema
   drift) via the existing RPP seam - within existing scope.
4. Campaign observation polish (list/detail/chain rendering, supervisor-alive
   indicator) - within existing scope.
5. Recording extraction/dev evidence under `docs/records` (the README-referenced
   records live in the sibling Brain repo, not here) - within existing scope.
6. Product installation/update/repair/removal, integration lifecycle, visual
   feature toggles, signed catalog, RPP/SIL/Lifecycle Core, and any apply/repair
   control - needs human authorization.
7. Tauri bundling, icons and code signing / notarization / updater (the
   installer and signed-catalog gates) - needs human authorization.
8. Native OS acceptance runs on Windows/macOS/Linux and a recorded result per
   platform - needs human authorization.

Tagging rule applied above: any item that adds install/update/repair/removal, a
setup wizard beyond the current engine-settings wizard, apply controls, or a signed
catalog is marked "needs human authorization" (6-8). Items 1-5 stay inside the
currently implemented development scope.

