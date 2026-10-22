import { useEffect, useState } from 'react';
import { APPS as CATALOG, visibleApps } from '../apps.ts';
import { homeMode } from '../homeModel.ts';
import { native } from '../../shared/api.ts';
import { terminal } from '../../products/orchestrator/tasks.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { Emblem, Icon, Weave, statusLabel, statusTone } from '../../shared/ui.tsx';
import { knownIdentity, readIdentity } from '../identity.ts';
import BrainOverview from '../../products/brain/views/BrainOverview.tsx';

export type Go = (target: 'apps' | 'discover' | 'activity' | 'settings' | { app: string; page: string }) => void;

export default function Home({ eco, go, hub }: { eco: Ecosystem; go: Go; hub?: { version: string; current: string } | null }) {
  const APPS = visibleApps(eco.lifecycle, native);
  const snapshots = APPS.map(app => ({ loading: eco.apps[app.id]?.loading ?? true, kind: eco.apps[app.id]?.summary.statusKind ?? 'unreachable' as const }));
  const mode = homeMode(native, snapshots);
  const running = eco.runs.filter(run => !terminal(run.status));
  const recent = eco.runs.filter(run => terminal(run.status)).slice(0, 4);
  const [assistant, setAssistant] = useState(() => knownIdentity()?.assistant ?? '');
  useEffect(() => {
    let active = true;
    readIdentity().then(identity => { if (active) setAssistant(identity.assistant); }).catch(() => { /* optional personalization */ });
    return () => { active = false; };
  }, []);
  const problems = APPS.filter(app => { const kind = eco.apps[app.id]?.summary.statusKind; return kind && kind !== 'ok'; });
  const live = native && mode !== 'loading';
  const failedStatus = (status: string) => statusTone(status) === 'error';
  const split = [
    { key: 'ok', label: 'Tamamlandı', n: eco.runs.filter(run => run.status === 'completed').length },
    { key: 'live', label: 'Sürüyor', n: running.length },
    { key: 'error', label: 'Başarısız', n: eco.runs.filter(run => failedStatus(run.status)).length },
    { key: 'idle', label: 'Diğer', n: eco.runs.filter(run => terminal(run.status) && run.status !== 'completed' && !failedStatus(run.status)).length },
  ];

  const hero = {
    preview: { eyebrow: 'Tarayıcı önizlemesi', title: 'Gerçek durum masaüstünde görünür', body: 'Bu önizlemede ürünlere bağlanılamaz; hiçbir durum uydurulmaz. Gerçek ekosistem durumunu görmek için Tauri masaüstü uygulamasını çalıştır.', cta: null },
    loading: { eyebrow: 'Kontrol ediliyor', title: 'Ekosistem durumu okunuyor…', body: 'Ürünlerin sağlık bilgisi alınıyor.', cta: null },
    'first-launch': { eyebrow: 'Hoş geldin', title: 'Respected ekosistemini birlikte kuralım', body: "Henüz çalışan bir ürün bulunamadı. Brain ve Orchestrator'ı Kütüphane sayfasından kurabilir veya Bağlantılar bölümünde mevcut bir kaynağı gösterebilirsin.", cta: { label: 'Ürünleri kur', run: () => go('apps') } },
    attention: { eyebrow: 'Dikkat gerekiyor', title: `${problems.length} ürün sağlıklı değil`, body: 'Ürünü açıp kontrolleri ve ayarları inceleyebilirsin.', cta: problems[0] ? { label: `${problems[0].name.replace('Respected ', '')} ürününü aç`, run: () => go({ app: problems[0].id, page: 'overview' }) } : null },
    active: { eyebrow: 'Ekosistem hazır', title: assistant ? `${assistant} ve ekosistemin hazır` : 'Ekosistemin hazır', body: running.length ? `${running.length} görev şu anda çalışıyor.` : 'Çalışan görev yok.', cta: running.length ? { label: 'Çalışan görevleri gör', run: () => go('activity') } : { label: 'Ürünleri keşfet', run: () => go('discover') } },
  }[mode];

  const updates = (eco.lifecycle?.products ?? []).filter(item => item?.installed && item.install_state === 'update-available' && item.available);
  const notes: { key: string; tone: string; text: string; label?: string; run?: () => void }[] = [
    ...(hub ? [{ key: 'hub', tone: 'warn', text: `Respected Hub ${hub.version} hazır (şu an ${hub.current})`, label: 'Güncelle', run: () => go('settings') }] : []),
    ...updates.map(item => ({ key: `u-${item!.product}`, tone: 'warn', text: `${APPS.find(app => app.id === item!.product)?.name ?? item!.product}: v${item!.available!.version} hazır (şu an v${item!.version})`, label: 'Güncelle', run: () => go({ app: item!.product, page: 'overview' }) })),
    ...problems.map(app => ({ key: `p-${app.id}`, tone: 'error', text: `${app.name}: ${eco.apps[app.id].summary.checks.find(check => !check.ok)?.name ?? eco.apps[app.id].summary.statusText}`, label: 'Aç', run: () => go({ app: app.id, page: 'overview' }) })),
  ];
  const has = (id: string) => APPS.some(app => app.id === id);
  const taskRow = (run: (typeof eco.runs)[number]) => <div className="task-row" key={run.run_id}><span className="t">{run.request?.task ?? run.run_id}</span>
    <div className="row"><span className={`badge tone-${statusTone(run.status)}`}>{statusTone(run.status) === 'live' && <span className="dot live" aria-hidden="true" />}{statusLabel(run.status)}</span><small>{run.request?.worker}</small></div></div>;

  return <div className="page home">
    <section className={`hero hero-${mode} ${live ? 'hero-compact' : ''}`} aria-label="Durum özeti"><Weave />
      <div><span className="eyebrow">{hero.eyebrow}</span><h1>{hero.title}</h1><p>{hero.body}</p>
        <div className="hero-actions">{hero.cta && <button className="cta" onClick={hero.cta.run}>{hero.cta.label}<Icon name="arrow" /></button>}</div></div>
      {live && <div className="hero-notes" role="status"><h2>Dikkat gerekenler</h2>
        {notes.length === 0 ? <p>Her şey yolunda: güncelleme ve sorun yok.</p>
          : notes.map(note => <div className="health-row" key={note.key}><span className={`dot ${note.tone}`} style={{ marginTop: 6 }} aria-hidden="true" /><div><strong>{note.text}</strong>{note.label && <button className="link-btn" onClick={note.run}>{note.label}</button>}</div></div>)}</div>}
      <div className="hero-art" aria-hidden="true"><span className="emblem-tile"><span className="mark" /></span></div>
    </section>
    {live && <section className="quick" aria-label="Uygulamalar">{CATALOG.map(app => { const installed = has(app.id);
      return <button key={app.id} className={`quick-card accent-${app.accent}`} onClick={() => go(installed ? { app: app.id, page: 'overview' } : 'discover')}>
        <Emblem app={app.id} /><span><strong>{app.name.replace('Respected ', '')}</strong><small>{app.tagline}</small></span><Icon name="arrow" /></button>; })}</section>}
    <div className="home-stack">
      <section className="side-card" aria-label="Görevler"><h2>Görevler {running.length > 0 && <span className="badge tone-live">{running.length} çalışıyor</span>}<button className="link-btn" onClick={() => go('activity')}>Tüm geçmiş</button></h2>
        {!live ? <p className="muted">Görev verisi masaüstü uygulamasında görünür.</p>
          : running.length + recent.length === 0 ? <p className="muted">Henüz görev yok. Orchestrator'da başlattığınız görevler burada görünür.</p>
          : <>{running.slice(0, 5).map(taskRow)}{recent.map(taskRow)}</>}
        {live && eco.runs.length > 0 && <><div className="runbar" role="img" aria-label={split.map(part => `${part.label}: ${part.n}`).join(', ')}>{split.filter(part => part.n).map(part => <i key={part.key} className={`tone-${part.key}`} style={{ flex: part.n }} />)}</div>
          <div className="runbar-legend">{split.filter(part => part.n).map(part => <span key={part.key}><i className={`tone-${part.key}`} />{part.label} {part.n}</span>)}</div></>}
      </section>
      {live && has('brain') && <section className="side-card" aria-label="Hafızadan"><h2>Hafızadan <button className="link-btn" onClick={() => go({ app: 'brain', page: 'memory' })}>Hafıza</button></h2>
        <BrainOverview flat openMemory={() => go({ app: 'brain', page: 'memory' })} /></section>}
    </div>
  </div>;
}
