import { useEffect, useState } from 'react';
import { APPS } from '../apps.ts';
import { native } from '../../shared/api.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { Emblem, Icon, Weave } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import InstallFacts from './InstallFacts.tsx';
import LifecycleControls from './LifecycleControls.tsx';
import { STATE_LABEL, type LifecycleAction } from '../lifecycle.ts';
import type { FactSubject } from '../installFacts.ts';

const STATE_TONE: Record<keyof typeof STATE_LABEL, string> = { absent: '', installed: 'ok', 'update-available': 'warn', 'repair-needed': 'error' };

// The library: only what is installed. A card shows the install state and the actions; health lives in Settings > Product status,
// so a card only points there when something is wrong. Not-yet-installed products are installed from Discover.
export default function MyApps({ eco, go, ask, onAsked }: { eco: Ecosystem; go: Go; ask?: { product: string; action: LifecycleAction } | null; onAsked?: () => void }) {
  // a product removed on this page keeps its card until the person leaves, so the result of the removal stays readable
  const [seen, setSeen] = useState<string[]>([]);
  useEffect(() => {
    const installed = (eco.lifecycle?.products ?? []).filter(item => item?.installed).map(item => item!.product);
    setSeen(before => installed.every(id => before.includes(id)) ? before : [...new Set([...before, ...installed])]);
  }, [eco.lifecycle]);
  const apps = APPS.filter(app => !native || seen.includes(app.id) || !!eco.lifecycle?.products.find(item => item?.product === app.id)?.installed);
  return <div className="page">
    <div className="page-head"><div><span className="eyebrow">Uygulamalarım</span><h1>Kütüphane</h1><p>Bu bilgisayara kurulu Respected uygulamaları. Yeni uygulama kurmak için Keşfet'e bakın.</p></div><button onClick={eco.refresh}><Icon name="refresh" />Durumu yenile</button></div>
    {native && !eco.lifecycle && <p className="muted" role="status">Kurulu uygulamalar okunuyor…</p>}
    {native && eco.lifecycle && apps.length === 0 && <section className="card"><h2>Henüz kurulu uygulama yok</h2><p style={{ margin: '8px 0 16px' }}>Brain ve Orchestrator'ı Keşfet sayfasından kurabilirsiniz.</p><button className="cta" onClick={() => go('discover')}>Keşfet'e git<Icon name="arrow" /></button></section>}
    <div className="lib-list">{apps.map(app => { const runtime = eco.apps[app.id]; const summary = runtime.summary;
      const lc = eco.lifecycle?.products.find(item => item?.product === app.id) ?? null;
      const installed = native ? !!lc?.installed : true;
      const problem = !runtime.loading && summary.statusKind !== 'ok';
      return <article key={app.id} className={`lib-item accent-${app.accent}`}>
        <div className="lib-art"><Weave /><Emblem app={app.id} />{lc && <span className={`lib-state ${STATE_TONE[lc.install_state]}`}>{STATE_LABEL[lc.install_state]}</span>}</div>
        <div className="lib-body"><h2>{app.name}</h2><p className="muted">{app.tagline}</p>
          {lc?.version && <p className="meta"><span className="ver">Kurulu sürüm v{lc.version}</span></p>}
          {installed && problem && <p className="notice" role="status">Ürün bir sorun bildiriyor. <button className="link-btn" onClick={() => go({ app: app.id, page: 'overview' })}>Genel bakışı aç</button></p>}
          {installed && app.id === 'orchestrator' && <p className="muted">Orchestrator'ın ayrı bir penceresi yoktur: "Çalışma alanını aç" ile görev ve ajanları yönetirsiniz. AI araçlarınızdan kullanmak için çalışma alanında MCP erişimini açın.</p>}
          <LifecycleControls name={app.name} lc={lc} onChanged={eco.refresh} onWizard={app.id === 'brain' ? () => go({ app: app.id, page: 'setup' }) : undefined}
            autoAction={ask?.product === app.id ? ask.action : undefined} onAutoDone={onAsked} />
          <InstallFacts subject={app.id as FactSubject} /></div>
        {installed && <div className="lib-actions">
          <button className="cta" onClick={() => go({ app: app.id, page: 'overview' })}>Çalışma alanını aç</button>
          <button onClick={() => go({ app: app.id, page: 'settings' })}>Ayarlar</button>
        </div>}</article>; })}</div>
    <div className="truth-bar"><Icon name="info" /><span>Uygulamalar GitHub'daki imzalı Respected kataloğundan indirilir, doğrulanır ve ürünün kendi klasörüne kurulur; verileriniz ve notlarınız ayrı kalır. Her uygulama Windows Ayarlar › Uygulamalar listesinde de ayrı görünür; oradan kaldırabilir veya değiştirebilirsiniz.</span></div>
  </div>;
}
