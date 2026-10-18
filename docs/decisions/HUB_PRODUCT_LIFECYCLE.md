# Hub product lifecycle, Safe Integration Layer and Windows distribution

Status: implemented and tested on disposable roots (2026-10-14). Windows only; unsigned local builds.

## Decisions

1. **Catalog bundles.** Everything the Hub installs comes from `<catalog>/<name>/bundle.json` + `payload/`. `bundle.json`
   lists every file with size and sha256 and a digest over that list (`tools/build_bundle.py`, `tools/build_runtime.py`).
   Kinds: `source` (Orchestrator), `runtime` (Python), `distribution` (frozen SecondBrain). All are **unsigned**
   (`"signature": null`) and shown as "imzasız yerel paket"; no production trust claim is made.
2. **Hub-managed copies (Orchestrator, runtime).** Verified staging copy -> per-file hashes -> the product's own RPP
   `manifest`/`health` on a scratch data root -> install record written into the product's own manifest -> atomic
   `current.json` commit. Journal in `<managed>/journal`, idempotent operation ids, rollback of the newest operation,
   crash recovery at start-up, uninstall keeps a restorable backup until the user purges it.
3. **Python runtime.** The Hub ships its own pruned CPython + the dependency closure of `agent-framework-core` (41 MB)
   and runs every Python product with it. The first product install provisions it automatically; no system Python is
   required (the native E2E starts the Hub with Python removed from `PATH`).
4. **SecondBrain through its own engine.** The Hub does not copy SecondBrain files. It verifies and stages the frozen
   distribution, shows a read-only plan, then runs the product's `setup` / `update` / `repair` / `uninstall` commands
   (`--allow-unattested` is passed explicitly for local, unattested packages; release attestation stays fail-closed
   for anything else). The product owns application files, vault, ownership records and its transaction journal.
5. **Safe Integration Layer.** SecondBrain's `integrations.plan` (RPP 1.1, read-only) implements
   Discover -> Inspect -> Plan -> Explain: detected AI-client configs, the exact files that would be created/modified,
   and a typed `LIFECYCLE_CONFLICT` for unknown or user-modified registrations (never overwritten). Approve -> Apply ->
   Verify -> Record ownership -> Rollback are the installer's (`setup` with `--global/--mcp/--schedule/--shortcut`,
   `uninstall`). Foreign entries are preserved byte for byte; switching an integration off restores the file exactly.
   Unsupported or unknown tools are listed as untouched, never guessed at.
6. **Consistency.** Hub status for a product is computed from the same facts as the product's own manifest:
   Orchestrator's install record is written by the product at install time (file digests included), SecondBrain's state is
   its own install manifest. Tampering is reported as `repair-needed` by both.
7. **Bridge hardening.** The webview can reach only read-only lifecycle ops through `rpp_request`
   (`forbidden_lifecycle_call`); everything that writes goes through journaled `lifecycle_*` commands. Hub code uses
   `rpp_call_trusted` for the product install record.
8. **Distribution.** `tools/build_hub_installer.py` stages the release exe + catalog and compiles
   `packaging/windows/hub_setup.iss` with Inno Setup (per-user, no admin). Nothing is downloaded.

## Known limits (honest list)

- No code signing, no release attestation, no remote catalog; Windows only (macOS/Linux untested).
- SecondBrain product operations take ~1 minute each (product-side hashing/fsync of ~2000 files).
- SecondBrain update/uninstall while the app runs can be deferred by the product (`pending`); the Hub reports that and asks to close it.
- The Orchestrator's role->model editing is not supported.
- WebView2 must exist on the machine (preinstalled on Windows 11).
