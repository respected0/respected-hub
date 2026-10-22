import { useEffect, useState } from 'react';
import { HUB_NAV, SETTINGS_TABS, type HubRoute, type SettingsTab } from './apps.ts';
import { native, startupIntent, type StartupIntent } from '../shared/api.ts';
import { hubUpdateStatus, type HubUpdate, type LifecycleAction } from './lifecycle.ts';
import { useEcosystem } from './useEcosystem.ts';
import Home, { type Go } from './views/Home.tsx';
import Library from './views/Library.tsx';
import { readIdentity } from './identity.ts';
import WindowControls from './WindowControls.tsx';
import Discover from './views/Discover.tsx';
import SettingsHub from './views/SettingsHub.tsx';

type Route = { kind: Exclude<HubRoute, 'settings' | 'discover'> } | { kind: 'discover'; app?: string } | { kind: 'settings'; tab: SettingsTab } | { kind: 'app'; app: string; page: string };
const HOME: Route = { kind: 'home' };
// A route saved by an older Hub: product status and activity are Settings tabs now.
const restore = (saved: { kind?: unknown; tab?: unknown } | null): Route => {
  if (!saved || typeof saved.kind !== 'string') return HOME;
  if (saved.kind === 'connections') return { kind: 'settings', tab: 'general' };   // product status moved into each product's Overview
  if (saved.kind === 'activity') return { kind: 'settings', tab: 'activity' };
  if (saved.kind === 'settings') return { kind: 'settings', tab: SETTINGS_TABS.some(item => item.id === saved.tab) ? saved.tab as SettingsTab : 'general' };
  return saved as Route;
};

// Two navigation levels: the top navigation (Hub) and, in the Library, the installed products with their own tabs.
export default function App() {
  // A WebView reload (renderer recovery) must not throw the user back to Home in the middle of an install: keep the page for this session.
  const [route, setRouteState] = useState<Route>(() => { try { return restore(JSON.parse(sessionStorage.getItem('hub.route') ?? 'null')); } catch { return HOME; } });
  const setRoute = (next: Route) => { setRouteState(next); try { sessionStorage.setItem('hub.route', JSON.stringify(next)); } catch { /* optional convenience */ } };
  // A lifecycle action the person already asked for elsewhere (Windows Installed apps, an update notice): My apps opens its plan.
  const [ask, setAsk] = useState<{ product: string; action: LifecycleAction } | null>(null);
  const openIntent = (wanted: StartupIntent | null) => {
    if (!wanted) return;
    if (wanted.action === 'install' && wanted.product === 'brain') { setRoute({ kind: 'app', app: 'brain', page: 'settings' }); return; }
    // a product is installed from Discover, managed (updated, repaired, removed) in its Library page
    setRoute(wanted.action === 'install' ? { kind: 'discover' } : { kind: 'app', app: wanted.product, page: 'overview' });
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
  // Updates are offered, never installed silently: Home points at Settings > Updates, where the person starts the Hub update.
  const [hubUpdate, setHubUpdate] = useState<HubUpdate | null>(null);
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
  const eco = useEcosystem();
  // the greeting's name is ready before the first Brain page opens
  useEffect(() => { void readIdentity().catch(() => undefined); }, []);
  const go: Go = target => setRoute(typeof target !== 'string' ? ('store' in target ? { kind: 'discover', app: target.store } : { kind: 'app', ...target }) : target === 'activity' ? { kind: 'settings', tab: 'activity' } : target === 'settings' ? { kind: 'settings', tab: 'general' } : { kind: target });
  const navItem = (id: string, label: string, active: boolean) => <button key={id} className={`nav-item ${active ? 'active' : ''}`} aria-current={active ? 'page' : undefined} onClick={() => setRoute(id === 'settings' ? { kind: 'settings', tab: 'general' } : { kind: id as Exclude<HubRoute, 'settings'> })}>{label}</button>;
  const hubOffer = hubUpdate?.available ?? null;

  return <div className="shell">
    <header className="topnav" data-tauri-drag-region>
      <nav aria-label="Hub sayfaları">{HUB_NAV.map(item => navItem(item.id, item.label, route.kind === item.id || (item.id === 'apps' && route.kind === 'app')))}</nav>
      <div className="topnav-spacer" data-tauri-drag-region />
      {native && <WindowControls />}
    </header>
    <main className="content">
        {!native && <div className="page" style={{ marginBottom: 20 }}><div className="notice warning">Tarayıcı önizlemesi: gerçek ürün ve görev verisi için Tauri masaüstü sürümünü çalıştır. Burada hiçbir durum uydurulmaz.</div></div>}
        {route.kind === 'home' && <Home eco={eco} go={go} hub={hubOffer ? { version: hubOffer.version, current: hubUpdate!.current } : null} />}
        {route.kind === 'discover' && <Discover key={route.app ?? ''} appId={route.app} eco={eco} go={go} ask={ask} onAsked={() => setAsk(null)} />}
        {route.kind === 'settings' && <SettingsHub tab={route.tab} onTab={tab => setRoute({ kind: 'settings', tab })} eco={eco} go={go} onCatalogChanged={() => { readHubUpdate(); eco.refresh(); }} />}
        {(route.kind === 'apps' || route.kind === 'app') && <Library eco={eco} appId={route.kind === 'app' ? route.app : undefined} page={route.kind === 'app' ? route.page : 'overview'} select={(appId, page = 'overview') => setRoute({ kind: 'app', app: appId, page })} go={go} ask={ask} onAsked={() => setAsk(null)} />}
    </main>
  </div>;
}
