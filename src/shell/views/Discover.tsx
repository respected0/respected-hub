import { APPS } from '../apps.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { STATE_LABEL, type LifecycleAction } from '../lifecycle.ts';
import { Emblem, Icon, Weave } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import InstallFacts from './InstallFacts.tsx';
import LifecycleControls from './LifecycleControls.tsx';
import type { FactSubject } from '../installFacts.ts';

// Discover is where a product is installed: it lists what the verified catalog offers. What is already installed is managed in the Library.
export default function Discover({ eco, go, ask, onAsked }: { eco: Ecosystem; go: Go; ask?: { product: string; action: LifecycleAction } | null; onAsked?: () => void }) {
  return <div className="page">
    <div className="page-head"><div><span className="eyebrow">Keşfet</span><h1>Kurulabilir uygulamalar</h1><p>Respected kataloğundaki doğrulanmış paketler. Kurduğunuz uygulamalar Kütüphane'de yönetilir.</p></div></div>
    <div className="lib-list">{APPS.map(app => { const lc = eco.lifecycle?.products.find(item => item?.product === app.id) ?? null; const installed = !!lc?.installed;
      return <article key={app.id} className={`lib-item accent-${app.accent}`}>
        <div className="lib-art"><Weave /><Emblem app={app.id} /><span className={`lib-state ${installed ? 'ok' : ''}`}>{lc ? STATE_LABEL[lc.install_state] : 'Durum okunamadı'}</span></div>
        <div className="lib-body"><h2>{app.name}</h2><p className="muted">{app.tagline}</p>
          <p className="meta">{lc?.available ? <span className="ver">Paket v{lc.available.version}{lc.available.trust === 'production' ? ' · imzalı' : lc.available.signed ? ' · geliştirme imzası (yerel)' : ' · imzasız yerel'}</span> : <span className="ver">Katalogda paket yok</span>}</p>
          {installed ? <p className="muted">Bu uygulama kurulu; güncelleme, onarım ve kaldırma Kütüphane'dedir.</p>
            : <LifecycleControls name={app.name} lc={lc} onChanged={eco.refresh} onWizard={app.id === 'brain' ? () => go({ app: app.id, page: 'setup' }) : undefined}
                autoAction={ask?.product === app.id ? ask.action : undefined} onAutoDone={onAsked} />}
          <InstallFacts subject={app.id as FactSubject} /></div>
        {installed && <div className="lib-actions"><button className="cta" onClick={() => go('apps')}>Kütüphanede aç<Icon name="arrow" /></button></div>}
      </article>; })}</div>
    <div className="truth-bar"><Icon name="shield" /><span>Paketler GitHub'daki imzalı Respected kataloğundan gelir. Hub imzayı ve her dosyanın özetini kurulumdan önce ve sonra doğrular; yalnızca siz onaylayınca kurar.</span></div>
  </div>;
}
