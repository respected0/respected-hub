import { native } from '../../shared/api.ts';
import { terminal } from '../../products/orchestrator/tasks.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { statusLabel, statusTone } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import { APPS } from '../apps.ts';
import { formatWhen, journalSentence } from '../lifecycle.ts';

// Everything that happened: tasks the engine ran, installs the Hub did, and each product's last install record.
export default function Activity({ eco }: { eco: Ecosystem; go?: Go }) {
  const running = eco.runs.filter(run => !terminal(run.status)).length;
  const nameOf = (id: string) => APPS.find(app => app.id === id)?.name ?? id;
  // The product's own install record says when it last changed, whoever ran the installer; a Hub journal entry that ends within a few minutes of it explains it as the Hub's.
  const records = (eco.lifecycle?.products ?? []).flatMap(item => item?.installed && item.recorded_at ? [{
    product: item.product, name: nameOf(item.product), version: item.version, at: item.recorded_at,
    viaHub: eco.journal.some(entry => entry.product === item.product && Math.abs(Number(entry.finished_at ?? entry.started_at) - Number(item.recorded_at)) < 5 * 60_000) }] : []);
  return <div className="settings-stack">
    <section className="card" aria-label="Görevler">
      <div className="card-head"><div><h2>Görevler</h2>
        <p>{!native ? 'Görev geçmişi masaüstü uygulamasında görünür.' : eco.runs.length === 0 ? 'Orchestrator bir görev çalıştırdığında durumu ve sonucu burada görünür.' : `${eco.runs.length} görev${running ? `, ${running} tanesi sürüyor` : ''}.`}</p></div></div>
      {native && eco.runs.length > 0 && <ol className="timeline" aria-label="Görev geçmişi">
        {eco.runs.map(run => <li key={run.run_id} className={`tl-item tone-${statusTone(run.status)}`}>
          <span className="t">{run.request?.task ?? run.run_id}</span><span className={`badge tone-${statusTone(run.status)}`}>{statusLabel(run.status)}</span>
          <span className="m"><span>{run.request?.worker ?? 'Hub'}</span>{run.reason && <span>{run.reason}</span>}</span></li>)}
      </ol>}
    </section>
    {eco.journal.length > 0 && <section className="card" aria-label="Kurulum işlemleri">
      <div className="card-head"><div><h2>Kurulum işlemleri</h2><p>Hub'dan yaptığınız kurulum, güncelleme, onarım ve kaldırma işlemleri.</p></div></div>
      <ol className="timeline" aria-label="Kurulum işlemleri">{eco.journal.map(entry => <li key={entry.operation_id} className={`tl-item tone-${entry.status === 'applied' ? 'ok' : entry.status === 'failed' ? 'error' : 'live'}`}>
        <span className="t">{journalSentence(entry, nameOf(entry.product))}</span>
        <span className="m"><span>{formatWhen(entry.started_at)}</span></span></li>)}</ol></section>}
    {records.length > 0 && <section className="card" aria-label="Kurulum kayıtları">
      <div className="card-head"><div><h2>Ürünlerin son kurulum kaydı</h2><p>Kurucuyu (Setup.exe) doğrudan çalıştırarak yaptıklarınız Hub'ın günlüğüne yazılmaz; izleri burada görünür.</p></div></div>
      <ol className="timeline" aria-label="Kurulum kayıtları">{records.map(item => <li key={item.product} className="tl-item tone-ok">
        <span className="t">{item.name} {item.version ? `v${item.version}` : ''}</span>
        <span className="m"><span>{formatWhen(item.at)}</span><span>{item.viaHub ? 'Hub üzerinden' : 'Kurucudan ya da komut satırından'}</span></span></li>)}</ol></section>}
  </div>;
}
