import { native } from '../../shared/api.ts';
import { terminal } from '../../products/orchestrator/tasks.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import { Icon, statusLabel, statusTone } from '../../shared/ui.tsx';
import type { Go } from './Home.tsx';
import { APPS } from '../apps.ts';
import { canRollback, describeLifecycleError, formatWhen, journalSentence, lifecycleRollback } from '../lifecycle.ts';
import { useState } from 'react';

export default function Activity({ eco, go }: { eco: Ecosystem; go: Go }) {
  const running = eco.runs.filter(run => !terminal(run.status)).length;
  const [error, setError] = useState('');
  const nameOf = (id: string) => APPS.find(app => app.id === id)?.name ?? (id === 'runtime' ? 'Python çalışma ortamı' : id);
  async function undo(id: string) {
    setError('');
    const reply = await lifecycleRollback(id);
    if (!reply.ok) setError(describeLifecycleError(reply.error));
    eco.refresh();
  }
  return <div className="page">
    <div className="page-head"><div><span className="eyebrow">Etkinlik</span><h1>Görev geçmişi</h1><p>Orchestrator motorunun gerçek görev kayıtları. {native && eco.runs.length > 0 ? `${eco.runs.length} kayıt${running ? `, ${running} tanesi sürüyor` : ''}.` : ''}</p></div><button onClick={eco.refresh}><Icon name="refresh" />Yenile</button></div>
    {eco.journal.length > 0 && <section aria-label="Kurulum işlemleri"><div className="section-head" style={{ marginBottom: 14 }}><h2>Kurulum işlemleri</h2></div>
      {error && <p className="error" role="alert">{error}</p>}
      <ol className="timeline" aria-label="Kurulum işlemleri">{eco.journal.map(entry => <li key={entry.operation_id} className={`tl-item tone-${entry.status === 'applied' ? 'ok' : entry.status === 'failed' ? 'error' : entry.status === 'rolled_back' ? 'idle' : 'live'}`}>
        <span className="t">{journalSentence(entry, nameOf(entry.product))}</span>
        <span className="m"><span>{formatWhen(entry.started_at)}</span><span className="mono">{entry.operation_id}</span>{canRollback(entry, eco.journal) && <button onClick={() => void undo(entry.operation_id)}>Geri al</button>}</span></li>)}</ol></section>}
    {!native && <section className="card"><h2>Masaüstü uygulaması gerekiyor</h2><p>Bu önizlemede motor görevlerine erişilemiyor. Tauri masaüstü sürümünde gerçek görev geçmişi burada görünür.</p></section>}
    {native && eco.runs.length === 0 && <section className="card"><h2>Henüz görev yok</h2><p style={{ margin: '8px 0 16px' }}>Orchestrator bir görev çalıştırdığında durum ve sonuç burada görünür.</p><button className="cta" onClick={() => go({ app: 'orchestrator', page: 'runs' })}>İlk görevi başlat<Icon name="arrow" /></button></section>}
    {native && eco.runs.length > 0 && <ol className="timeline" aria-label="Görev geçmişi">
      {eco.runs.map(run => <li key={run.run_id} className={`tl-item tone-${statusTone(run.status)}`}>
        <span className="t">{run.request?.task ?? run.run_id}</span><span className={`badge tone-${statusTone(run.status)}`}>{statusLabel(run.status)}</span>
        <span className="m"><span>{run.request?.worker ?? 'Hub'}</span>{run.phase && <span>Aşama: {run.phase}</span>}{run.reason && <span>{run.reason}</span>}<span className="mono">{run.run_id}</span></span></li>)}
    </ol>}
  </div>;
}
