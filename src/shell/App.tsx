import { useEffect, useState } from 'react';
import { APPS, HUB_NAV, SETTINGS_TABS, visibleApps, type HubRoute, type SettingsTab } from './apps.ts';
import { native, startupIntent, type StartupIntent } from '../shared/api.ts';
import { describeLifecycleError, hubUpdateInstall, hubUpdateStatus, type HubUpdate, type LifecycleAction } from './lifecycle.ts';
import { useEcosystem } from './useEcosystem.ts';
import { globalHealth } from './shellModel.ts';
import AppWorkspace from './views/AppWorkspace.tsx';
import Home, { type Go } from './views/Home.tsx';
import MyApps from './views/MyApps.tsx';
import Discover from './views/Discover.tsx';
import SettingsHub from './views/SettingsHub.tsx';
import { Emblem, Icon } from '../shared/ui.tsx';

type Route = { kind: Exclude<HubRoute, 'settings'> } | { kind: 'settings'; tab: SettingsTab } | { kind: 'app'; app: string; page: string };
const HOME: Route = { kind: 'home' };
// A route saved by an older Hub: product status and activity are Settings tabs now.
const restore = (saved: { kind?: unknown; tab?: unknown } | null): Route => {
  if (!saved || typeof saved.kind !== 'string') return HOME;
  if (saved.kind === 'connections') return { kind: 'settings', tab: 'status' };
  if (saved.kind === 'activity') return { kind: 'settings', tab: 'activity' };
  if (saved.kind === 'settings') return { kind: 'settings', tab: SETTINGS_TABS.some(item => item.id === saved.tab) ? saved.tab as SettingsTab : 'general' };
  return saved as Route;
};

// Two navigation levels: the global rail (Hub) and per-app tabs rendered inside each workspace.
export default function App() {
  // A WebView reload (renderer recovery) must not throw the user back to Home in the middle of an install: keep the page for this session.
  const [route, setRouteState] = useState<Route>(() => { try { return restore(JSON.parse(sessionStorage.getItem('hub.route') ?? 'null')); } catch { return HOME; } });
  const setRoute = (next: Route) => { setRouteState(next); try { sessionStorage.setItem('hub.route', JSON.stringify(next)); } catch { /* optional convenience */ } };
  // A lifecycle action the person already asked for elsewhere (Windows Installed apps, an update notice): My apps opens its plan.
  const [ask, setAsk] = useState<{ product: string; action: LifecycleAction } | null>(null);
  const openIntent = (wanted: StartupIntent | null) => {
    if (!wanted) return;
    if (wanted.action === 'install' && wanted.product === 'brain') { setRoute({ kind: 'app', app: 'brain', page: 'setup' }); return; }
    setRoute({ kind: wanted.action === 'install' ? 'discover' : 'apps' });   // a product is installed from Discover, removed from the Library
    if (wanted.action === 'uninstall' || (wanted.action === 'install' && wanted.product === 'orchestrator')) setAsk({ product: wanted.product, action: wanted.action });
  };
  // A product installer or an Installed-apps entry started while the Hub is already open forwards its choice here instead of opening a second window.
  useEffect(() => {
    if (!native) return;
    let stop: (() => void) | undefined;
    let cancelled = false;
    import('@tauri-apps/api/event').then(({ listen }) => listen<StartupIntent>('startup-intent', event => openIntent(event.payload)))
      .then(off => { if (cancelled) off(); else stop = off; }).catch(() => undefined);
    return () => { cancelled = true; stop?.(); };
  }, []);
  // The reason the Hub was started (product installer, Installed apps), once per session (a reload must not undo later navigation).
  useEffect(() => {
    if (!native || sessionStorage.getItem('hub.intent.done')) return;
    sessionStorage.setItem('hub.intent.done', '1');
    startupIntent().then(openIntent).catch(() => undefined);
  }, []);
  // Updates are offered, never installed silently: the person says "update" or "later".
  const [hubUpdate, setHubUpdate] = useState<HubUpdate | null>(null);
  const [later, setLater] = useState<string[]>(() => { try { return JSON.parse(sessionStorage.getItem('hub.later') ?? '[]'); } catch { return []; } });
  const postpone = (key: string) => { const next = [...later, key]; setLater(next); try { sessionStorage.setItem('hub.later', JSON.stringify(next)); } catch { /* optional convenience */ } };
  const [hubInstall, setHubInstall] = useState('');
  const readHubUpdate = () => { if (native) void hubUpdateStatus().then(reply => { if (reply.ok) setHubUpdate(reply.result); }); };
  // the startup catalog check runs in the background and announces itself when done (offline: the last verified catalog stays)
  useEffect(() => {
    readHubUpdate();
    if (!native) return;
    let stop: (() => void) | undefined;
    let cancelled = false;
    import('@tauri-apps/api/event').then(({ listen }) => listen('catalog-updated', () => { readHubUpdate(); eco.refresh(); }))
      .then(off => { if (cancelled) off(); else stop = off; }).catch(() => undefined);
    return () => { cancelled = true; stop?.(); };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
  async function installHubUpdate() {
    setHubInstall('Hub güncellemesi indiriliyor ve doğrulanıyor…');
    const reply = await hubUpdateInstall();
    setHubInstall(reply.ok ? 'Güncelleme başlatıldı; Hub kapanıp yeniden açılacak.' : describeLifecycleError(reply.error));
  }
  const [spin, setSpin] = useState(false);
  const eco = useEcosystem();
  const go: Go = target => setRoute(typeof target !== 'string' ? { kind: 'app', ...target } : target === 'connections' ? { kind: 'settings', tab: 'status' } : target === 'activity' ? { kind: 'settings', tab: 'activity' } : { kind: target });
  const app = route.kind === 'app' ? APPS.find(item => item.id === route.app) : undefined;
  const shown = visibleApps(eco.lifecycle, native);
  const kinds = shown.map(item => eco.apps[item.id]?.summary.statusKind ?? 'unreachable' as const);
  const health = globalHealth(native, shown.some(item => eco.apps[item.id]?.loading ?? true), kinds);
  const hubLabel = HUB_NAV.find(item => item.id === route.kind)?.label;
  const settingsTab = route.kind === 'settings' ? SETTINGS_TABS.find(item => item.id === route.tab)?.label : undefined;
  const pageLabel = app ? app.pages.find(page => page.id === (route as { page: string }).page)?.label : undefined;
  const navItem = (id: string, label: string, active: boolean) => <button key={id} className={`nav-item ${active ? 'active' : ''}`} aria-current={active ? 'page' : undefined} title={label} onClick={() => setRoute(id === 'settings' ? { kind: 'settings', tab: 'general' } : { kind: id as Exclude<HubRoute, 'settings'> })}><Icon name={id} /><span className="nav-text">{label}</span></button>;
  const refresh = () => { eco.refresh(); setSpin(true); window.setTimeout(() => setSpin(false), 800); };
  const names: Record<string, string> = { brain: 'Brain', orchestrator: 'Orchestrator' };
  const productUpdates = (eco.lifecycle?.products ?? []).filter(item => item?.install_state === 'update-available' && item.available && !later.includes(`${item.product}@${item.available.version}`));
  const hubOffer = hubUpdate?.available && !later.includes(`hub@${hubUpdate.available.version}`) ? hubUpdate.available : null;

  return <div className="shell">
    <aside className="rail" aria-label="Hub gezinmesi">
      <div className="brand"><span className="mark" aria-hidden="true" /><div><strong>Respected</strong><small>Hub</small></div></div>
      <nav aria-label="Hub sayfaları">{HUB_NAV.map(item => navItem(item.id, item.label, route.kind === item.id))}</nav>
      <span className="nav-label">Çalışma alanları</span>
      <nav aria-label="Uygulama çalışma alanları">
        {shown.map(a => { const active = route.kind === 'app' && route.app === a.id; return <button key={a.id} title={a.name} className={`nav-item sub accent-${a.accent} ${active ? 'active' : ''}`} aria-current={active ? 'page' : undefined} onClick={() => setRoute({ kind: 'app', app: a.id, page: 'overview' })}>
          <Emblem app={a.id} className="emblem-sm" /><span className="nav-text">{a.name.replace('Respected ', '')}</span><span className={`dot ${eco.apps[a.id]?.summary.statusKind ?? 'unreachable'}`} aria-hidden="true" /></button>; })}
      </nav>
      <div className="rail-foot"><span><span className={`dot ${health.tone === 'idle' || health.tone === 'unreachable' ? '' : health.tone}`} aria-hidden="true" />{health.label}</span><span>{native ? 'Masaüstü' : 'Önizleme'}{hubUpdate ? ` · ${hubUpdate.current}` : ''}</span></div>
    </aside>
    <div className="stage">
      <header className="topbar">
        <div className="crumbs" aria-label="Konum"><span>Respected</span><span className="sep">/</span>{app ? <><span>{app.name.replace('Respected ', '')}</span><span className="sep">/</span><strong>{pageLabel}</strong></> : settingsTab ? <><span>{hubLabel}</span><span className="sep">/</span><strong>{settingsTab}</strong></> : <strong>{hubLabel}</strong>}</div>
        <div className="topbar-actions">
          <span className="health-pill" role="status"><span className={`dot ${health.tone === 'idle' || health.tone === 'unreachable' ? '' : health.tone}`} aria-hidden="true" />{health.label}</span>
          <button className={`icon-btn ${spin ? 'spin' : ''}`} onClick={refresh} aria-label="Durumu yenile" title="Durumu yenile"><Icon name="refresh" /></button>
        </div>
      </header>
      <main className="content">
        {!native && <div className="page" style={{ marginBottom: 20 }}><div className="notice warning">Tarayıcı önizlemesi: gerçek ürün ve görev verisi için Tauri masaüstü sürümünü çalıştır. Burada hiçbir durum uydurulmaz.</div></div>}
        {(hubOffer || hubInstall) && <div className="page" style={{ marginBottom: 16 }}><div className="notice" role="status">
          {hubInstall || <>Respected Hub {hubOffer!.version} hazır (şu an {hubUpdate!.current}). Güncellensin mi?</>}
          {!hubInstall && <span className="actions"><button className="cta" onClick={() => void installHubUpdate()}>Güncelle</button><button onClick={() => postpone(`hub@${hubOffer!.version}`)}>Sonra</button></span>}</div></div>}
        {productUpdates.map(item => <div key={item!.product} className="page" style={{ marginBottom: 16 }}><div className="notice" role="status">
          {names[item!.product] ?? item!.product} için güncelleme var: {item!.version} → {item!.available!.version}. Kurulsun mu?
          <span className="actions"><button className="cta" onClick={() => { setRoute({ kind: 'apps' }); setAsk({ product: item!.product, action: 'update' }); }}>Güncelle</button>
            <button onClick={() => postpone(`${item!.product}@${item!.available!.version}`)}>Sonra</button></span></div></div>)}
        {route.kind === 'home' && <Home eco={eco} go={go} />}
        {route.kind === 'apps' && <MyApps eco={eco} go={go} ask={ask} onAsked={() => setAsk(null)} />}
        {route.kind === 'discover' && <Discover eco={eco} go={go} ask={ask} onAsked={() => setAsk(null)} />}
        {route.kind === 'settings' && <SettingsHub tab={route.tab} onTab={tab => setRoute({ kind: 'settings', tab })} eco={eco} go={go} onCatalogChanged={() => { readHubUpdate(); eco.refresh(); }} />}
        {route.kind === 'app' && <AppWorkspace appId={route.app} page={route.page} eco={eco} go={page => setRoute({ kind: 'app', app: route.app, page })} onStatus={() => setRoute({ kind: 'settings', tab: 'status' })} />}
      </main>
    </div>
  </div>;
}
