import { APPS } from '../apps.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { Emblem, Icon, Weave } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import InstallFacts from './InstallFacts.tsx';
import type { FactSubject } from '../installFacts.ts';
import LifecycleControls from './LifecycleControls.tsx';
import { STATE_LABEL, type LifecycleAction } from '../lifecycle.ts';

const STATE_TONE: Record<keyof typeof STATE_LABEL, string> = { absent: '', installed: 'ok', 'update-available': 'warn', 'repair-needed': 'error' };

// The library: one card per product. The card shows the state and the actions; health checks and "what does this install" fold away.
export default function MyApps({ eco, go, ask, onAsked }: { eco: Ecosystem; go: Go; ask?: { product: string; action: LifecycleAction } | null; onAsked?: () => void }) {
  return <div className="page">
    <div className="page-head"><div><span className="eyebrow">Uygulamalarım</span><h1>Kütüphane</h1><p>Hub'a bağlı ürünler. Durum, sürüm ve yetenekler ürünün kendi sağlık yanıtından okunur.</p></div><button onClick={eco.refresh}><Icon name="refresh" />Durumu yenile</button></div>
    <div className="lib-list">{APPS.map(app => { const runtime = eco.apps[app.id]; const summary = runtime.summary;
      const lc = eco.lifecycle?.products.find(item => item?.product === app.id) ?? null;
      const installed = eco.lifecycle ? !!lc?.installed : true;   // before the lifecycle answer arrives, do not hide anything
      return <article key={app.id} className={`lib-item accent-${app.accent}`}>
        <div className="lib-art"><Weave /><Emblem app={app.id} />{lc && <span className={`lib-state ${STATE_TONE[lc.install_state]}`}>{STATE_LABEL[lc.install_state]}</span>}</div>
        <div className="lib-body"><h2>{app.name}</h2><p className="muted">{app.tagline}</p>
          <p className="meta"><span className={`badge-status ${runtime.loading ? 'unreachable' : summary.statusKind}`}><span className={`dot ${runtime.loading ? '' : summary.statusKind}`} aria-hidden="true" />{runtime.loading ? 'Kontrol ediliyor…' : summary.statusText}</span>
            {summary.version && <span className="ver">v{summary.version}</span>}</p>
          {summary.errorText && <p className="error" role="alert">{summary.errorText}</p>}
          {installed && app.id === 'orchestrator' && <p className="muted">Orchestrator'ın ayrı bir penceresi yoktur: "Çalışma alanını aç" ile görev ve ajanları yönetirsiniz. AI araçlarınızdan kullanmak için çalışma alanında MCP erişimini açın.</p>}
          <LifecycleControls name={app.name} lc={lc} onChanged={eco.refresh} onWizard={app.id === 'brain' ? () => go({ app: app.id, page: 'setup' }) : undefined}
            autoAction={ask?.product === app.id ? ask.action : undefined} onAutoDone={onAsked} />
          {(summary.capabilities.length > 0 || summary.checks.length > 0) && <details className="lib-more"><summary>Yetenekler ve sağlık kontrolleri</summary>
            {summary.capabilities.length > 0 && <ul className="capabilities" aria-label="Yetenekler">{summary.capabilities.map(cap => <li key={cap}>{cap}</li>)}</ul>}
            {summary.checks.length > 0 && <ul className="checks compact" aria-label="Sağlık kontrolleri">{summary.checks.map(check => <li key={check.name} className={check.ok ? 'ok' : 'bad'}><span aria-hidden="true">{check.ok ? '✓' : '✕'}</span>{check.name}{check.detail ? ` — ${check.detail}` : ''}</li>)}</ul>}</details>}
          <InstallFacts subject={app.id as FactSubject} /></div>
        {installed && <div className="lib-actions">
          <button className="cta" onClick={() => go({ app: app.id, page: 'overview' })}>Çalışma alanını aç</button>
          <button onClick={() => go({ app: app.id, page: 'settings' })}>Ayarlar</button>
        </div>}</article>; })}</div>
    <div className="truth-bar"><Icon name="info" /><span>Uygulamalar GitHub'daki imzalı Respected kataloğundan indirilir, doğrulanır ve ürünün kendi klasörüne kurulur; verileriniz ve notlarınız ayrı kalır. Her uygulama Windows Ayarlar › Uygulamalar listesinde de ayrı görünür; oradan kaldırabilir veya değiştirebilirsiniz.</span></div>
  </div>;
}
