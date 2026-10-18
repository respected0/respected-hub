# Hub Orchestrator workspace mount (slice D4)

## What is mounted

The Orchestrator app tabs in `src/apps.ts` are now, in order:

1. `overview` - "Genel bakis" - the generic Durum/status section plus the
   standalone `OverviewPanel` (run counts and latest runs from `eco.runs`,
   worker CLIs from the engine `status` action, `Tum gorevler` navigates to
   `runs` via `go`).
2. `agents` - "Ajanlar ve modeller" - `AgentsPanel`, fed with real engine
   worker data from the engine `status` action and the Orchestrator RPP
   settings from `rppTry('orchestrator', 'settings.readback')`. When the
   readback is unavailable the panel receives `null` and shows its honest
   empty state; nothing is invented.
3. `evidence` - "Kanitlar" - `EvidencePanel`, with `eco.runs` and campaign ids
   from `listCampaigns()`.
4. `campaigns` - "Kampanyalar" - unchanged `Campaigns` view.
5. `runs` - "Gorevler" - unchanged `Tasks` view.
6. `settings` - "Ayarlar" - the existing `ProductPanel` plus a new
   "Veri dizini" section.

The SecondBrain tabs (`overview`, `setup`, `settings`) are unchanged.

All page/mount decision logic lives in the pure module
`src/views/orchestrator/workspaceModel.ts` (`ORCHESTRATOR_PAGES`, `mountFor`,
`dataRootSummary`) so it is unit-testable with `node --test`. `src/apps.ts`
imports `ORCHESTRATOR_PAGES` so the registry and the mount logic cannot drift.

## Data root behaviour

The settings page shows the real `data_root` from `loadSettings()` and the
number of campaigns found by `listCampaigns()` under it. When zero campaigns
are found the copy says so plainly and points at "Baglantilar ve ayarlar"
(Connections), which already saves `data_root`. Browser (non-Tauri) preview
shows an honest notice instead of any engine claim.

Native default: `get_settings()` in `src-tauri/src/lib.rs` picks the default
`data_root` through the pure helper `default_data_root`. An absolute path in
the environment variable `RESPECTED_ORCHESTRATOR_DATA_ROOT` wins; otherwise the
previous app-local-data sibling path is used. A saved `connections.json` still
wins over both because `get_settings` returns it first. No developer directory
is hardcoded.

## Not supported yet (labelled, never faked)

- Editing role -> model assignment from Hub. Assignment is defined per campaign
  in the campaign spec file; the panel states this.
- Per-run approval from Hub. EvidencePanel states that Hub does not approve or
  apply patches; the required command is run in a terminal by the user.