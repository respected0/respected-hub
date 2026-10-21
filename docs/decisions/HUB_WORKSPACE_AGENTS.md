# Hub workspace: agents (roles) vs models (profiles)

Status: accepted (Slice D2).

## Context

The Orchestrator workspace in the Hub previously showed only worker CLI
availability and one paragraph noting that worker CLIs and model/provider
profiles are different concepts. Users had no separate read-only view of the
engine-defined agent roles and of the configured model/provider profiles, and no
statement of where the per-campaign role -> model assignment actually lives.

Three different things were easy to conflate:

- **Agent roles** — the engine's own review roles (implementer, independent
  reviewer, final master).
- **Worker CLIs** — the executables the engine can start (`status.workers`,
  name -> resolved path or null).
- **Model / provider profiles** — engine settings that name a default worker or
  a default profile for a role.

## Decision

- Add `src/products/orchestrator/views/agentsModel.ts`: a pure, testable model. It takes
  the raw `status.workers` record and the raw RPP `settings.readback` settings
  and returns two clearly separate lists plus a plain note.
  - `roles` is the fixed set of engine roles above; it is never derived from
    settings and never invented.
  - `profiles` is read only from settings keys that really exist in the
    readback (`default_worker`, `default_worker_profile`,
    `default_reviewer_profile`). Keys that are absent or empty are skipped.
- Add `src/products/orchestrator/views/AgentsPanel.tsx` (default export) rendering the
  roles list, the model/provider profiles list and the worker CLI list as
  separate, read-only blocks, plus the note. No add/edit/delete controls.
- Add `src/products/orchestrator/views/agents.css` for the small amount of layout the
  global styles do not cover. `src/hub.css`, `src/style.css` and
  `src/shell/views/AppWorkspace.tsx` are left untouched; the panel is standalone and
  can be mounted inside the Orchestrator workspace later.

## Honesty rules

- Outside Tauri the panel shows "Tauri masaüstü uygulaması gerekli" and no
  fabricated data.
- Every value comes from the engine reply or the RPP readback; nothing is
  invented. When the engine exposes no profile settings, the panel says so
  plainly instead of showing demo providers.
- The note states that the per-campaign role -> model assignment lives in the
  campaign spec and is NOT editable in the Hub.
- No live metrics are shown.

## Missing engine contracts

The Hub cannot do the following today because no engine/RPP capability exists.
They are recorded here rather than faked:

- **No agent registry API.** There is no operation to list, create, edit or
  delete agent definitions. Roles are therefore shown as the engine's fixed
  set, not as user-managed records.
- **No provider health/quota API.** There is no operation that reports provider
  or model health, availability or remaining quota, so the panel shows no live
  metrics.
- **No role-assignment write API.** There is no operation to bind a role to a
  model/provider for a campaign. That binding lives in the campaign spec
  (used by `campaign-start`) and cannot be read or written through the Hub.
  The engine allow-list stays exactly
  `status, create, start, list, inspect, cancel, check, retry, campaign-start,
  campaign-status` plus the RPP entry point.