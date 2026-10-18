import { useEffect, useState } from 'react';
import { APPS, HUB_NAV, visibleApps, type HubRoute } from './apps.ts';
import { native, startupIntent } from './api.ts';
import { useEcosystem } from './useEcosystem.ts';
import { globalHealth } from './shellModel.ts';
import AppWorkspace from './views/AppWorkspace.tsx';
import Connections from './views/Connections.tsx';
import Home, { type Go } from './views/Home.tsx';
import MyApps from './views/MyApps.tsx';
import Discover from './views/Discover.tsx';
import Activity from './views/Activity.tsx';
import Settings from './views/Settings.tsx';
import { Emblem, Icon } from './ui.tsx';

type Route = { kind: HubRoute } | { kind: 'app'; app: string; page: string };

// Two navigation levels: the global rail (Hub) and per-app tabs rendered inside each workspace.
export default function App() {
  // A WebView reload (renderer recovery) must not throw the user back to Home in the middle of an install: keep the page for this session.
  const [route, setRouteState] = useState<Route>(() => { try { const saved = JSON.parse(sessionStorage.getItem('hub.route') ?? 'null'); return saved && typeof saved.kind === 'string' ? saved : { kind: 'home' }; } catch { return { kind: 'home' }; } });
  const setRoute = (next: Route) => { setRouteState(next); try { sessionStorage.setItem('hub.route', JSON.stringify(next)); } catch { /* optional convenience */ } };
  const openProductInstall = (product: string | null) => { if (product === 'secondbrain') setRoute({ kind: 'app', app: 'secondbrain', page: 'setup' }); else if (product === 'orchestrator') setRoute({ kind: 'apps' }); };
  // A product installer started while the Hub is already open forwards its choice here instead of opening a second window.
  useEffect(() => {
    if (!native) return;
    let stop: (() => void) | undefined;
    let cancelled = false;
    import('@tauri-apps/api/event').then(({ listen }) => listen<string>('startup-intent', event => openProductInstall(event.payload)))
      .then(off => { if (cancelled) off(); else stop = off; }).catch(() => undefined);
    return () => { cancelled = true; stop?.(); };
  }, []);
  // The product-specific installer opens Hub on that product's install page, once per session (a reload must not undo later navigation).
  useEffect(() => {
    if (!native || sessionStorage.getItem('hub.intent.done')) return;
    sessionStorage.setItem('hub.intent.done', '1');
    startupIntent().then(openProductInstall).catch(() => undefined);
  }, []);
  const [spin, setSpin] = useState(false);
  const eco = useEcosystem();
  const go: Go = target => setRoute(typeof target === 'string' ? { kind: target } : { kind: 'app', ...target });
  const app = route.kind === 'app' ? APPS.find(item => item.id === route.app) : undefined;
  const shown = visibleApps(eco.lifecycle, native);
  const kinds = shown.map(item => eco.apps[item.id]?.summary.statusKind ?? 'unreachable' as const);
  const health = globalHealth(native, shown.some(item => eco.apps[item.id]?.loading ?? true), kinds);
  const hubLabel = HUB_NAV.find(item => item.id === route.kind)?.label;
  const pageLabel = app ? app.pages.find(page => page.id === (route as { page: string }).page)?.label : undefined;
  const navItem = (id: string, label: string, active: boolean) => <button key={id} className={`nav-item ${active ? 'active' : ''}`} aria-current={active ? 'page' : undefined} title={label} onClick={() => setRoute({ kind: id as HubRoute })}><Icon name={id} /><span className="nav-text">{label}</span></button>;
  const refresh = () => { eco.refresh(); setSpin(true); window.setTimeout(() => setSpin(false), 800); };

  return <div className="shell">
    <aside className="rail" aria-label="Hub gezinmesi">
      <div className="brand"><span className="mark" aria-hidden="true" /><div><strong>Respected</strong><small>Hub</small></div></div>
      <nav aria-label="Hub sayfaları">{HUB_NAV.map(item => navItem(item.id, item.label, route.kind === item.id))}</nav>
      <span className="nav-label">Çalışma alanları</span>
      <nav aria-label="Uygulama çalışma alanları">
        {shown.map(a => { const active = route.kind === 'app' && route.app === a.id; return <button key={a.id} title={a.name} className={`nav-item sub accent-${a.accent} ${active ? 'active' : ''}`} aria-current={active ? 'page' : undefined} onClick={() => setRoute({ kind: 'app', app: a.id, page: 'overview' })}>
          <Emblem app={a.id} className="emblem-sm" /><span className="nav-text">{a.name.replace('Respected ', '')}</span><span className={`dot ${eco.apps[a.id]?.summary.statusKind ?? 'unreachable'}`} aria-hidden="true" /></button>; })}
      </nav>
      <div className="rail-foot"><span><span className={`dot ${health.tone === 'idle' || health.tone === 'unreachable' ? '' : health.tone}`} aria-hidden="true" />{health.label}</span><span>{native ? 'Masaüstü' : 'Önizleme'} · 0.1.0</span></div>
    </aside>
    <div className="stage">
      <header className="topbar">
        <div className="crumbs" aria-label="Konum"><span>Respected</span><span className="sep">/</span>{app ? <><span>{app.name.replace('Respected ', '')}</span><span className="sep">/</span><strong>{pageLabel}</strong></> : <strong>{hubLabel}</strong>}</div>
        <div className="topbar-actions">
          <span className="health-pill" role="status"><span className={`dot ${health.tone === 'idle' || health.tone === 'unreachable' ? '' : health.tone}`} aria-hidden="true" />{health.label}</span>
          <button className={`icon-btn ${spin ? 'spin' : ''}`} onClick={refresh} aria-label="Durumu yenile" title="Durumu yenile"><Icon name="refresh" /></button>
        </div>
      </header>
      <main className="content">
        {!native && <div className="page" style={{ marginBottom: 20 }}><div className="notice warning">Tarayıcı önizlemesi: gerçek ürün ve görev verisi için Tauri masaüstü sürümünü çalıştır. Burada hiçbir durum uydurulmaz.</div></div>}
        {route.kind === 'home' && <Home eco={eco} go={go} />}
        {route.kind === 'apps' && <MyApps eco={eco} go={go} />}
        {route.kind === 'discover' && <Discover eco={eco} go={go} />}
        {route.kind === 'connections' && <div className="page"><div className="page-head"><div><span className="eyebrow">Ürün durumu</span><h1>Ürün durumu ve bağlantı denetimi</h1><p>Hub kurulu her ürünle kendi protokolü (RPP) üzerinden konuşur. Burada her ürünün bağlantısı, sürümü ve sağlık kontrolleri görünür; bir şey çalışmıyorsa nedenini buradan okuyun. Ürün kurmak ya da kaldırmak için Uygulamalarım'ı kullanın.</p></div></div><Connections /></div>}
        {route.kind === 'activity' && <Activity eco={eco} go={go} />}
        {route.kind === 'settings' && <Settings />}
        {route.kind === 'app' && <AppWorkspace appId={route.app} page={route.page} eco={eco} go={page => setRoute({ kind: 'app', app: route.app, page })} />}
      </main>
    </div>
  </div>;
}
