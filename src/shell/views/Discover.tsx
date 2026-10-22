import { useState } from 'react';
import { APPS } from '../apps.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { STATE_LABEL, type LifecycleAction } from '../lifecycle.ts';
import { Emblem, Weave } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import LifecycleControls from './LifecycleControls.tsx';

// Discover is the store: one featured app with its introduction and version, and the whole catalog as a strip below it.
// Install here; an installed app opens in the Library. A product page per app can grow from the featured panel later.
export default function Discover({ eco, go, ask, onAsked }: { eco: Ecosystem; go: Go; ask?: { product: string; action: LifecycleAction } | null; onAsked?: () => void }) {
  const [picked, setPicked] = useState(ask?.product ?? APPS[0]?.id);
  const featured = APPS.find(app => app.id === picked) ?? APPS[0];
  const lifecycleOf = (id: string) => eco.lifecycle?.products.find(item => item?.product === id) ?? null;
  const lc = lifecycleOf(featured.id); const installed = !!lc?.installed;
  const version = lc?.available?.version ?? lc?.version;
  return <div className="page store">
    <section className={`store-hero accent-${featured.accent}`} aria-label="Öne çıkan uygulama">
      <div className="store-art"><Weave /><Emblem app={featured.id} /></div>
      <aside className="store-panel">
        <h1>{featured.name}</h1><p>{featured.tagline}</p>
        <p className="meta">{version ? <span className="ver">v{version}</span> : <span className="ver">Katalogda paket yok</span>}</p>
        {installed
          ? <div className="actions"><button className="cta" onClick={() => go({ app: featured.id, page: 'overview' })}>Kütüphanede aç</button></div>
          : <LifecycleControls key={featured.id} name={featured.name} lc={lc} onChanged={eco.refresh} onWizard={featured.id === 'brain' ? () => go({ app: featured.id, page: 'settings' }) : undefined}
              autoAction={ask?.product === featured.id ? ask.action : undefined} onAutoDone={onAsked} />}
      </aside>
    </section>
    <section aria-label="Katalog"><div className="section-head" style={{ marginBottom: 'var(--s-4)' }}><h2>Uygulamalar</h2></div>
      <div className="store-tiles">{APPS.map(app => { const item = lifecycleOf(app.id);
        return <button key={app.id} className={`store-tile accent-${app.accent} ${app.id === featured.id ? 'on' : ''}`} aria-pressed={app.id === featured.id} onClick={() => setPicked(app.id)}>
          <span className="store-tile-art"><Weave /><Emblem app={app.id} /></span>
          <span className="store-tile-body"><strong>{app.name}</strong><small>{app.tagline}</small>
            <span className="store-tile-foot"><span className="ver">{item?.available?.version ?? item?.version ? `v${item?.available?.version ?? item?.version}` : ''}</span><span className={`badge-status ${item?.installed ? 'ok' : 'unreachable'}`}>{item ? STATE_LABEL[item.install_state] : '…'}</span></span></span>
        </button>; })}</div></section>
  </div>;
}
