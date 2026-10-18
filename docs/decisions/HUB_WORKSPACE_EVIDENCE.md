# Hub workspace review, evidence and approvals

Status: accepted (Workstream D3, read-only slice).

## Context

The Orchestrator exposes per-run and per-campaign review state over its private
engine entry point. For a single run, `inspect <run_id>` returns the request
(task, worker, project_root), status, phase, reason, error, log excerpts, a
patch preview and the changed files. For a campaign, `campaign-status <id>`
returns `result.lanes[]` with each lane's `reviews[]`, `changed_files`,
`token_usage` and optional chain info, plus `status`/`events`/`supervisor_alive`.

Two engine actions stay outside the Hub on purpose:

- `apply` is not in the `engine_request` allow-list
  (`status, create, start, list, inspect, cancel, check, retry, campaign-start,
  campaign-status`). Applying a patch is a mutating action and the Hub does not
  perform it.
- `maf-approve` is not in the allow-list either, and it requires an interactive
  terminal by design.

The Hub must not pretend to approve or to apply anything.

## Decision

- `src/views/orchestrator/evidenceModel.ts` is a pure, testable mapper (no
  React, no Tauri imports). `evidenceFor(inspect, campaign, fallbackRunId)`
  turns an `inspect` payload and/or a `campaign-status` reply into
  `{headline, verdict, evidence[], reviews[], hasPatch, nextStepText}`. It is
  tolerant of missing or malformed fields; unknown states degrade to plain
  Turkish placeholders, never demo data.
  - `verdict` is one of `approved | needs_changes | failed | human_approval |
    unknown`, derived from run/campaign/lane statuses, review decisions and a
    present error, in that priority order (failed > human_approval >
    needs_changes > approved > unknown).
  - `reviews[]` entries are `{stage, profile, decision, reason}` collected from
    `result.lanes[].reviews[]` (and any top-level `reviews[]`).
  - `nextStepText` is shown as copyable text. For `MASTER_APPROVED` it gives the
    exact manual apply command; for `HUMAN_APPROVAL_REQUIRED` the interactive
    approve command. Both state that the Hub does not run the command.
- `src/views/orchestrator/EvidencePanel.tsx` (default export, props
  `{runs: Run[]; campaignIds: string[]}`) lets the user pick a run (calls engine
  `inspect`) or a campaign (calls `campaign-status`) and renders the evidence.
  Outside Tauri it shows an honest notice and never fabricates data.
- `src/views/orchestrator/evidence.css` carries the panel's styles; no change to
  `src/hub.css` and no new dependency.
- `tests/orchestratorEvidence.test.ts` covers the pure model with `node --test`.

The panel is intentionally NOT mounted in `App.tsx` (owned by other lanes); a
later lane wires it into navigation.

## Manual commands

The exact terminal command is the Orchestrator's own human CLI. The Hub shows it
verbatim and never runs it:

- apply a `MASTER_APPROVED` run: `python -m respected_orchestrator apply <run_id>`
- approve an interactive run: `python -m respected_orchestrator maf-approve <run_id>`

## Missing engine contracts

These capabilities are absent from the current engine surface. The Hub does not
fake them; each is recorded here for a follow-up lane.

- Apply/approve exposure with a safe confirmation: the Hub cannot apply a patch
  or approve a run. A future contract would need an explicit, confirmed action
  that keeps the interactive-terminal requirement of `maf-approve` intact.
- Review-decision list endpoint: there is no dedicated read for review decisions
  across runs; the panel can only read decisions that `campaign-status` embeds
  under `result.lanes[].reviews[]`. A run-level `inspect` reply has no documented
  `reviews[]` field.
- Token aggregates: `token_usage` is only available per lane inside
  `campaign-status`; there is no campaign- or run-level token aggregate, so the
  panel sums whatever numeric lanes report and shows nothing when none do.

## Consequences

`evidenceModel.ts` stays pure and unit-tested; `EvidencePanel.tsx` is checked by
`tsc`. The Rust `engine_request` allow-list remains the single trust boundary for
engine actions, and no mutating engine action is reachable from the Hub.