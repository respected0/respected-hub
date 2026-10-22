import { APPS } from '../apps.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { STATE_LABEL } from '../lifecycle.ts';
import { Emblem, Icon, Weave } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';

// What the person can do next with each product (the action itself runs on My Apps).
const STORE_ACTION: Record<keyof typeof STATE_LABEL, string> = { absent: 'Kur', installed: 'Kurulu · Yönet', 'update-available': 'Güncelle', 'repair-needed': 'Onar' };

// Discover lists what the local catalog really contains (verified bundles) and how each product is installed.
export default function Discover({ eco, go }: { eco: Ecosystem; go: Go }) {
  const catalogRoot = eco.lifecycle?.catalog_root;
  return <div className="page">
    <div className="page-head"><div><span className="eyebrow">Keşfet</span><h1>Kurulabilir ürünler</h1><p>Yerel katalogdaki doğrulanmış paketler. Kurulum, güncelleme ve onarım Uygulamalarım sayfasından yönetilir.</p></div></div>
    <section aria-label="Katalog ürünleri"><div className="section-head" style={{ marginBottom: 14 }}><h2>Katalog</h2>{catalogRoot && <span className="mono muted">{catalogRoot}</span>}</div>
      <div className="app-cards">{APPS.map(app => { const lc = eco.lifecycle?.products.find(item => item?.product === app.id) ?? null;
        return <button key={app.id} className={`app-card accent-${app.accent}`} onClick={() => go('apps')}>
          <span className="app-card-art"><Weave /><Emblem app={app.id} /><span className="lib-state">{lc ? STATE_LABEL[lc.install_state] : 'Durum okunamadı'}</span></span>
          <span className="app-card-body"><strong>{app.name}</strong><small>{app.tagline}</small>
            <span className="app-card-foot">
              {lc?.available ? <span className="ver">Paket v{lc.available.version}{lc.available.trust === 'production' ? ' · imzalı' : lc.available.signed ? ' · geliştirme imzası (yerel)' : ' · imzasız yerel'}</span> : <span className="ver">Pakette yok</span>}
              <span className={`card-cta ${lc && lc.install_state !== 'installed' && lc.available ? 'go' : ''}`}>{lc ? STORE_ACTION[lc.install_state] : 'Uygulamalarım'}<Icon name="arrow" /></span>
            </span></span></button>; })}</div></section>
    <div className="truth-bar"><Icon name="shield" /><span>Uzak, imzalı ürün kataloğu henüz yok: bu yüzden internetten ürün önerilmez veya indirilmez. Paketler yerel olarak üretilir (<code>release/tools/build/build_bundle.py</code>), her dosyanın özeti kurulumdan önce ve sonra doğrulanır.</span></div>
  </div>;
}
