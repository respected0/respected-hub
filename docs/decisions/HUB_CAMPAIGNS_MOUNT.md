# Hub: mounting the Campaigns view (Slice A)

Status: accepted.

## What is mounted

- The existing `src/views/Campaigns.tsx` is now wired into the Orchestrator
  workspace. `src/apps.ts` gains one page `{ id: 'campaigns', label: 'Kampanyalar' }`
  and `src/views/AppWorkspace.tsx` renders `<Campaigns />` for that page. The
  existing `overview`, `runs` and `settings` pages are unchanged.
- No new engine action, no new dependency and no new routing layer. The view
  reuses the already-accepted `listCampaigns`, `campaignStatus` and
  `startCampaign` wrappers and the read-only `campaign_list` command.

## Data-root rule

- The campaign folder is the user's configured Hub data root (`Settings.data_root`
  loaded via the existing `loadSettings` in `src/api.ts`). It is shown read-only
  in the Kampanyalar header; it is never hardcoded and the settings defaults are not
  touched.
- When the list is empty the view says plainly (Turkish) that no campaign was
  found in the selected data folder and that the folder can be changed in Ayarlar.
- Outside Tauri the view keeps its honest notice and shows no demo data. Campaign
  creation stays explicit: a spec is sent only after the button press, no auto-start.

## Manual native smoke steps

1. Run the Hub in Tauri and open Ayarlar; set `data_root` to the folder that
   contains the `campaigns/` root.
2. Open Orchestrator > Kampanyalar.
3. Confirm the header shows the same data root, the list matches the campaign
   folders under `<data_root>/campaigns`, and selecting one loads lanes, timeline
   and supervisor status from the engine.
4. With an empty/absent campaigns folder, confirm the empty hint and that it
   points to Ayarlar.

## Missing engine contracts

None. The mounted view needs only `campaign_list`, `campaign-start` and
`campaign-status`, all already on the allow-list.