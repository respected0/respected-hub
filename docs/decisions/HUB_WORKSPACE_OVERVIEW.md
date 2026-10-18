# Hub workspace overview

Status: accepted (Workstream D1, read-only slice).

## Context

The Orchestrator workspace needs an overview page that shows honest, real engine
state without a browser-side polling loop of its own. Two engine reads feed it:
`status` (`workers`: name to resolved CLI path, or null when missing) and `list`
(the run list with `run_id`, `status`, `phase`, optional `request`).

## Decision

- `src/views/orchestrator/overviewModel.ts` is a pure, testable mapper (no React,
  no Tauri imports). `deriveOverview(status, runs)` returns `{runCounts,
  latestRuns, workersAvailable, workersMissing, warnings}`. It tolerates missing
  or malformed fields: unknown statuses degrade to plain Turkish groups, malformed
  run records are dropped, and absent engine data becomes an honest warning
  instead of demo data.
  - `runCounts` groups runs as `active` (non-terminal), `succeeded`
    (`completed`), `failed` (`failed`/`timed_out`/`interrupted`/`invalid`) and
    `other` (e.g. `cancelled`), using the shared `terminal()` helper from
    `src/tasks.ts`.
  - `latestRuns` is the newest five. The engine `list` reply has no timestamps,
    so "newest" is approximated by `run_id`, which is timestamp-prefixed
    (`run-20261011122324-...`). The sort is direction-independent: it is correct
    whether the engine returns runs oldest- or newest-first.
  - `workersAvailable` / `workersMissing` split `status.workers` by whether a
    resolved path is present.
- `src/views/orchestrator/OverviewPanel.tsx` (default export, props `{runs;
  workers; onOpenTasks?; onOpenRun?}`) renders the counts, the latest runs and
  the worker availability. It owns no timer; the shell refreshes and passes data.
  Outside Tauri it shows an honest notice and never fabricates data.
- `src/views/orchestrator/overview.css` carries the panel's styles; no change to
  `src/hub.css` and no new dependency.
- `tests/orchestratorOverview.test.ts` covers the pure model with `node --test`.

The panel is intentionally NOT mounted in `AppWorkspace.tsx` or `App.tsx`
(owned by other lanes); a later lane wires it into navigation.

## Missing engine contracts

These capabilities are absent from the current engine surface. The Hub does not
fake them; each is recorded here for a follow-up lane.

- Aggregate usage/token endpoint: the engine `status`/`list` surface reports run
  status only. There is no aggregate token or usage read, so the overview shows
  counts, not usage.
- Live metrics: there is no endpoint for CPU/memory/queue depth or throughput.
  The panel shows only run-state counts and never invents numbers.
- Per-run timestamps: `list` returns no `created_at`/`started_at`, so the newest
  runs are ordered by `run_id` prefix. A real timestamp field would make the
  ordering exact.
- Worker detail: `status.workers` gives only name plus resolved path. There is no
  worker version, capability or health field to show.

## Consequences

`overviewModel.ts` stays pure and unit-tested; `OverviewPanel.tsx` is checked by
`tsc`. The Rust `engine_request` allow-list remains the single trust boundary, and
no new engine action is introduced.