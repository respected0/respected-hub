// Pure, testable decision logic for the Orchestrator workspace mount (slice D4).
// No React and no Tauri imports: node --test can import this module directly.
// Real data only; the browser preview is labelled honestly, never faked.

interface WorkspacePage { id: string; label: string; description: string; }

// Orchestrator workspace tabs, in tab order. This is the single source of truth
// used by apps.ts so the registry and the mount logic cannot drift apart.
export const ORCHESTRATOR_PAGES: WorkspacePage[] = [
  { id: 'overview', label: 'Genel bak\u0131\u015f', description: '\u00dcr\u00fcn\u00fcn sa\u011fl\u0131\u011f\u0131, g\u00f6rev \u00f6zeti ve son \u00e7al\u0131\u015ft\u0131rmalar.' },
  { id: 'agents', label: 'Ajanlar ve modeller', description: 'Rollere atanan modeller, sa\u011flay\u0131c\u0131 profilleri ve kurulu \u00e7al\u0131\u015fan ara\u00e7lar.' },
  { id: 'evidence', label: 'Kan\u0131tlar', description: 'Motorun verdi\u011fi kan\u0131tlar ve inceleme kararlar\u0131. Hub onay vermez ve yama uygulamaz; gereken komut bir terminalde sizin taraf\u0131n\u0131zdan \u00e7al\u0131\u015ft\u0131r\u0131l\u0131r.' },
  { id: 'campaigns', label: 'Kampanyalar', description: 'Orchestrator\u0027\u0131n kendi ba\u015f\u0131na s\u00fcrd\u00fc\u011f\u00fc uzun projeler: hangi \u015ferit nerede, son olaylar, s\u00fcperviz\u00f6r \u00e7al\u0131\u015f\u0131yor mu.' },
  { id: 'runs', label: 'G\u00f6revler', description: 'Yeni g\u00f6rev ba\u015flat\u0131n; \u00e7al\u0131\u015fanlar\u0131n sonu\u00e7lar\u0131n\u0131 ve yamalar\u0131n\u0131 izleyin.' },
  { id: 'settings', label: 'Ayarlar', description: 'Orchestrator ayarlar\u0131, MCP eri\u015fimi ve Brain haf\u0131zas\u0131 ba\u011flant\u0131s\u0131.' },
];

type WorkspaceMount =
  | 'overview' | 'agents' | 'evidence' | 'campaigns' | 'runs' | 'settings' | 'memory' | 'none';

// Which component an app/page pair mounts. Every orchestrator page id resolves to
// a real mount; anything unrecognised resolves to 'none' (nothing is rendered).
export function mountFor(appId: string, page: string): WorkspaceMount {
  if (appId === 'orchestrator') {
    return (ORCHESTRATOR_PAGES.some(candidate => candidate.id === page) ? page : 'none') as WorkspaceMount;
  }
  if (appId === 'brain') {
    if (page === 'overview' || page === 'settings' || page === 'memory') return page;
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