# Respected Hub

Tauri 2 + React + TypeScript desktop source for ecosystem management. The initial
development flow discovers sibling product sources, selects a Git project,
submits a bounded Orchestrator task and displays status, logs, patch and cancel.
The native Rust bridge invokes the engine's private JSON interface; no shell
plugin, user-facing CLI, permanent service or setup wizard is added.

## Development

Dependencies are local to this project. Node >=22.12, Rust/Cargo and native Tauri
platform prerequisites are required. On Windows this includes C++ build tools and
WebView2; see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).
No global dependencies were installed during extraction.

```text
npm ci
npm test
npm run build
npm run tauri -- dev
```

The settings section configures Python, Orchestrator/Brain source locations and an
independent Orchestrator DataRoot. Defaults discover siblings of this source
directory. Connections persist in Hub's own application data; engine state is
separate. Source detection does not mean a production product is installed.

Closing Hub leaves tasks running in Orchestrator. Reopen to inspect or cancel.
Retry is explicit; engine validation decides whether a result is successful.
Completed results have a separate **verify result** action that rejects stale or
tampered evidence. There is no automatic apply.

`npm run dev` is a browser preview: it disables engine operations without Tauri,
labels that limitation, and does not fabricate statuses or task results.
Native IPC and folder selection must be tested in the actual desktop build.

Product installation/update/repair/removal, integration lifecycle, visual feature
toggles, signed catalog, RPP, SIL and finite Lifecycle Core are later work. Their
controls are not represented as working features. Bundling is disabled for this
development stage. Do not claim Windows/macOS/Linux release acceptance from a
TypeScript build alone.

The [source atlas](docs/REPOSITORY_MAP.md) and the [dated extraction evidence](../secondbrain/docs/records/2026-10-product-extraction/IMPLEMENTATION.md)
record the actual verification and native toolchain blocker. Current ecosystem
extraction status is centralized in Brain's [PROJECT_STATUS](../secondbrain/docs/PROJECT_STATUS.md).
