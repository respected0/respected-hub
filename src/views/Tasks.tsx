import { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { buildRequest, terminal, type TaskForm } from '../tasks';
import { engine, native, products, type Products as ProductCatalog, type Run } from '../api';

const initial: TaskForm = { projectRoot: '', task: '', worker: 'codex', kind: 'write', ownership: '', forbidden: '.env\ncredentials',
  acceptance: '[["python", "-m", "unittest", "discover", "-s", "tests"]]', customCommand: '["your-cli", "{prompt}"]', timeout: 900 };

// Task creation and run inspection. Moved verbatim from the former single-page App; engine owns task state.
export default function Tasks() {
  const [form, setForm] = useState(initial);
  const [catalog, setCatalog] = useState<ProductCatalog | null>(null);
  const [runs, setRuns] = useState<Run[]>([]);
  const [selected, setSelected] = useState<Run | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const field = (key: keyof TaskForm, value: string | number) => setForm(current => ({ ...current, [key]: value }));
  async function refresh() {
    const [inventory, tasks] = await Promise.all([products(), engine<{ runs: Run[] }>('list')]);
    setCatalog(inventory); setRuns(tasks.runs);
  }

  useEffect(() => {
    if (!native) return;
    let active = true;
    const poll = async () => {
      try {
        const tasks = await engine<{ runs: Run[] }>('list');
        if (active) setRuns(tasks.runs);
        if (selectedId) {
          const detail = await engine<Run>('inspect', selectedId);
          if (active) setSelected(detail);
        }
      } catch (reason) { if (active) setError(String(reason)); }
    };
    products().then(value => { if (active) setCatalog(value); }).catch(reason => { if (active) setError(String(reason)); });
    void poll();
    const timer = window.setInterval(poll, 1500);
    return () => { active = false; window.clearInterval(timer); };
  }, [selectedId]);

  async function action(work: () => Promise<void>) {
    setBusy(true); setError(''); setMessage('');
    try { await work(); await refresh(); }
    catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  }

  async function submit() {
    const created = await engine<Run>('create', undefined, buildRequest(form));
    setSelectedId(created.run_id);
    await engine('start', created.run_id);
    setMessage('Görev gönderildi. Durumu motorun doğrulama sonucu belirleyecek.');
  }

  return <>
    {catalog?.engine.error && <p role="alert" className="error">{catalog.engine.error}</p>}
    {error && <p role="alert" className="error">{error}</p>}
    {message && <p role="status" className="message">{message}</p>}
      <div className="workspace">
        <section className="panel"><h2>Yeni görev</h2><form onSubmit={event => { event.preventDefault(); void action(submit); }}>
          <label>Git projesi<div className="input-row"><input value={form.projectRoot} onChange={e => field('projectRoot', e.target.value)} placeholder="Mutlak proje yolu" required /><button type="button" disabled={!native || busy} onClick={() => void action(async () => { const directory = await open({ directory: true, multiple: false }); if (typeof directory === 'string') field('projectRoot', directory); })}>Seç</button></div></label>
          <label>Görev<textarea value={form.task} onChange={e => field('task', e.target.value)} placeholder="Hangi sonuca ulaşılmalı?" required rows={3} /></label>
          <div className="columns"><label>Worker<select value={form.worker} onChange={e => field('worker', e.target.value)}>{['codex', 'claude', 'gemini', 'antigravity', 'cursor', 'custom'].map(worker => <option key={worker} value={worker}>{worker}{catalog?.engine.workers && worker !== 'custom' && !catalog.engine.workers[worker] ? ' · bulunamadı' : ''}</option>)}</select></label>
          <label>İş türü<select value={form.kind} onChange={e => field('kind', e.target.value)}><option value="write">Yazma + doğrulama</option><option value="read">Salt okuma</option></select></label></div>
          <div className="columns"><label>İzin verilen yollar<textarea value={form.ownership} onChange={e => field('ownership', e.target.value)} placeholder={'src/module.py\ntests'} required rows={3} /><small>Proje köküne göre bir yol / satır.</small></label>
          <label>Yasak yollar<textarea value={form.forbidden} onChange={e => field('forbidden', e.target.value)} rows={3} /></label></div>
          {form.worker === 'custom' && <label>Özel worker komutu<textarea value={form.customCommand} onChange={e => field('customCommand', e.target.value)} /><small>JSON argv listesi. {'{prompt}'} görev brief'ini alır.</small></label>}
          <label>Doğrulama komutları<textarea value={form.acceptance} onChange={e => field('acceptance', e.target.value)} rows={2} /><small>JSON argv listelerinin listesi; yazma görevi için zorunlu.</small></label>
          <label>Süre sınırı (saniye)<input type="number" min="1" max="3600" value={form.timeout} onChange={e => field('timeout', Number(e.target.value))} /></label>
          <p className="muted">Worker ayrı worktree'de çalışır; ana projeye otomatik apply yapılmaz. CLI'nin kendi izinleri geçerlidir.</p>
          <button className="primary" type="submit" disabled={!native || busy}>{busy ? 'İşleniyor…' : 'Görevi çalıştır'}</button>
        </form></section>
        <section className="panel"><h2>Görevler <span className="count">{runs.length}</span></h2>
          {!runs.length && <p className="empty">Henüz bir görev yok. İlk sonucu burada takip edebilirsin.</p>}
          <div className="run-list">{runs.map(run => <button className={`run ${selectedId === run.run_id ? 'selected' : ''}`} key={run.run_id} onClick={() => { setSelected(null); setSelectedId(run.run_id); }}><span>{run.request?.task ?? run.run_id}</span><small>{run.request?.worker} · <span className={`status ${run.status}`}>{run.status}</span></small></button>)}</div>
          {selected && <div className="detail"><h3>{selected.request?.task ?? selected.run_id}</h3><p><span className={`status ${selected.status}`}>{selected.status}</span> {selected.phase ?? selected.reason ?? ''}</p><small>{selected.run_id}</small>
            {selected.error && <p className="error">{selected.error}</p>}
            <div className="actions">
              {!terminal(selected.status) && <button disabled={busy} onClick={() => void action(async () => { await engine('cancel', selected.run_id); setMessage('İptal istendi; motorun terminal durumunu bekliyoruz.'); })}>Durdur</button>}
              {selected.status === 'queued' && <button disabled={busy} onClick={() => void action(async () => { await engine('start', selected.run_id); })}>Başlat</button>}
              {selected.status === 'completed' && <button disabled={busy} onClick={() => void action(async () => { await engine('check', selected.run_id); setMessage('Sonuç bütünlüğü ve güncel projeyle uyumu doğrulandı. Apply yapılmadı.'); })}>Sonucu doğrula</button>}
              {['failed', 'timed_out', 'cancelled', 'interrupted'].includes(selected.status) && <button disabled={busy} onClick={() => void action(async () => { const task = await engine<Run>('retry', selected.run_id); setSelectedId(task.run_id); await engine('start', task.run_id); })}>Yeniden dene</button>}
            </div>
            <h3>Sonuç / patch</h3><pre>{selected.patch_preview || 'Patch henüz yok.'}</pre><h3>Loglar</h3>{Object.entries(selected.logs ?? {}).map(([name, log]) => <details key={name}><summary>{name}</summary><pre>{log || 'Boş log'}</pre></details>)}
          </div>}
        </section>
      </div>
  </>;
}
