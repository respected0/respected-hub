import { useEffect, useState } from 'react';
import { APPS, type AppDef } from '../apps.ts';
import { engine, loadSettings, native, type Run, type Settings } from '../api.ts';
import { listCampaigns } from '../campaigns.ts';
import { rppTry } from '../productClient.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import BrainSetup from './BrainSetup.tsx';
import BrainInstall from './BrainInstall.tsx';
import Tasks from './Tasks.tsx';
import { ProductPanel } from './Products.tsx';
import Campaigns from './Campaigns.tsx';
import MemoryAccess from './MemoryAccess.tsx';
import MemoryIndex from './MemoryIndex.tsx';
import MemoryWorkspace from './MemoryWorkspace.tsx';
import OrchestratorMcp from './OrchestratorMcp.tsx';
import OverviewPanel from './orchestrator/OverviewPanel.tsx';
import AgentsPanel from './orchestrator/AgentsPanel.tsx';
import EvidencePanel from './orchestrator/EvidencePanel.tsx';
import { dataRootSummary, mountFor } from './orchestrator/workspaceModel.ts';
import { Emblem } from '../ui.tsx';
import './orchestrator/settings.css';

function Overview({ app, eco, go }: { app: AppDef; eco: Ecosystem; go: (page: string) => void }) {
  const runtime = eco.apps[app.id]; const summary = runtime.summary;
  const [workers, setWorkers] = useState<Record<string, string[] | null> | null>(null);
  useEffect(() => { if (native && app.id === 'orchestrator') engine<{ workers?: Record<string, string[] | null> }>('status').then(s => setWorkers(s.workers ?? null)).catch(() => setWorkers(null)); }, [app.id]);
  return <>
    <section className="panel-glass"><div className="section-head"><h2>Durum</h2><span className={`badge-status ${summary.statusKind}`}>{summary.statusText}</span></div>
      {summary.errorText && <p className="error" role="alert">{summary.errorText}</p>}
      {summary.checks.length > 0 && <ul className="checks">{summary.checks.map((check, i) => <li key={i} className={check.ok ? 'ok' : 'bad'}><span aria-hidden="true">{check.ok ? '\u2713' : '\u2715'}</span> {check.name}{check.detail ? ` \u2014 ${check.detail}` : ''}</li>)}</ul>}
      {summary.capabilities.length > 0 && <ul className="capabilities" aria-label="Yetenekler">{summary.capabilities.map(c => <li key={c}>{c}</li>)}</ul>}</section>
    {app.id === 'secondbrain' && runtime.configSource === 'defaults' && <section className="panel-glass"><h2>Kurulum bekliyor</h2><p>Yapılandırma dosyası henüz yok.</p><button className="cta" onClick={() => go('setup')}>SecondBrain kurulumunu tamamla</button></section>}
    {app.id === 'orchestrator' && <OverviewPanel runs={eco.runs} workers={workers} onOpenTasks={() => go('runs')} />}
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
    {app.id === 'secondbrain' && <MemoryIndex />}
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
  return <div className={`page accent-${app.accent}`}>
    <header className="ws-banner"><Emblem app={app.id} />
      <div className="ws-title"><span className="eyebrow">Çalışma alanı</span><h1>{app.name}</h1><p>{app.tagline}</p>
        <span className={`badge-status ${runtime.loading ? 'unreachable' : summary.statusKind}`}><span className={`dot ${runtime.loading ? '' : summary.statusKind}`} aria-hidden="true" />{runtime.loading ? 'Kontrol ediliyor…' : summary.statusText}{summary.version ? ` · v${summary.version}` : ''}</span></div>
      <div className="ws-tabs" role="tablist" aria-label={`${app.name} bölümleri`}>{app.pages.map(item => <button key={item.id} role="tab" aria-selected={view === mountFor(app.id, item.id)} className="tab" onClick={() => go(item.id)}>{item.label}</button>)}</div>
    </header>
    {view === 'overview' && <Overview app={app} eco={eco} go={go} />}
    {view === 'setup' && (() => { const lc = eco.lifecycle?.products.find(item => item?.product === 'secondbrain'); return lc?.engine_managed && lc.install_state === 'absent' && lc.available
      ? <BrainInstall onDone={eco.refresh} />
      : <BrainSetup configured={eco.apps[app.id].configSource === 'file'} onDone={eco.refresh} engine={!!lc?.engine_managed && lc.installed} />; })()}
    {view === 'runs' && <Tasks />}
    {view === 'campaigns' && <Campaigns />}
    {view === 'agents' && <AgentsPage />}
    {view === 'evidence' && <EvidencePage runs={eco.runs} />}
    {view === 'memory' && <MemoryWorkspace />}
    {view === 'settings' && <SettingsPage app={app} />}
  </div>;
}