import { native } from '../../shared/api.ts';
import { terminal } from '../../products/orchestrator/tasks.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { Icon, statusLabel, statusTone } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import { APPS } from '../apps.ts';
import { formatWhen, journalSentence } from '../lifecycle.ts';

export default function Activity({ eco, go }: { eco: Ecosystem; go: Go }) {
  const running = eco.runs.filter(run => !terminal(run.status)).length;
  const nameOf = (id: string) => APPS.find(app => app.id === id)?.name ?? id;
  // The product's own install record says when it last changed, whoever ran the installer; a Hub journal entry that ends within a few minutes of it explains it as the Hub's.
  const records = (eco.lifecycle?.products ?? []).flatMap(item => item?.installed && item.recorded_at ? [{
    product: item.product, name: nameOf(item.product), version: item.version, at: item.recorded_at,
    viaHub: eco.journal.some(entry => entry.product === item.product && Math.abs(Number(entry.finished_at ?? entry.started_at) - Number(item.recorded_at)) < 5 * 60_000) }] : []);
  return <div className="settings-stack">
    <div className="section-head"><h2>Görev geçmişi{native && eco.runs.length > 0 ? <small className="muted"> · {eco.runs.length} kayıt{running ? `, ${running} tanesi sürüyor` : ''}</small> : null}</h2><button onClick={eco.refresh}><Icon name="refresh" />Yenile</button></div>
    {eco.journal.length > 0 && <section aria-label="Kurulum işlemleri"><div className="section-head" style={{ marginBottom: 14 }}><h2>Hub'dan yapılan kurulum işlemleri</h2></div>
      <p className="muted" style={{ margin: '0 0 12px' }}>Hub'dan yaptığınız kurulum, güncelleme, onarım ve kaldırma işlemleri. Ürünün kurucusunu (Setup.exe) doğrudan çalıştırarak yaptıklarınız Hub'a kaydedilmez; onların izini aşağıdaki kurulum kaydı gösterir.</p>
      <ol className="timeline" aria-label="Kurulum işlemleri">{eco.journal.map(entry => <li key={entry.operation_id} className={`tl-item tone-${entry.status === 'applied' ? 'ok' : entry.status === 'failed' ? 'error' : 'live'}`}>
        <span className="t">{journalSentence(entry, nameOf(entry.product))}</span>
        <span className="m"><span>{formatWhen(entry.started_at)}</span><span className="mono">{entry.operation_id}</span></span></li>)}</ol></section>}
    {records.length > 0 && <section aria-label="Kurulum kayıtları"><div className="section-head" style={{ marginBottom: 14 }}><h2>Ürünlerin son kurulum kaydı</h2></div>
      <ol className="timeline" aria-label="Kurulum kayıtları">{records.map(item => <li key={item.product} className="tl-item tone-ok">
        <span className="t">{item.name} {item.version ? `v${item.version}` : ''}: kurulum kaydı güncellendi</span>
        <span className="m"><span>{formatWhen(item.at)}</span><span>{item.viaHub ? 'Hub üzerinden' : 'Kurucudan ya da komut satırından'}</span></span></li>)}</ol></section>}
    {!native && <section className="card"><h2>Masaüstü uygulaması gerekiyor</h2><p>Bu önizlemede motor görevlerine erişilemiyor. Tauri masaüstü sürümünde gerçek görev geçmişi burada görünür.</p></section>}
    {native && eco.runs.length === 0 && <section className="card"><h2>Henüz görev yok</h2><p style={{ margin: '8px 0 16px' }}>Orchestrator bir görev çalıştırdığında durum ve sonuç burada görünür.</p><button className="cta" onClick={() => go({ app: 'orchestrator', page: 'runs' })}>İlk görevi başlat<Icon name="arrow" /></button></section>}
    {native && eco.runs.length > 0 && <ol className="timeline" aria-label="Görev geçmişi">
      {eco.runs.map(run => <li key={run.run_id} className={`tl-item tone-${statusTone(run.status)}`}>
        <span className="t">{run.request?.task ?? run.run_id}</span><span className={`badge tone-${statusTone(run.status)}`}>{statusLabel(run.status)}</span>
        <span className="m"><span>{run.request?.worker ?? 'Hub'}</span>{run.phase && <span>Aşama: {run.phase}</span>}{run.reason && <span>{run.reason}</span>}<span className="mono">{run.run_id}</span></span></li>)}
    </ol>}
  </div>;
}
