# Respected Hub

The desktop app of the Respected ecosystem (Tauri 2 + React + TypeScript). It installs, updates, repairs and removes
**SecondBrain** and **Orchestrator** from a signed catalog on GitHub Releases, runs SecondBrain's setup wizard, manages
their AI-client integrations (Safe Integration Layer) and gives each product a workspace. Products talk to the Hub only
through their versioned protocol (RPP); they keep working when the Hub is closed.

## Layout

| Path | What it is |
| --- | --- |
| `src/shell/` | app frame, navigation, Home, My Apps, Activity, Settings, lifecycle controls |
| `src/products/orchestrator/`, `src/products/secondbrain/` | each product's workspace views and their pure models |
| `src/shared/` | RPP client, native bridge, design system |
| `src-tauri/src/` | Rust: lifecycle and journal, signed catalog and updates, bundle trust, RPP bridge, product modules, uninstall cleanup |
| `src-tauri/tests/` | end-to-end tests that drive the real products on disposable roots |
| `tests/` | TypeScript unit tests (same areas as `src/`); `tests/native/` drives a built Hub through WebView2 |
| `packaging/windows/` | Inno Setup script of the Hub installer |
| `release/` | building, signing and publishing a release; the handoff and install layout |
| `docs/decisions/` | binding design decisions |

## Develop

Node >= 22.12, Rust/Cargo and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (C++ build tools,
WebView2). The Rust end-to-end tests build the sibling products, so the three repositories sit side by side
(see `release/BUILD_FROM_SCRATCH.md`).

```powershell
npm ci
npm run check                                   # tsc + unit tests
cargo test --manifest-path src-tauri/Cargo.toml # unit + end-to-end (builds both products once, cached)
python release/tools/acceptance/accept_hub.py              # both, guarded against any change to your real AI-client config
npm run tauri -- dev
```

A development build can run the sibling product sources on Python (Settings › Ürün durumu); a production build only runs
the verified, signed copies it installed. `npm run dev` is a browser preview without the native bridge: engine
operations are disabled there, never faked.
