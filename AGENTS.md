# Product source rules

This is Respected Hub: Tauri 2, React and TypeScript. Preserve the visual entry
point; do not add a public CLI or replace IPC with mocked task success.
GUI-driven product lifecycle (install/update/repair/uninstall), the Safe Integration
Layer and the SecondBrain setup wizard are in scope (user decision 2026-10-11,
RESPECTED_MASTER_DIRECTIVE.md), implemented and tested only on disposable roots.
Treat sibling source discovery as development-only. Native platform acceptance and
release signing remain independent gates; never claim them without recorded evidence.

Do not install global tools, edit existing AI configs or modify live Brain data.
Do not run source repository Git actions/commits or parallel agents unless asked.
Orchestrator runs must survive Hub closing; engine owns task state and cancellation.
Expose only explicit allowed private engine operations; no generic shell execution.

Keep `docs/repository_inventory.json` role/purpose/relationships accurate in the
same task. Use `python tools/repository_map.py --filesystem-only --update`, review
new records, `--accept-reviewed PATH`, `--write` and `--check`; do not edit generated
Markdown. Run TypeScript/build/tests and native checks when the toolchain exists.
Exclude dependencies/cache/build outputs, never unexplained source files.

Keep decisions under docs/decisions, evidence under docs/records and current
cross-product extraction status in `../secondbrain/docs/PROJECT_STATUS.md` for
this staged extraction. Documentation references never become runtime imports.
