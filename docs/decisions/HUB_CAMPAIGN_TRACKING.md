# Hub campaign tracking

Status: accepted (Workstream A2).

## Context

The Orchestrator already exposes durable multi-repository campaigns over its
private JSON entry point: `campaign-start` (a JSON spec on stdin, returns
`{campaign_id,status,pid}`) and `campaign-status ID` (returns
`{campaign_id,status,result,events,supervisor_alive}`). Campaigns live under
`<data_root>/campaigns/<campaign_id>` and identities are `camp-` plus 1..60
characters of `[a-z0-9-]`. The Hub needs to observe and (explicitly) start these
campaigns without exposing any generic shell.

## Decision

- Extend the `engine_request` allow-list with exactly two actions:
  `campaign-start` (no run identity, JSON body required, capped at 64 KiB) and
  `campaign-status` (its run identity must match `^camp-[a-z0-9-]{1,60}$` and is
  validated before any process is spawned). Existing actions are unchanged.
- Add a separate read-only `campaign_list` command returning the newest 30
  campaign directory NAMES under `<data_root>/campaigns`. It accepts only valid
  identities, ignores symlinks/junctions, and returns an empty list when the
  folder is missing. It never follows a path out of the campaigns root.
- `src/products/orchestrator/campaigns.ts` holds the typed wrappers (`listCampaigns`, `campaignStatus`,
  `startCampaign`) and the pure `summarizeCampaign(status)` mapper.
  `summarizeCampaign` is tolerant of missing fields, so older campaigns without
  chain/continuation info still render.
- `src/products/orchestrator/views/Campaigns.tsx` renders the list, a detail panel (lanes, timeline,
  chain when present, supervisor-alive indicator) and a collapsed advanced
  section that sends a spec only after an explicit confirm. Polling runs every
  5 s only while a campaign is running and the view is mounted.

## Honesty rules

- Outside Tauri the view shows "Tauri masaüstü uygulaması gerekli" and shows no
  fabricated campaign data or metrics.
- Every value comes from the engine reply; nothing is invented. Unsupported or
  unknown fields degrade to plain Turkish placeholders instead of demo data.
- Starting a campaign is never automatic and is clearly labelled as running real
  workers.

## Consequences

`Campaigns.tsx` is intentionally NOT mounted in `App.tsx`; a later lane wires it
into navigation. `src/hub.css` carries the view's styles; no new dependency,
routing library or UI framework is introduced. The Rust allow-list remains the
single trust boundary for engine actions.