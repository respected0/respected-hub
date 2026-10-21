# Remote catalog, product lifecycle and Windows registration (decision, 2026-10-21)

Decided with the owner before the first public beta. Model: a game launcher. The Hub is the only installer people
download; it installs, updates, repairs and removes the products, which then run on their own (no Hub process needed).

## What the person sees
* **Download**: one small installer (`RespectedHub-Windows-Setup.exe`, Hub only). `RespectedSecondBrain-…` and
  `RespectedOrchestrator-…` are the same file; they open the Hub on that product (`--install=<product>`).
* **Install / update / repair** happen inside the Hub. A product archive is downloaded only then; installing, verifying,
  registering hooks/MCP and uninstalling need no network. The last installed archive stays in the cache so repair works
  offline; an update deletes the older archive and the older installed version (no leftovers, no rollback of an update).
* **Updates are offered, never silent**: "Hub X hazır. Güncellensin mi?" / "SecondBrain için güncelleme var" with
  *Güncelle* / *Sonra*. The Hub's own update downloads the verified installer and runs it `/SILENT /RELAUNCH=1`.
* **Windows Installed apps** lists Respected Hub, Respected SecondBrain and Respected Orchestrator separately.
  *Uninstall* opens the Hub's confirmation (`--uninstall=<product>`), *Modify* opens the product page (`--manage=…`).
* **Uninstalling the Hub** asks whether to remove the products too (default *No*). *Yes*: products, their data and our
  AI-client entries go; vault and notes stay. *No*: products keep working; a copy of the Hub executable stays in
  `RespectedHub\maintenance` so their Installed-apps entries keep working, and it deletes itself after the last product.

## Frozen contracts (every shipped Hub depends on them; change only with a migration in a newer Hub)
| Contract | Value |
|---|---|
| Catalog URL | `https://github.com/respected0/respected-hub/releases/download/catalog/catalog.json` (+ `.sig`) |
| Catalog signature | Ed25519 release key over `b"respected-catalog-v1\n"` + exact file bytes; `.sig` = `<key_id> <base64>` |
| Catalog schema 1 | `{schema, sequence, published, hub:{stable,beta}, products:{secondbrain,orchestrator,runtime:{stable,beta}}}`; a release = `{version,url,sha256,size,min_hub?}`; a lower `sequence` than the cached one is refused |
| Archive | zip of one signed bundle folder (`bundle.json` + `payload/`) at its root; bundle signature as before (`respected-bundle-v1`) |
| Versions | SemVer order (`0.1.0-beta.1 < 0.1.0-rc.1 < 0.1.0 < 0.1.1-beta.1`), PEP 440 spellings equal (`0.1.0b1`); `lifecycle::version_key` |
| Channels | `beta` (default; sees beta and stable) and `stable`; stored in `managed\hub-preferences.json` |
| Installed-apps keys | `HKCU\…\Uninstall\RespectedSecondBrain`, `…\RespectedOrchestrator`; Hub: Inno AppId `{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}` |
| Hub arguments | `--install=<p>`, `--uninstall=<p>`, `--manage=<p>`, `--uninstall-products`, `--sync-registrations` |
| Folders | `%LOCALAPPDATA%\Respected\{RespectedHub\{app,managed,maintenance}, RespectedBrain\{app,data}, RespectedOrchestrator\data}`; vault where the person chose |
| MCP launcher path | `RespectedHub\managed\orchestrator\mcp\respected-orchestrator-mcp.cmd` (written into AI-client configs) |

## Why these choices
* **Products from GitHub Releases, not inside the installer**: the installer no longer carries ~80 MB nobody may use, and
  products get their own release cadence per repository. Plain download URLs (no API) avoid rate limits; public repos need no token.
* **Windows' own curl.exe and tar.exe** (Windows 10 1803+/11) instead of new Rust dependencies: https only in production,
  every byte checked twice (index SHA-256, then the bundle's own signature and per-file hashes) before anything is installed.
* **A newer Hub does the update work**, so a bug in an old Hub's update code is fixed by the next release; `min_hub` makes
  an old Hub update itself before a product that needs it.
* **Orchestrator stays under `RespectedHub\managed`** for the beta: it runs without the Hub (the MCP launcher calls the
  managed Python directly) and can be moved later by a newer Hub that rewrites the launcher and the MCP entries it owns.

## Code
`src-tauri/src/catalog.rs` (index, channels, download, cache, Hub self-update), `registration.rs` (Installed apps,
maintenance copy), `lifecycle.rs` (`version_key`, no previous version kept), `lib.rs` (wiring, intents, startup sweep),
`packaging/windows/hub_setup.iss` (no catalog inside, kept-products path, relaunch), `release/tools/publish_release.py`
(archives, catalog, signature, `gh` upload). Proof: `tests/remote_catalog_e2e.rs`, unit tests in `catalog.rs`/`lifecycle.rs`.
