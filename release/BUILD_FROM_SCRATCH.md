# Continue development from nothing but the three repositories

Verified 2026-10-18: a fresh clone of the three GitHub repositories (same commits as the release) passes `tsc` and the 103 Hub tests,
the release self-check, and builds the Orchestrator bundle. What is NOT in git is listed at the bottom.

## 1. Folder layout (the tools find each other by this)
```
<any folder>\respected-ecosystem\
    respected-hub\            github.com/respected0/respected-hub          (also holds release tooling + handoff: release\)
    respected-secondbrain\    github.com/respected0/respectedbrain
    respected-orchestrator\   github.com/respected0/respected-orchestrator
```
```
git clone https://github.com/respected0/respected-hub.git
git clone https://github.com/respected0/respectedbrain.git respected-secondbrain
git clone https://github.com/respected0/respected-orchestrator.git
```
Generated next to them, never committed: `dist\` (installers), `.build-cache\`, `.cargo-target-hub\` (Rust build output, ~12 GB).

## 2. Tools to install once (Windows 10/11)
| Tool | Used for | Version seen |
|---|---|---|
| Python (python.org install, NOT the Store build) | build scripts, products, the Hub's managed runtime is copied from it | 3.13 |
| Node.js + npm | Hub frontend, tests | 24 |
| Rust (MSVC toolchain) + Visual Studio Build Tools (C++) | Hub (Tauri) | 1.99 |
| Inno Setup 6 | installers (`ISCC.exe`; or set `INNO_COMPILER`) | 6.7 |
| Microsoft Edge WebView2 runtime | running the Hub | ships with Windows 11 |

Python packages (global or venv that runs the build): `pip install cryptography pyinstaller setuptools wheel "agent-framework-core==1.21.0"`,
then **`pip install -e respected-secondbrain --no-deps`** and `pip install -e respected-orchestrator --no-deps`.
The SecondBrain editable install matters: the frozen engine takes its version and metadata from it (a stale install once stamped 0.0.1).
Hub frontend: `cd respected-hub && npm ci`.

## 3. Everyday checks
* Hub: `cd respected-hub && npx tsc --noEmit && node --test tests/*.test.ts`; Rust: `cd src-tauri && cargo test --lib` (full: `python release/tools/accept_hub.py`).
* SecondBrain: `cd respected-secondbrain && set PYTHONPATH=%CD%\src && python -m unittest discover -s tests` (use an ABSOLUTE PYTHONPATH).
* Release tooling self-check: `python respected-hub/release/tools/test_release_tools.py`.

## 4. Build a release (what produced v0.1.0-beta.1)
```
$k = "$env:USERPROFILE\.respected-release\respected-release.vault"
$s = Read-Host "Vault passphrase" -AsSecureString
$env:RESPECTED_VAULT_PASSPHRASE = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($s))
python respected-hub\release\tools\build_hub_installer.py --release-key $k
python respected-hub\release\tools\release_package.py build --key $k
python respected-hub\release\tools\release_package.py verify
Remove-Item Env:\RESPECTED_VAULT_PASSPHRASE
```
Bump versions in `respected-hub/src-tauri/tauri.conf.json` (+ Cargo.toml/package.json), `respected-secondbrain/pyproject.toml`, `respected-orchestrator/pyproject.toml`; reinstall SecondBrain editable; delete `.build-cache\brain-dist\RespectedBrain` to force a fresh engine build.
Without `--release-key` the build is a DEVELOPMENT installer (dev trust key, banner says so) and must never be published.

## 5. Not in git: back these up yourself
| What | Why it matters | If lost |
|---|---|---|
| `%USERPROFILE%\.respected-release\respected-release.vault` + its passphrase | the only way to sign releases | cannot sign updates the shipped Hubs trust. Backup: cloud copy of the vault + passphrase on paper/password manager |
| nothing else is irreplaceable | `dist\`, `.build-cache\`, `.cargo-target-hub\` regenerate from the repos | |

Optional history (not needed to develop): `.campaign-data\` (autonomous-run evidence), `respected-secondbrain\.local\` (old archives).
The release key id is `dabcb5265227bbdf` (public line: `respected-hub/src-tauri/release-keys.txt`). The handoff with the full project history is `release/docs/MASTER_HANDOFF.md`.
