# Track B — Premium UI status (2026-10-12)

Integrated from the isolated copy `respected-hub-ux` into `respected-hub/src` (frontend only; no Rust/RPP/IPC/product source touched).
Rollback copies of every replaced file: `respected-hub-ux/backup-real-src/`. Screenshots (real Tauri window): `respected-hub-ux/shots/real/`.

## Done
- Design system `src/shared/design.css` (replaces `style.css` + `hub.css`), tokens, responsive rail (icon rail below 1040px).
- Shell: rail with icons/emblems, top bar with breadcrumb, real health pill, refresh (`App.tsx`, `ui.tsx`, `shellModel.ts`).
- Home, My Apps, Discover, Activity rebuilt on real data only; workspace banner + tabs (`AppWorkspace.tsx`).
- Orchestrator/SecondBrain/Connections/Settings keep their logic; restyled through shared classes.
- Fixed: literal `\uXXXX` text in AppWorkspace JSX (Turkish chars), "Kurulum bekliyor" shown for apps without a setup page.

## Not done / open (do not redo the Done list)
- Hub-managed install/update/repair/uninstall UI: no backend yet (honest note in My Apps).
- Signed catalog in Discover: no backend (shown as unavailable).
- Orchestrator role→model editing and per-run approval: no backend.
- Campaigns page still has its own page header under the workspace banner; Tasks/Campaigns/Products forms are restyled, not redesigned.
- Product-supplied health text (checks) is English; comes from engines.
- Skeleton loading states, light theme, keyboard shortcut layer, window-state persistence: not started.
- `docs/repository_inventory.json` still lists `src/hub.css`/`src/style.css`; new files `src/shared/design.css`, `src/shared/ui.tsx`, `src/shell/shellModel.ts`, `tests/shellModel.test.ts` are unlisted (Master inventory pass; `repository_map.py` needs git, repo has none).
- Known Rust failure (not touched): `rpp_e2e::manifest_health_and_schema_are_reachable_for_both_products` expects RPP "1.0", engine returns "1.1".
