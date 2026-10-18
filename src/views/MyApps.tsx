import { APPS } from '../apps.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { Emblem, Icon, Weave } from '../ui.tsx';
import type { Go } from './Home.tsx';
import LifecycleControls from './LifecycleControls.tsx';

export default function MyApps({ eco, go }: { eco: Ecosystem; go: Go }) {
  return <div className="page">
    <div className="page-head"><div><span className="eyebrow">Kütüphane</span><h1>Uygulamalarım</h1><p>Hub'a bağlı ürünler. Durum, sürüm ve yetenekler ürünün kendi sağlık yanıtından okunur.</p></div><button onClick={eco.refresh}><Icon name="refresh" />Durumu yenile</button></div>
    <div className="lib-list">{APPS.map(app => { const runtime = eco.apps[app.id]; const summary = runtime.summary; const needsSetup = runtime.configSource === 'defaults' && app.pages.some(page => page.id === 'setup');
      const installed = eco.lifecycle ? !!eco.lifecycle.products.find(item => item?.product === app.id)?.installed : true;   // before the lifecycle answer arrives, do not hide anything
      return <article key={app.id} className={`lib-item accent-${app.accent}`}>
        <div className="lib-art"><Weave /><Emblem app={app.id} /></div>
        <div className="lib-body"><h2>{app.name}</h2><p>{app.tagline}</p>
          <p className="meta"><span className={`badge-status ${runtime.loading ? 'unreachable' : summary.statusKind}`}><span className={`dot ${runtime.loading ? '' : summary.statusKind}`} aria-hidden="true" />{runtime.loading ? 'Kontrol ediliyor…' : summary.statusText}</span>
            {summary.version && <span className="ver">v{summary.version}</span>}{needsSetup && <em className="pill">Kurulum bekliyor</em>}</p>
          {summary.capabilities.length > 0 && <ul className="capabilities" aria-label="Yetenekler">{summary.capabilities.map(cap => <li key={cap}>{cap}</li>)}</ul>}
          {summary.checks.length > 0 && <ul className="checks compact" aria-label="Sağlık kontrolleri">{summary.checks.map(check => <li key={check.name} className={check.ok ? 'ok' : 'bad'}><span aria-hidden="true">{check.ok ? '✓' : '✕'}</span>{check.name}{check.detail ? ` — ${check.detail}` : ''}</li>)}</ul>}
          {summary.errorText && <p className="error" role="alert">{summary.errorText}</p>}
          {installed && app.id === 'orchestrator' && <p className="muted">Orchestrator'ın ayrı bir penceresi yoktur: burada, "Çalışma alanını aç" ile görev ve ajanları yönetirsiniz. AI araçlarınızdan kullanmak için çalışma alanında MCP erişimini açın.</p>}
          <LifecycleControls name={app.name} lc={eco.lifecycle?.products.find(item => item?.product === app.id) ?? null} onChanged={eco.refresh} onWizard={app.id === 'secondbrain' ? () => go({ app: app.id, page: 'setup' }) : undefined} /></div>
        {installed && <div className="lib-actions">
          {needsSetup && <button className="cta" onClick={() => go({ app: app.id, page: 'setup' })}>Kurulumu tamamla</button>}
          <button className={needsSetup ? '' : 'cta'} onClick={() => go({ app: app.id, page: 'overview' })}>Çalışma alanını aç</button>
          <button onClick={() => go({ app: app.id, page: 'settings' })}>Ayarlar</button>
        </div>}</article>; })}</div>
    <section className="lib-item sys-component" aria-label="Sistem bileşenleri"><div className="lib-body"><h2>Python çalışma ortamı</h2><p>Ürünler Hub'ın kendi Python'unu kullanır; bilgisayarında Python kurulu olması gerekmez. İlk ürün kurulumunda otomatik kurulur.</p>
      <LifecycleControls name="Python çalışma ortamı" lc={eco.lifecycle?.products.find(item => item?.product === 'runtime') ?? null} onChanged={eco.refresh} /></div></section>
    <div className="truth-bar"><Icon name="info" /><span>Hub'dan kurulan uygulamalar, katalog paketinden doğrulanmış bir kopya olarak Hub'ın yönettiği klasöre yerleştirilir; verileriniz ayrı kalır. Kurulu değilse Hub, Bağlantılar'daki geliştirme kaynağını kullanır.{eco.lifecycle && <> Yönetilen klasör: <code>{eco.lifecycle.managed_root}</code></>}</span></div>
  </div>;
}
