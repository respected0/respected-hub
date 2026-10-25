import { useEffect, useState } from 'react';
import { APPS, type AppDef } from '../apps.ts';
import { loadSettings, native, type Run, type Settings } from '../../shared/api.ts';
import { listCampaigns } from '../../products/orchestrator/campaigns.ts';
import { orchestratorWorkers, peekSettings, peekWorkers, settingsReadback } from '../../shared/settingsData.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import type { LifecycleAction } from '../lifecycle.ts';
import LifecycleControls from './LifecycleControls.tsx';
import BrainSetup from '../../products/brain/views/BrainSetup.tsx';
import BrainInstall from '../../products/brain/views/BrainInstall.tsx';
import Tasks from '../../products/orchestrator/views/Tasks.tsx';
import { ProductPanel, ProductStatusPanel } from './Products.tsx';
import InstallFacts from './InstallFacts.tsx';
import Hello from './Hello.tsx';
import BrainOverview from '../../products/brain/views/BrainOverview.tsx';
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

// How each product works, in three steps (Orchestrator has no window of its own; Brain works through the AI tools the person already uses).
const HOW: Record<string, [string, string][]> = {
  brain: [
    ['AI araçlarınızı bağlayın', 'Ayarlar sayfasında hangi AI araçlarının (Claude, Codex…) oturumlarının hafızaya yakalanacağını seçersiniz.'],
    ['Oturumlar notlara dönüşür', 'Bir oturum bitince konuşma sır maskelemesinden geçer, AI aracınızla özetlenir ve kasanızdaki günlüğe yazılır.'],
    ['Sonraki oturumlar hatırlar', 'AI aracınız hafızada arayıp ilgili notları getirir. Her şey kendi bilgisayarınızdaki kasada kalır.'],
  ],
  orchestrator: [
    ['Görev verin', 'Görevler sayfasından ya da MCP açıksa AI aracınızdan (Claude, Codex…) bir iş tarif edersiniz.'],
    ['Model izole çalışır', 'İş, projenizin ayrı bir kopyasında yapılır; gerçek dosyalarınıza dokunulmaz. Ayrı bir model sonucu inceler.'],
    ['Siz karar verirsiniz', 'Kanıtlar sayfasında ne değiştiğini görürsünüz. Yama yalnızca sizin onayınızla gerçek projeye uygulanır.'],
  ],
};

// The Overview opens with a greeting and the product's state, then what it did lately, how it works, what was checked and what it installs.
function Overview({ app, eco, go }: { app: AppDef; eco: Ecosystem; go: (page: string) => void }) {
  const [workers, setWorkers] = useState<Record<string, string[] | null> | null>(() => peekWorkers() ?? null);
  const [workersLoading, setWorkersLoading] = useState(native && app.id === 'orchestrator' && !peekWorkers());
  useEffect(() => { if (native && app.id === 'orchestrator') orchestratorWorkers().then(setWorkers).catch(() => setWorkers(null)).finally(() => setWorkersLoading(false)); }, [app.id]);
  const runtime = eco.apps[app.id];
  return <>
    <Hello app={app.id} product={app.name.replace('Respected ', '')} kind={runtime.summary.statusKind} loading={runtime.loading} />
    {app.id === 'orchestrator' && <div className="notice warning">Kurulumdan sonra kullandığınız AI aracını yeniden başlatın ve aracın ilgili yerinden (eklentiler, MCP) Orchestrator'a izin verin. Yoksa araç Orchestrator'ı görmez.</div>}
    {app.id === 'brain' && <div className="notice warning">Kurulumdan sonra kullandığınız AI aracını yeniden başlatın ve aracın ilgili yerinden (eklentiler, hook'lar) Brain'in istediği onayları verin. Onaylamazsanız hafıza yazılmaz.</div>}
    {app.id === 'orchestrator' ? <OverviewPanel runs={eco.runs} workers={workers} workersLoading={workersLoading} onOpenTasks={() => go('runs')} /> : <BrainOverview openMemory={() => go('memory')} />}
    <section className="panel-glass"><div className="section-head"><h2>Nasıl çalışır?</h2></div>
      <ol className="plan-steps">{HOW[app.id]?.map(([title, text]) => <li key={title}><strong>{title}.</strong> {text}</li>)}</ol></section>
    {app.rpp && <ProductStatusPanel id={app.rpp} summary={runtime.summary} />}
    {(app.id === 'brain' || app.id === 'orchestrator') && <section className="panel-glass"><InstallFacts subject={app.id} /></section>}
  </>;
}

// Real engine data only; both calls degrade to honest empty states on failure.
function AgentsPage() {
  const readSettings = (readback: { ok: boolean; result?: unknown } | null | undefined) => readback?.ok && readback.result && typeof readback.result === 'object' ? readback.result as Record<string, unknown> : null;
  const [workers, setWorkers] = useState<Record<string, string[] | null> | null>(() => peekWorkers() ?? null);
  const [settings, setSettings] = useState<Record<string, unknown> | null>(() => readSettings(peekSettings('orchestrator').readback));
  const [loading, setLoading] = useState(!(peekWorkers() && peekSettings('orchestrator').readback));  // the last answers show at once, then refresh
  const [error, setError] = useState('');
  useEffect(() => {
    if (!native) { setLoading(false); return; }
    let active = true;
    // both questions start a program: ask them together, not one after the other
    void Promise.all([
      orchestratorWorkers().then(value => { if (active) setWorkers(value); }, () => { if (active) { setWorkers(null); setError('Worker durumu al\u0131namad\u0131.'); } }),
      settingsReadback('orchestrator').then(readback => { if (active) setSettings(readSettings(readback)); }),
    ]).then(() => { if (active) setLoading(false); });
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
    {app.id === 'orchestrator' && <ProductPanel id="orchestrator" />}
    {app.id === 'orchestrator' && <MemoryAccess />}
    {app.id === 'orchestrator' && <OrchestratorMcp />}
    {app.id === 'orchestrator' && <DataRootSection />}
  </>;
}

export default function AppWorkspace({ appId, page, eco, go, ask, onAsked }: { appId: string; page: string; eco: Ecosystem; go: (page: string) => void; ask?: { product: string; action: LifecycleAction } | null; onAsked?: () => void }) {
  const app = APPS.find(item => item.id === appId);
  if (!app) return <div className="page"><p className="error">Uygulama bulunamadı.</p></div>;
  const view = mountFor(app.id, page);
  const runtime = eco.apps[app.id]; const summary = runtime.summary;
  const pageInfo = app.pages.find(item => item.id === page);
  const lc = eco.lifecycle?.products.find(item => item?.product === app.id) ?? null;
  return <div className={`lib-pane accent-${app.accent}`}>
    <header className="ws-banner"><Weave /><Emblem app={app.id} />
      <div className="ws-title"><h1>{app.name}</h1><p>{app.tagline}</p>
        {!runtime.loading && eco.busy !== app.id && summary.statusKind !== 'ok' && view !== 'overview' && <span className="ws-meta"><button className="link-btn" onClick={() => go('overview')}>Ürün bir sorun bildiriyor: Genel bakışı aç</button></span>}
</div>
      <LifecycleControls name={app.name} lc={lc} onChanged={eco.refresh} onBusy={on => eco.setBusy(on ? app.id : null)} onWizard={app.id === 'brain' ? () => go('settings') : undefined} autoAction={ask?.product === app.id ? ask.action : undefined} onAutoDone={onAsked} />
      <div className="ws-tabs" role="tablist" aria-label={`${app.name} bölümleri`}>{app.pages.map(item => <button key={item.id} role="tab" aria-selected={view === mountFor(app.id, item.id)} className="tab" onClick={() => go(item.id)}>{item.label}</button>)}</div>
    </header>
    <section className="ws-page" aria-label={pageInfo?.label}>
      <div className="ws-head"><h2>{pageInfo?.label}</h2><p className="muted">{pageInfo?.description}</p></div>
      <div className="ws-stack">
      {view === 'overview' && <Overview app={app} eco={eco} go={go} />}
      {view === 'settings' && app.id === 'brain' && (() => { const lc = eco.lifecycle?.products.find(item => item?.product === 'brain'); return lc?.install_state === 'absent' && lc.available
        ? <BrainInstall onDone={eco.refresh} />
        : <BrainSetup onDone={eco.refresh} />; })()}
      {view === 'runs' && <Tasks />}
      {view === 'campaigns' && <Campaigns />}
      {view === 'agents' && <AgentsPage />}
      {view === 'evidence' && <EvidencePage runs={eco.runs} />}
      {view === 'memory' && <><MemoryWorkspace /><MemoryIndex /></>}
      {view === 'settings' && app.id !== 'brain' && <SettingsPage app={app} />}
      </div>
    </section>
  </div>;
}