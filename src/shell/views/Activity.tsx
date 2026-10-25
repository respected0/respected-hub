import type { Ecosystem } from '../useEcosystem.ts';
import type { Go } from './Home.tsx';
import { APPS } from '../apps.ts';
import { formatWhen, journalSentence } from '../lifecycle.ts';

// What the Hub did to the programs: installs it ran and each product's last install record (tasks live in the Orchestrator's own Görevler page).
export default function Activity({ eco }: { eco: Ecosystem; go?: Go }) {
  const nameOf = (id: string) => APPS.find(app => app.id === id)?.name ?? id;
  // The product's own install record says when it last changed, whoever ran the installer; a Hub journal entry that ends within a few minutes of it explains it as the Hub's.
  const records = (eco.lifecycle?.products ?? []).flatMap(item => item?.installed && item.recorded_at ? [{
    product: item.product, name: nameOf(item.product), version: item.version, at: item.recorded_at,
    viaHub: eco.journal.some(entry => entry.product === item.product && Math.abs(Number(entry.finished_at ?? entry.started_at) - Number(item.recorded_at)) < 5 * 60_000) }] : []);
  return <div className="settings-stack">
    {eco.journal.length === 0 && records.length === 0 && <p className="muted">Henüz bir kurulum işlemi yok.</p>}
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
