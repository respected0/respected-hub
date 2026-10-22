import { useEffect, useState } from 'react';
import { APPS, type AppDef } from '../apps.ts';
import { engine, loadSettings, native, type Run, type Settings } from '../../shared/api.ts';
import { listCampaigns } from '../../products/orchestrator/campaigns.ts';
import { rppTry } from '../../shared/productClient.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import BrainSetup from '../../products/brain/views/BrainSetup.tsx';
import BrainInstall from '../../products/brain/views/BrainInstall.tsx';
import Tasks from '../../products/orchestrator/views/Tasks.tsx';
import { ProductPanel, ProductStatusPanel } from './Products.tsx';
import Campaigns from '../../products/orchestrator/views/Campaigns.tsx';
import MemoryAccess from '../../products/brain/views/MemoryAccess.tsx';
import MemoryIndex from '../../products/brain/views/MemoryIndex.tsx';
import MemoryWorkspace from '../../products/brain/views/MemoryWorkspace.tsx';
import OrchestratorMcp from '../../products/orchestrator/views/OrchestratorMcp.tsx';
import OverviewPanel from '../../products/orchestrator/views/OverviewPanel.tsx';
import AgentsPanel from '../../products/orchestrator/views/AgentsPanel.tsx';
import EvidencePanel from '../../products/orchestrator/views/EvidencePanel.tsx';
import { dataRootSummary, mountFor } from '../workspaceModel.ts';
import { Emblem, Weave } from '../../shared/ui.tsx';
import '../../products/orchestrator/views/settings.css';

const BRAIN_ABOUT: Record<string, string> = { memory: 'Notlarda ara, oku, hızlı not ekle, sabah özetini gör.', setup: 'AI araç bağlantıları, MCP izinleri ve not kasası.', settings: 'Ürün ayarları, arama dizini ve anlamsal arama.' };

// The Overview is where product health lives (it is shown nowhere else): status, checks and capabilities, then what the product offers.
function Overview({ app, eco, go }: { app: AppDef; eco: Ecosystem; go: (page: string) => void }) {
  const [workers, setWorkers] = useState<Record<string, string[] | null> | null>(null);
  useEffect(() => { if (native && app.id === 'orchestrator') engine<{ workers?: Record<string, string[] | null> }>('status').then(s => setWorkers(s.workers ?? null)).catch(() => setWorkers(null)); }, [app.id]);
  return <>
    {app.rpp && <ProductStatusPanel id={app.rpp} />}
    {app.id === 'orchestrator'
      ? <OverviewPanel runs={eco.runs} workers={workers} onOpenTasks={() => go('runs')} />
      : <section className="panel-glass"><div className="section-head"><h2>Neler yapabilirsiniz?</h2></div>
          <div className="app-cards">{app.pages.filter(page => BRAIN_ABOUT[page.id]).map(page => <button key={page.id} className="app-card" onClick={() => go(page.id)}><span className="app-card-body"><strong>{page.label}</strong><small>{BRAIN_ABOUT[page.id]}</small></span></button>)}</div></section>}
  </>;
}

// Real engine data only; both calls degrade to honest empty states on failure.
function AgentsPage() {
  const [workers, setWorkers] = useState<Record<string, string[] | null> | null>(null);
  const [settings, setSettings] = useState<Record<string, unknown> | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  useEffect(() => {
    if (!native) { setLoading(false); return; }
    let active = true;
    void (async () => {
      try {
        const status = await engine<{ workers?: Record<string, string[] | null> }>('status');
        if (active) setWorkers(status.workers ?? null);
      } catch {
        if (active) { setWorkers(null); setError('Worker durumu al\u0131namad\u0131.'); }
      }
      const readback = await rppTry('orchestrator', 'settings.readback');
      if (active) setSettings(readback?.ok && readback.result && typeof readback.result === 'object' ? readback.result as Record<string, unknown> : null);
      if (active) setLoading(false);
    })();
    return () => { active = false; };
  }, []);
  return <AgentsPanel workers={workers} settings={settings} loading={loading} error={error} />;
}

function EvidencePage({ runs }: { runs: Run[] }) {
  const [campaignIds, setCampaignIds] = useState<string[]>([]);
  useEffect(() => {
    if (!native) return;
    listCampaigns().then(setCampaignIds).catch(() => setCampaignIds([]));
  }, []);
  return <EvidencePanel runs={runs} campaignIds={campaignIds} />;
}

function DataRootSection() {
  const [dataRoot, setDataRoot] = useState('');
  const [campaignCount, setCampaignCount] = useState<number | null>(null);
  const [error, setError] = useState('');
  useEffect(() => {
    if (!native) return;
    loadSettings().then((settings: Settings) => setDataRoot(settings.data_root)).catch(reason => setError(String(reason)));
    listCampaigns().then(ids => setCampaignCount(ids.length)).catch(() => setCampaignCount(null));
  }, []);
  const summary = dataRootSummary(dataRoot, campaignCount, native);
  return <section className="panel-glass data-root-section" aria-label="Veri dizini">
    <h2>Veri dizini</h2>
    <p className="muted" role="status">{summary.text}</p>
    {error && <p className="error" role="alert">{error}</p>}
    <p className="muted" role="note">Rol → model ataması düzenleme ve çalıştırma başına onay Hub'da henüz desteklenmiyor.</p>
  </section>;
}

function SettingsPage({ app }: { app: AppDef }) {
  return <>
    {app.rpp && <ProductPanel id={app.rpp} />}
    {app.id === 'brain' && <MemoryIndex />}
    {app.id === 'orchestrator' && <MemoryAccess />}
    {app.id === 'orchestrator' && <OrchestratorMcp />}
    {app.id === 'orchestrator' && <DataRootSection />}
  </>;
}

export default function AppWorkspace({ appId, page, eco, go }: { appId: string; page: string; eco: Ecosystem; go: (page: string) => void }) {
  const app = APPS.find(item => item.id === appId);
  if (!app) return <div className="page"><p className="error">Uygulama bulunamadı.</p></div>;
  const view = mountFor(app.id, page);
  const runtime = eco.apps[app.id]; const summary = runtime.summary;
  const pageInfo = app.pages.find(item => item.id === page);
  return <div className={`page accent-${app.accent}`}>
    <header className="ws-banner"><Weave /><Emblem app={app.id} />
      <div className="ws-title"><span className="eyebrow">Çalışma alanı</span><h1>{app.name}</h1><p>{app.tagline}</p>
        <span className="ws-meta">{summary.version ? <span className="ver">v{summary.version}</span> : null}{!runtime.loading && summary.statusKind !== 'ok' && view !== 'overview' && <button className="link-btn" onClick={() => go('overview')}>Ürün bir sorun bildiriyor: Genel bakışı aç</button>}</span></div>
      <div className="ws-tabs" role="tablist" aria-label={`${app.name} bölümleri`}>{app.pages.map(item => <button key={item.id} role="tab" aria-selected={view === mountFor(app.id, item.id)} className="tab" onClick={() => go(item.id)}>{item.label}</button>)}</div>
    </header>
    <section className="ws-page" aria-label={pageInfo?.label}>
      <div className="ws-head"><h2>{pageInfo?.label}</h2><p className="muted">{pageInfo?.description}</p></div>
      <div className="ws-stack">
      {view === 'overview' && <Overview app={app} eco={eco} go={go} />}
      {view === 'setup' && (() => { const lc = eco.lifecycle?.products.find(item => item?.product === 'brain'); return lc?.install_state === 'absent' && lc.available
        ? <BrainInstall onDone={eco.refresh} />
        : <BrainSetup onDone={eco.refresh} />; })()}
      {view === 'runs' && <Tasks />}
      {view === 'campaigns' && <Campaigns />}
      {view === 'agents' && <AgentsPage />}
      {view === 'evidence' && <EvidencePage runs={eco.runs} />}
      {view === 'memory' && <MemoryWorkspace />}
      {view === 'settings' && <SettingsPage app={app} />}
      </div>
    </section>
  </div>;
}