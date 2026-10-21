# Development overrides stay out of production; uninstalling the Hub (audit A6 and category B)

Status: implemented 2026-10-17.

## Development overrides (A6)
Everything that exists only so tests can run on disposable roots is now compiled out of a production build: `trust::dev_enabled()` is true in debug
builds or with `--features dev-trust`. Behind it: `RESPECTED_HUB_TEST_HOME`, `RESPECTED_HUB_MANAGED_ROOT`, `RESPECTED_HUB_CATALOG_ROOT`,
`RESPECTED_HUB_BRAIN_APP_DIR`, `RESPECTED_HUB_BRAIN_DATA_DIR`, `RESPECTED_ORCHESTRATOR_DATA_ROOT`, the passthrough of `RESPECTED_APP_DIR`/`RESPECTED_DATA_DIR`
to the Orchestrator's memory provider, the fallback to the source checkout (`CARGO_MANIFEST_DIR`) for product roots and the catalog, the
`RespectedOrchestratorDevelopment` data folder, the dev signing key, and `RESPECTED_HUB_ALLOW_UNSIGNED`. A production build ignores the variables
(they are never read), uses `RespectedOrchestrator` as data folder, the managed copies only, and the shipped catalog only.
* `save_settings` (python / product source roots / SecondBrain launcher paths) is refused in production and the "source connections" panel is hidden:
  a compromised webview can no longer choose which program the Hub starts.
* The webview bridge refuses `settings.apply|validate` for any `memory_*` key (memory provider command, token, grant, enable, write) - only the Hub's
  native commands set those.
* Proof: `cargo test --release --lib trust` (no dev-trust) asserts every override above is ignored and the test home never reaches a product process;
  debug tests assert the overrides work for the isolated end-to-end tests; bridge test for `memory_*` settings keys.

## Uninstalling the Hub (category B)
Before: uninstalling the Hub left SecondBrain's hooks, MCP entries, scheduled task and shortcut and the Orchestrator MCP entries behind with no UI to
remove them. Now the uninstaller (`packaging/windows/hub_setup.iss`) asks "remove the products too?" (default **No**; silent uninstalls pass
`/REMOVEPRODUCTS=1`). Yes runs `respected-hub.exe --uninstall-products` (no window; `cleanup.rs`): Orchestrator MCP registrations are undone,
SecondBrain goes through its own uninstaller (vault and data kept, foreign config entries restored), then Orchestrator, then the Hub's
managed folder is deleted; a log is written beside it. Notes, vaults and Orchestrator run data are never touched.
Proof: `tests/cleanup_e2e.rs` runs the real executable on a fake HOME with SecondBrain and the Orchestrator installed and registered in Claude and
Codex configs, after the clients rewrote their files: our entries are gone, the clients' and user's changes survive, the vault note is intact, the
managed folder is gone; plus the no-op run on an empty machine.

## A second launch
A second Hub launch (for example a product installer's last step) no longer opens a second window: the running one is focused and, for
`--install=<product>`, receives a `startup-intent` event that opens the right page.

## Engine gate
A queued SecondBrain install/update/repair now gives up after 120 s with a "another long operation is running" error when a long engine call (for
example the semantic model download) is still in progress, instead of freezing every status poll behind it.
