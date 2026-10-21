# Continue development from nothing but the three repositories

Verified 2026-10-18 and again 2026-10-21 (fresh clone of the three GitHub repositories compared with the working copies).
What is NOT in git is listed at the bottom.

## 1. Folder layout (the tools find each other by this)
```
<any folder>\respected-ecosystem\
    respected-hub\            github.com/respected0/respected-hub          (also holds release tooling + handoff: release\)
    respected-secondbrain\    github.com/respected0/respected-secondbrain
    respected-orchestrator\   github.com/respected0/respected-orchestrator
```
```
git clone https://github.com/respected0/respected-hub.git
git clone https://github.com/respected0/respected-secondbrain.git
git clone https://github.com/respected0/respected-orchestrator.git
```
Generated next to them, never committed: `dist\` (installers), `.build-cache\`, `.cargo-target-hub\` (Rust build output, ~12 GB).

## 2. Tools to install once (Windows 10/11)
| Tool | Used for | Version seen |
|---|---|---|
| Python (python.org install, NOT the Store build) | build scripts and freezing both products (PyInstaller) | 3.13 |
| Node.js + npm | Hub frontend, tests | 24 |
| Rust (MSVC toolchain) + Visual Studio Build Tools (C++) | Hub (Tauri) | 1.99 |
| Inno Setup 6 | installers (`ISCC.exe`; or set `INNO_COMPILER`) | 6.7 |
| Microsoft Edge WebView2 runtime | running the Hub | ships with Windows 11 |

Python packages (global or venv that runs the build): `pip install cryptography`, then
`pip install -e "respected-secondbrain[dev]" -e "respected-orchestrator[dev]"`. SecondBrain's frozen engine takes its
version from that editable install; its build stops if the installed package is another checkout or an older version.
Hub frontend: `cd respected-hub && npm ci`.

## 3. Everyday checks
* Hub: `cd respected-hub && npm run check`; Rust: `cargo test --manifest-path src-tauri/Cargo.toml --no-fail-fast` (set `CARGO_TARGET_DIR` to `<ecosystem>\.cargo-target-hub`); both at once with the real-environment guard: `python release/tools/accept_hub.py`. The first Rust run freezes both products into `<ecosystem>\.build-cache\dist` (a few minutes, then cached).
* SecondBrain: `cd respected-secondbrain && python -m unittest discover -s tests -t .` (the distribution test needs `python tools/build_installer.py --platform windows --output dist` first).
* Orchestrator: `cd respected-orchestrator && python -m unittest discover -s tests -t .` (the packaged-build tests need `python tools/build_dist.py` first; do not run it in parallel with the SecondBrain suite).
* Release tooling self-check: `python respected-hub/release/tools/test_release_tools.py`.

## 4. Build and publish a release
One version for everything to start with (`0.1.0-beta.1`); bump it in `respected-hub/src-tauri/tauri.conf.json` (+ `Cargo.toml`,
`package.json`, `packaging/windows/hub_setup.iss` default), `respected-secondbrain/pyproject.toml`, `respected-orchestrator/pyproject.toml`
and `respected_orchestrator/__init__.py`; reinstall both editable packages. A release always refreezes both products from the current source. Package versions are SemVer; "beta" is part of the version
(`0.1.0-beta.2`, `0.1.0-rc.1`, `0.1.0`), never a separate label.
```
$k = "$env:USERPROFILE\.respected-release\respected-release.vault"
$s = Read-Host "Vault passphrase" -AsSecureString
$env:RESPECTED_VAULT_PASSPHRASE = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($s))
python respected-hub\release\tools\publish_release.py build --key $k        # Hub installers + signed product archives (dist\release)
python respected-hub\release\tools\publish_release.py catalog --key $k      # merges into the published catalog, signs it
python respected-hub\release\tools\release_package.py build --key $k --dir dist\release   # SHA256SUMS for the installers
python respected-hub\release\tools\publish_release.py upload                # prints the gh commands; add --yes to publish
Remove-Item Env:\RESPECTED_VAULT_PASSPHRASE
```
`build --only secondbrain` (etc.) releases one product; the catalog keeps the others as published. `catalog --channel stable` publishes
to the stable channel; `--min-hub X` makes older Hubs update themselves first. Upload order is products first, catalog last.
Without `--release-key`/`--key` a build is DEVELOPMENT (dev trust key) and must never be published.

## 5. Not in git: back these up yourself
| What | Why it matters | If lost |
|---|---|---|
| `%USERPROFILE%\.respected-release\respected-release.vault` + its passphrase | signs the catalog and every package | installed Hubs can no longer receive updates (they trust only this key); people would have to download a new Hub by hand. Backup: cloud copy of the vault + passphrase on paper/password manager |
| nothing else is irreplaceable | `dist\`, `.build-cache\`, `.cargo-target-hub\` regenerate from the repos | |

Optional history (not needed to develop): `.campaign-data\` (autonomous-run evidence), `respected-secondbrain\.local\` (old archives).
The release key id is `dabcb5265227bbdf` (public line: `respected-hub/src-tauri/release-keys.txt`). Where things stand: `release/docs/MASTER_HANDOFF.md`.
