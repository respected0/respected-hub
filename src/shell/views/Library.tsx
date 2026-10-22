import { useEffect, useRef, useState } from 'react';
import { APPS } from '../apps.ts';
import { native } from '../../shared/api.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import type { LifecycleAction } from '../lifecycle.ts';
import { Emblem, Icon } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import AppWorkspace from './AppWorkspace.tsx';

// The library: installed products listed on the left, the selected product's page on the right (install state and its actions on top, then its tabs).
// Not-yet-installed products are installed from Discover.
export default function Library({ eco, appId, page, select, go, ask, onAsked }: { eco: Ecosystem; appId?: string; page: string; select: (appId: string, page?: string) => void; go: Go; ask?: { product: string; action: LifecycleAction } | null; onAsked?: () => void }) {
  // a product removed here keeps its entry until the person leaves, so the result of the removal stays readable
  const [seen, setSeen] = useState<string[]>([]);
  useEffect(() => {
    const installed = (eco.lifecycle?.products ?? []).filter(item => item?.installed).map(item => item!.product);
    setSeen(before => installed.every(id => before.includes(id)) ? before : [...new Set([...before, ...installed])]);
  }, [eco.lifecycle]);
  const listed = APPS.filter(app => !native || seen.includes(app.id) || app.id === appId || !!eco.lifecycle?.products.find(item => item?.product === app.id)?.installed);
  const current = listed.find(app => app.id === appId) ?? listed[0];
  const [query, setQuery] = useState('');
  const [wanted, setWanted] = useState({ update: false, problem: false });
  const [open, setOpen] = useState(false);
  const tools = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent | KeyboardEvent) => { if (event instanceof KeyboardEvent ? event.key === 'Escape' : !tools.current?.contains(event.target as Node)) setOpen(false); };
    document.addEventListener('mousedown', close); document.addEventListener('keydown', close);
    return () => { document.removeEventListener('mousedown', close); document.removeEventListener('keydown', close); };
  }, [open]);
  const stateOf = (id: string) => eco.lifecycle?.products.find(item => item?.product === id)?.install_state;
  const hasUpdate = (id: string) => stateOf(id) === 'update-available';
  const hasProblem = (id: string) => stateOf(id) === 'repair-needed' || (!eco.apps[id]?.loading && eco.apps[id]?.summary.statusKind !== 'ok');
  const active = wanted.update || wanted.problem;
  // checked filters add up (an app matching any of them is shown); none checked shows everything
  const matches = listed.filter(app => app.name.toLowerCase().includes(query.trim().toLowerCase())
    && (!active || (wanted.update && hasUpdate(app.id)) || (wanted.problem && hasProblem(app.id))));

  if (!current) return <div className="page"><section className="card"><h2>{native && !eco.lifecycle ? 'Kurulu uygulamalar okunuyor…' : 'Henüz kurulu uygulama yok'}</h2>
    {eco.lifecycle && <><p style={{ margin: '8px 0 16px' }}>Brain ve Orchestrator'ı Keşfet sayfasından kurabilirsiniz.</p><button className="cta" onClick={() => go('discover')}>Keşfet'e git</button></>}</section></div>;

  return <div className="lib-shell">
    <nav className="lib-side" aria-label="Kurulu uygulamalar">
      <div className="lib-tools" ref={tools}>
        <div className="lib-searchbox"><input type="search" placeholder="Ada göre ara" aria-label="Uygulama ara" value={query} onChange={event => setQuery(event.target.value)} />
          {query && <button type="button" className="lib-clear" aria-label="Aramayı temizle" onClick={() => setQuery('')}><Icon name="x" /></button>}</div>
        <button type="button" className={`lib-filter ${active ? 'on' : ''}`} aria-label="Filtreler" aria-expanded={open} title="Filtreler" onClick={() => setOpen(!open)}><Icon name="filter" /></button>
        {open && <div className="lib-popover" role="dialog" aria-label="Kütüphane filtreleri"><h3>Kütüphane filtreleri</h3>
          <label className="check"><input type="checkbox" checked={wanted.update} onChange={event => setWanted({ ...wanted, update: event.target.checked })} />Güncelleme var</label>
          <label className="check"><input type="checkbox" checked={wanted.problem} onChange={event => setWanted({ ...wanted, problem: event.target.checked })} />Sorun bildirenler</label>
          {active && <button type="button" className="link-btn" onClick={() => setWanted({ update: false, problem: false })}>Filtreleri temizle</button>}</div>}
      </div>
      <div className="lib-group">Kurulu <span>({matches.length})</span></div>
      {matches.length === 0 && <p className="muted lib-none">Eşleşen uygulama yok.</p>}
      {matches.map(app => { const runtime = eco.apps[app.id];
        return <button key={app.id} className={`lib-entry accent-${app.accent} ${app.id === current.id ? 'active' : ''}`} aria-current={app.id === current.id ? 'page' : undefined} onClick={() => select(app.id)}>
          <Emblem app={app.id} small /><strong>{app.name.replace('Respected ', '')}</strong>{hasUpdate(app.id) && <span className="lib-upd" title="Güncelleme var"><Icon name="download" /></span>}
          <span className={`dot ${runtime?.loading ? '' : runtime?.summary.statusKind ?? ''}`} aria-hidden="true" /></button>; })}</nav>
    <div className="lib-main"><AppWorkspace key={current.id} appId={current.id} page={page} eco={eco} go={target => select(current.id, target)} ask={ask} onAsked={onAsked} /></div>
  </div>;
}
