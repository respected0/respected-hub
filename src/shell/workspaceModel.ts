// Pure, testable decision logic for the Orchestrator workspace mount (slice D4).
// No React and no Tauri imports: node --test can import this module directly.
// Real data only; the browser preview is labelled honestly, never faked.

interface WorkspacePage { id: string; label: string; }

// Orchestrator workspace tabs, in tab order. This is the single source of truth
// used by apps.ts so the registry and the mount logic cannot drift apart.
export const ORCHESTRATOR_PAGES: WorkspacePage[] = [
  { id: 'overview', label: 'Genel bak\u0131\u015f' },
  { id: 'agents', label: 'Ajanlar ve modeller' },
  { id: 'evidence', label: 'Kan\u0131tlar' },
  { id: 'campaigns', label: 'Kampanyalar' },
  { id: 'runs', label: 'G\u00f6revler' },
  { id: 'settings', label: 'Ayarlar' },
];

type WorkspaceMount =
  | 'overview' | 'agents' | 'evidence' | 'campaigns' | 'runs' | 'settings' | 'setup' | 'memory' | 'none';

// Which component an app/page pair mounts. Every orchestrator page id resolves to
// a real mount; anything unrecognised resolves to 'none' (nothing is rendered).
export function mountFor(appId: string, page: string): WorkspaceMount {
  if (appId === 'orchestrator') {
    return (ORCHESTRATOR_PAGES.some(candidate => candidate.id === page) ? page : 'none') as WorkspaceMount;
  }
  if (appId === 'secondbrain') {
    if (page === 'overview' || page === 'settings' || page === 'memory') return page;
    if (page === 'setup') return 'setup';
  }
  return 'none';
}

interface DataRootSummary {
  dataRoot: string;
  campaignCount: number | null;
  text: string;
}

// Honest summary of the configured orchestrator data root. Reads only what the
// shell really has: data_root from get_settings and the campaign count from
// listCampaigns(). Zero, unknown and browser-only cases are stated plainly.
export function dataRootSummary(dataRoot: unknown, campaignCount: unknown, isNative: boolean): DataRootSummary {
  const root = typeof dataRoot === 'string' ? dataRoot.trim() : '';
  const count = typeof campaignCount === 'number' && Number.isFinite(campaignCount) && campaignCount >= 0
    ? Math.floor(campaignCount)
    : null;

  if (!isNative) {
    return { dataRoot: root, campaignCount: count,
      text: 'Veri dizini ve kampanya say\u0131s\u0131 yaln\u0131zca Tauri masa\u00fcst\u00fc uygulamas\u0131nda okunur.' };
  }
  if (!root) {
    return { dataRoot: '', campaignCount: count,
      text: 'Veri dizini hen\u00fcz yap\u0131land\u0131r\u0131lmad\u0131.' };
  }
  if (count === null) {
    return { dataRoot: root, campaignCount: null,
      text: `Veri dizini: ${root}. Kampanya say\u0131s\u0131 okunamad\u0131.` };
  }
  if (count === 0) {
    return { dataRoot: root, campaignCount: 0,
      text: `Veri dizini: ${root}. Bu dizin alt\u0131nda kampanya bulunamad\u0131; dizini "Ba\u011flant\u0131lar ve ayarlar" sayfas\u0131ndan de\u011fi\u015ftirebilirsiniz.` };
  }
  return { dataRoot: root, campaignCount: count,
    text: `Veri dizini: ${root}. Bu dizin alt\u0131nda ${count} kampanya bulundu.` };
}