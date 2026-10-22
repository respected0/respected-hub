import { useEffect, useState } from 'react';
import { visibleApps } from '../apps.ts';
import { homeMode } from '../homeModel.ts';
import { native } from '../../shared/api.ts';
import { terminal } from '../../products/orchestrator/tasks.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { Emblem, Icon, Weave, statusLabel, statusTone } from '../../shared/ui.tsx';
import { rppCall } from '../../shared/productClient.ts';

export type Go = (target: 'apps' | 'connections' | 'discover' | 'activity' | { app: string; page: string }) => void;

export default function Home({ eco, go }: { eco: Ecosystem; go: Go }) {
  const APPS = visibleApps(eco.lifecycle, native);
  const snapshots = APPS.map(app => ({ loading: eco.apps[app.id]?.loading ?? true, kind: eco.apps[app.id]?.summary.statusKind ?? 'unreachable' as const }));
  const mode = homeMode(native, snapshots);
  const running = eco.runs.filter(run => !terminal(run.status));
  const recent = eco.runs.filter(run => terminal(run.status)).slice(0, 4);
  const [assistant, setAssistant] = useState('');
  useEffect(() => {
    let active = true;
    rppCall('brain', 'settings.readback').then(response => {
      const name = response.ok ? (response.result as { settings?: { assistant_name?: unknown } }).settings?.assistant_name : '';
      if (active && typeof name === 'string') setAssistant(name.trim());
    }).catch(() => { /* optional personalization */ });
    return () => { active = false; };
  }, []);
  const problems = APPS.filter(app => { const kind = eco.apps[app.id]?.summary.statusKind; return kind && kind !== 'ok'; });
  const healthy = APPS.length - problems.length;
  const live = native && mode !== 'loading';

  const hero = {
    preview: { eyebrow: 'Tarayıcı önizlemesi', title: 'Gerçek durum masaüstünde görünür', body: 'Bu önizlemede ürünlere bağlanılamaz; hiçbir durum uydurulmaz. Gerçek ekosistem durumunu görmek için Tauri masaüstü uygulamasını çalıştır.', cta: null },
    loading: { eyebrow: 'Kontrol ediliyor', title: 'Ekosistem durumu okunuyor…', body: 'Ürünlerin sağlık bilgisi alınıyor.', cta: null },
    'first-launch': { eyebrow: 'Hoş geldin', title: 'Respected ekosistemini birlikte kuralım', body: "Henüz çalışan bir ürün bulunamadı. Brain ve Orchestrator'ı Kütüphane sayfasından kurabilir veya Bağlantılar bölümünde mevcut bir kaynağı gösterebilirsin.", cta: { label: 'Ürünleri kur', run: () => go('apps') } },
    attention: { eyebrow: 'Dikkat gerekiyor', title: `${problems.length} ürün sağlıklı değil`, body: 'Ürünü açıp kontrolleri ve ayarları inceleyebilirsin.', cta: problems[0] ? { label: `${problems[0].name.replace('Respected ', '')} ürününü aç`, run: () => go({ app: problems[0].id, page: 'overview' }) } : null },
    active: { eyebrow: 'Ekosistem hazır', title: assistant ? `${assistant} ve ekosistemin hazır` : 'Ekosistemin hazır', body: running.length ? `${running.length} görev şu anda çalışıyor.` : 'Çalışan görev yok; yeni bir görevi Orchestrator çalışma alanından başlatabilirsin.', cta: running.length ? { label: 'Çalışan görevleri gör', run: () => go('activity') } : { label: 'Görev başlat', run: () => go({ app: 'orchestrator', page: 'runs' }) } },
  }[mode];

  return <div className="page home">
    <section className={`hero hero-${mode}`} aria-label="Durum özeti"><Weave />
      <div><span className="eyebrow">{hero.eyebrow}</span><h1>{hero.title}</h1><p>{hero.body}</p>
        <div className="hero-actions">{hero.cta && <button className="cta" onClick={hero.cta.run}>{hero.cta.label}<Icon name="arrow" /></button>}
          {live && <button onClick={() => go('apps')}>Kütüphane</button>}</div></div>
      <div className="hero-art" aria-hidden="true"><span className="emblem-tile"><span className="mark" /></span></div>
    </section>
    {live && <section className="metrics" aria-label="Ekosistem özeti">
      <div className="metric"><span>Sağlıklı ürün</span><strong>{healthy}<small>/{APPS.length}</small></strong></div>
      <div className="metric"><span>Çalışan görev</span><strong>{running.length}</strong></div>
      <div className="metric"><span>Kayıtlı görev</span><strong>{eco.runs.length}</strong></div>
    </section>}
    <div className="home-grid">
      <div className="home-main">
        <section aria-label="Uygulamalar"><div className="section-head" style={{ marginBottom: 14 }}><h2>Kütüphane</h2><button className="link-btn" onClick={() => go('apps')}>Tümünü yönet</button></div>
          <div className="app-cards">{APPS.map(app => { const runtime = eco.apps[app.id]; const summary = runtime.summary;
            return <button key={app.id} className={`app-card accent-${app.accent}`} onClick={() => go({ app: app.id, page: 'overview' })}>
              <span className="app-card-art"><Weave /><Emblem app={app.id} />{summary.version && <span className="lib-state">v{summary.version}</span>}</span>
              <span className="app-card-body"><strong>{app.name}</strong><small>{app.tagline}</small>
                <span className="app-card-foot"><span className={`badge-status ${runtime.loading ? 'unreachable' : summary.statusKind}`}><span className={`dot ${runtime.loading ? '' : summary.statusKind}`} aria-hidden="true" />{runtime.loading ? 'Kontrol ediliyor…' : summary.statusText}</span>
                  <span className={`card-cta ${summary.statusKind === 'ok' ? 'go' : ''}`}>Aç<Icon name="arrow" /></span></span></span>
            </button>; })}</div></section>
      </div>
      <aside className="home-side" aria-label="Ürün sağlığı ve görevler">
        <section className="side-card"><h2>Ürün sağlığı</h2>
          {!live && <p className="muted">{native ? 'Okunuyor…' : 'Masaüstü uygulamasında gerçek sağlık bilgisi görünür.'}</p>}
          {live && APPS.map(app => { const summary = eco.apps[app.id].summary; const bad = summary.checks.find(check => !check.ok);
            return <div className="health-row" key={app.id}><span className={`dot ${summary.statusKind}`} style={{ marginTop: 6 }} aria-hidden="true" /><div><strong>{app.name.replace('Respected ', '')}</strong>
              <small>{bad ? `${bad.name}${bad.detail ? ` — ${bad.detail}` : ''}` : summary.checks.length ? `${summary.checks.length} kontrol geçti` : summary.statusText}</small></div></div>; })}
        </section>
        <section className="side-card"><h2>Devam eden görevler {running.length > 0 && <span className="badge tone-live">{running.length}</span>}</h2>
          {!live ? <p className="muted">Görev verisi masaüstü uygulamasında görünür.</p>
            : running.length === 0 ? <p className="muted">Çalışan görev yok.</p>
            : running.slice(0, 4).map(run => <div className="task-row" key={run.run_id}><span className="t">{run.request?.task ?? run.run_id}</span><div className="row"><span className="badge tone-live"><span className="dot live" aria-hidden="true" />{statusLabel(run.status)}</span><small>{run.request?.worker}</small></div></div>)}
        </section>
        {live && recent.length > 0 && <section className="side-card"><h2>Son biten görevler <button className="link-btn" onClick={() => go('activity')}>Tümü</button></h2>
          {recent.map(run => <div className="task-row" key={run.run_id}><span className="t">{run.request?.task ?? run.run_id}</span><div className="row"><span className={`badge tone-${statusTone(run.status)}`}>{statusLabel(run.status)}</span><small>{run.request?.worker}</small></div></div>)}</section>}
      </aside>
    </div>
  </div>;
}
