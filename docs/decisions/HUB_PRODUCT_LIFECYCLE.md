# Hub product lifecycle and Safe Integration Layer

Status: implemented; Windows only. Bundle signing: `BUNDLE_TRUST.md`; download and updates: `REMOTE_CATALOG_AND_LIFECYCLE.md`.

## Decisions

1. **One kind of package.** Every product ships as its packaged (frozen) distribution: `bundle.json` + `payload/` (the
   whole program folder). `bundle.json` lists every file with size and sha256 and a digest over that list and carries an
   Ed25519 signature (`release/tools/build_bundle.py <product>` freezes the product with its own build tool and wraps
   it). No product needs Python on the machine.
2. **Orchestrator: a Hub-managed copy.** Verified staging copy -> per-file hashes -> the product's own RPP
   `manifest`/`health` from `respected-orchestrator.exe` on a scratch data root -> the product records the install from
   its `distribution.json` (`lifecycle.apply` with `target.distribution`) -> atomic `current.json` commit. Journal in
   `<managed>/journal`, idempotent operation ids, rollback of the newest operation, crash recovery at start-up, only the
   active version kept. The stable MCP launcher is rewritten whenever the active copy changes.
3. **SecondBrain through its own engine.** The Hub verifies and stages the frozen distribution, shows a read-only plan,
   then runs the product's `setup` / `update` / `repair` / `uninstall` (`--allow-unattested`: the Hub's Ed25519 check is
   the authenticity gate for the beta). The product owns application files, vault, ownership records and its journal.
4. **Safe Integration Layer.** SecondBrain's `integrations.plan` (RPP 1.1, read-only): Discover -> Inspect -> Plan ->
   Explain, with a typed `LIFECYCLE_CONFLICT` for unknown or user-modified registrations (never overwritten). Apply,
   verify, record ownership and rollback are the installer's. Foreign entries are preserved; switching an integration off
   undoes only Respected's own entries.
5. **Consistency.** Hub status and the product's own manifest come from the same facts; tampering shows as
   `repair-needed` in both.
6. **Bridge hardening.** The webview reaches only read-only lifecycle ops through `rpp_request`
   (`forbidden_lifecycle_call`); everything that writes goes through journaled `lifecycle_*` commands.
7. **Distribution of the Hub.** `release/tools/build_hub_installer.py` compiles `packaging/windows/hub_setup.iss` with
   Inno Setup (per-user, no admin). The installer carries no products; they come from the signed catalog.

## Known limits

- No Authenticode signature (SmartScreen warns); Windows only.
- SecondBrain operations take about a minute each (product-side hashing/fsync of ~2000 files).
- An update while a product is running can be deferred by the product (`pending`); the Hub reports it.
- WebView2 must exist on the machine (preinstalled on Windows 11).
