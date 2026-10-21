# Product source rules

This is Respected Hub: Tauri 2, React and TypeScript. Preserve the visual entry point; do not add a public CLI or replace
IPC with mocked task success. Product lifecycle (install/update/repair/uninstall), the Safe Integration Layer and the
SecondBrain setup wizard are tested only on disposable roots and a fake home. Source discovery of sibling repositories is
development-only. Release signing and native acceptance are separate gates; never claim them without evidence.

Do not install global tools, edit existing AI configs or modify live SecondBrain data. No Git commits, pushes or parallel
agents unless the user asks. Orchestrator runs must survive the Hub closing; the engine owns task state and cancellation.
Expose only explicitly allowed private engine operations; no generic shell execution.

Before calling a change done: `npm run check`, `cargo test` (or `python release/tools/accept_hub.py`), and for UI or
lifecycle changes the native driver `tests/native/hub_native_e2e.mjs` against a built exe. Binding decisions live in
`docs/decisions/` and change in the same task as the code. Documentation references never become runtime imports.
