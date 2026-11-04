import { useCallback, useEffect, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { onReturn } from '../../../shared/focus.ts';
import { native } from '../../../shared/api.ts';
import { buildCampaignSpec, campaignNames, campaignStatus, emptyCampaignForm, peekCampaigns, startCampaign, summarizeCampaign } from '../campaigns.ts';
import './campaigns.css';
import type { CampaignForm, CampaignListItem, CampaignPhase, CampaignStatusResponse, CampaignSummary } from '../campaigns.ts';

function errorText(reason: unknown): string {
  return reason instanceof Error ? reason.message : String(reason);
}

function formatAt(at: string): string {
  if (!at) return '—';
  const time = Date.parse(at);
  return Number.isNaN(time) ? at : new Date(time).toLocaleString('tr-TR');
}

function phaseLabel(phase: CampaignPhase): string {
  switch (phase) {
    case 'running': return 'Çalışıyor';
    case 'finished': return 'Tamamlandı';
    case 'needs_review': return 'İnceleme bekliyor';
    case 'blocked': return 'Engellendi';
    case 'interrupted': return 'Yarıda kesildi';
  }
}

function phaseKind(phase: CampaignPhase): 'ok' | 'warn' | 'error' {
  if (phase === 'finished') return 'ok';
  if (phase === 'running' || phase === 'needs_review') return 'warn';
  return 'error';
}

function CampaignDetail({ id, status, summary }: { id: string; status: CampaignStatusResponse; summary: CampaignSummary }) {
  return <article className="panel-glass campaign-panel">
    <div className="section-head">
      <div><h2>{summary.goal || id}</h2><p className="muted" role="status" aria-live="polite"><code>{id}</code> · {summary.headline}</p></div>
      <span className={`badge-status ${phaseKind(summary.phase)}`}>{phaseLabel(summary.phase)}</span>
    </div>
    <p className="campaign-supervisor">
      <span className={`dot ${status.supervisor_alive ? 'ok' : 'warn'}`} aria-hidden="true" />
      {status.supervisor_alive ? 'Süpervizör çalışıyor' : 'Süpervizör çalışmıyor'}
      {typeof status.status === 'string' ? ` · motor durumu: ${status.status}` : ''}
    </p>

    <p className="campaign-goal">{summary.goal ?? 'Kampanya hedefi kayıtlı değil.'}</p>
    <div className="campaign-facts">
      <span><strong>Oluşturuldu</strong> {summary.createdAt ? formatAt(summary.createdAt) : '—'}</span>
      <span><strong>Toplam token</strong> {summary.totalTokens === null ? 'bildirilmedi' : summary.totalTokens.toLocaleString('tr-TR')}</span>
      <span><strong>Son inceleme</strong> {summary.finalReview ?? 'henüz yok'}</span>
    </div>

    <section aria-label="İş akışları">
      <h3>İş akışları</h3>
      {summary.workstreams.length === 0
        ? <p className="muted">Bu kampanya için iş akışı tanımı kayıtlı değil.</p>
        : <ul className="campaign-workstreams">{summary.workstreams.map((stream, index) => <li key={`${stream.name}-${index}`}>
            <strong>{stream.name}</strong>
            {stream.after.length > 0 && <span className="muted">önce: {stream.after.join(', ')}</span>}
            <pre className="task-text">{stream.task || 'Görev metni kayıtlı değil.'}</pre>
          </li>)}</ul>}
    </section>

    <section aria-label="Şeritler">
      <h3>Şeritler</h3>
      {summary.lanes.length === 0
        ? <p className="muted">Bu kampanya için şerit bilgisi yok.</p>
        : <ul className="campaign-lanes">{summary.lanes.map((lane, index) => <li key={`${lane.name}-${index}`}>
            <strong>{lane.name}</strong>
            <span className="lane-status">{lane.statusText}</span>
            {lane.detail && <span className="muted">{lane.detail}</span>}
            {lane.tokens !== null && <span className="muted">{lane.tokens} token</span>}
          </li>)}</ul>}
    </section>

    <section aria-label="Olay akışı">
      <h3>Olay akışı</h3>
      {summary.timeline.length === 0
        ? <p className="muted">Henüz olay kaydı yok.</p>
        : <ol className="campaign-timeline">{summary.timeline.map((item, index) => <li key={index}>
            <time>{formatAt(item.at)}</time><span>{item.text}</span>
          </li>)}</ol>}
    </section>

    {summary.chain && <section className="chain-note" aria-label="Devam zinciri">
      <h3>Devam zinciri</h3>
      <p>Bu kampanya <strong>{summary.chain.parent}</strong> kampanyasının devamı (sıra {summary.chain.index}). Sonuç: {summary.chain.outcome}.</p>
    </section>}

    {summary.finalText && <section aria-label="Son inceleme">
      <h3>Son inceleme notu</h3>
      <pre className="campaign-final">{summary.finalText}</pre>
    </section>}
  </article>;
}

export default function Campaigns() {
  const [names, setNames] = useState<CampaignListItem[]>(() => peekCampaigns() ?? []);
  const [listState, setListState] = useState<'loading' | 'ready' | 'failed'>(() => peekCampaigns() ? 'ready' : 'loading');
  const [listError, setListError] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const [status, setStatus] = useState<CampaignStatusResponse | null>(null);
  const [statusState, setStatusState] = useState<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  const [statusError, setStatusError] = useState('');
  const [spec, setSpec] = useState('');
  const [form, setForm] = useState<CampaignForm>(emptyCampaignForm);
  const [startBusy, setStartBusy] = useState(false);
  const [startSource, setStartSource] = useState<'form' | 'advanced'>('form');
  const [startMessage, setStartMessage] = useState('');
  const [startError, setStartError] = useState('');
  const selectedRef = useRef<string | null>(null);
  selectedRef.current = selected;

  const loadList = useCallback(async (quiet = false) => {
    if (!quiet && !peekCampaigns()) setListState('loading');
    setListError('');
    try {
      setNames(await campaignNames());
      setListState('ready');
    } catch (reason) {
      setListState('failed'); setListError(errorText(reason));
    }
  }, []);

  const loadStatus = useCallback(async (id: string, quiet = false) => {
    if (!quiet) { setStatusState('loading'); setStatusError(''); }
    try {
      const reply = await campaignStatus(id);
      if (selectedRef.current !== id) return;
      setStatus(reply); setStatusState('ready'); setStatusError('');
    } catch (reason) {
      if (selectedRef.current !== id) return;
      setStatusState('failed'); setStatusError(errorText(reason));
    }
  }, []);

  const select = useCallback((id: string) => {
    setSelected(id); setStatus(null); setStatusState('idle'); setStatusError('');
    void loadStatus(id);
  }, [loadStatus]);

  const summary = status ? summarizeCampaign(status) : null;
  const running = status !== null && (status.supervisor_alive === true || summarizeCampaign(status).phase === 'running');

  useEffect(() => { if (!native) return; void loadList(); return onReturn(() => void loadList(true)); }, [loadList]);

  // Only a running campaign is polled, and only while this view stays mounted.
  useEffect(() => {
    if (!native || !selected || !running) return;
    const handle = window.setInterval(() => { void loadStatus(selected, true); }, 5000);
    return () => window.clearInterval(handle);
  }, [selected, running, loadStatus]);

  async function submitSpec() {
    setStartSource('advanced');
    setStartBusy(true); setStartMessage(''); setStartError('');
    try {
      const result = await startCampaign(spec);
      setStartMessage(`Kampanya başlatıldı: ${result.campaign_id}`);
      setSpec('');
      await loadList();
      select(result.campaign_id);
    } catch (reason) {
      setStartError(errorText(reason));
    } finally {
      setStartBusy(false);
    }
  }

  function field<K extends keyof CampaignForm>(key: K, value: CampaignForm[K]) {
    setForm(current => ({ ...current, [key]: value }));
  }

  function stream(index: number, key: 'name' | 'task' | 'after', value: string) {
    setForm(current => ({ ...current, streams: current.streams.map((item, at) => at === index ? { ...item, [key]: value } : item) }));
  }

  function addStream() {
    setForm(current => ({ ...current, streams: [...current.streams, { name: `is-${current.streams.length + 1}`, task: '', after: '' }] }));
  }

  async function submitForm() {
    setStartSource('form');
    setStartBusy(true); setStartMessage(''); setStartError('');
    try {
      const result = await startCampaign(JSON.stringify(buildCampaignSpec(form)));
      setStartMessage(`Kampanya başlatıldı: ${result.campaign_id}`);
      setForm(emptyCampaignForm());
      await loadList();
      select(result.campaign_id);
    } catch (reason) {
      setStartError(errorText(reason));
    } finally {
      setStartBusy(false);
    }
  }

  if (!native) {
    return <div className="notice warning">Kampanyaları izlemek ve başlatmak için Tauri masaüstü uygulaması gerekli. Tarayıcı önizlemesinde gerçek kampanya verisi gösterilmez.</div>;
  }

  return <div className="campaigns-view ws-stack" aria-label="Kampanyalar">
    <section className="panel-glass campaign-new" aria-label="Yeni kampanya">
      <div className="section-head"><div><h2>Yeni kampanya</h2><p className="muted">Aynı proje üzerinde paralel iş akışları. Başlatmadan hiçbir işçi çalışmaz.</p></div></div>
      <form onSubmit={event => { event.preventDefault(); void submitForm(); }}>
        <label>Hedef<textarea rows={2} value={form.goal} onChange={event => field('goal', event.target.value)} placeholder="Bu kampanya hangi sonucu üretecek?" required /></label>
        <label>Proje kökü<div className="input-row"><input value={form.projectRoot} onChange={event => field('projectRoot', event.target.value)} placeholder="Mutlak proje yolu" required /><button type="button" disabled={startBusy} onClick={() => void open({ directory: true, multiple: false }).then(directory => { if (typeof directory === 'string') field('projectRoot', directory); })}>Seç</button></div></label>
        <div className="columns">
          <label>Worker<select value={form.worker} onChange={event => field('worker', event.target.value)}>{['codex', 'claude', 'gemini', 'antigravity', 'cursor'].map(worker => <option key={worker} value={worker}>{worker}</option>)}</select></label>
          <label>İş türü<select value={form.kind} onChange={event => field('kind', event.target.value as CampaignForm['kind'])}><option value="write">Yazma + doğrulama</option><option value="read">Salt okuma</option></select></label>
        </div>
        <div className="columns">
          <label>İzin verilen yollar<textarea rows={2} value={form.ownership} onChange={event => field('ownership', event.target.value)} placeholder={'src\ntests'} required /><small>Proje köküne göre, her satır bir yol.</small></label>
          <label>Yasak yollar<textarea rows={2} value={form.forbidden} onChange={event => field('forbidden', event.target.value)} /></label>
        </div>
        <label>Doğrulama komutları<textarea rows={2} value={form.acceptance} onChange={event => field('acceptance', event.target.value)} placeholder="python -m unittest discover -s tests" /><small>Her satır bir komut; yazma görevinde zorunlu.</small></label>
        <div className="columns">
          <label>Süre sınırı (saniye)<input type="number" min="1" max="14400" value={form.timeout} onChange={event => field('timeout', Number(event.target.value))} /></label>
          <label className="campaign-check"><input type="checkbox" checked={form.finalSkip} onChange={event => field('finalSkip', event.target.checked)} /> Son incelemeyi atla (Claude gerekmez)</label>
        </div>
        <h3>İş akışları</h3>
        {form.streams.map((item, index) => <div className="campaign-stream" key={index}>
          <div className="columns">
            <label>Ad<input value={item.name} onChange={event => stream(index, 'name', event.target.value)} placeholder="main" required /></label>
            <label>Önce (virgülle)<input value={item.after} onChange={event => stream(index, 'after', event.target.value)} placeholder="main" /></label>
          </div>
          <label>Görev<textarea rows={3} value={item.task} onChange={event => stream(index, 'task', event.target.value)} placeholder="Bu iş akışı ne yapacak?" required /></label>
          {form.streams.length > 1 && <button type="button" onClick={() => setForm(current => ({ ...current, streams: current.streams.filter((_, at) => at !== index) }))}>İş akışını kaldır</button>}
        </div>)}
        <div className="campaign-actions">
          <button type="button" onClick={addStream} disabled={form.streams.length >= 8}>İş akışı ekle</button>
          <button type="submit" className="cta" disabled={startBusy}>{startBusy ? 'Gönderiliyor…' : 'Kampanyayı başlat'}</button>
        </div>
        {startSource === 'form' && startMessage && <p className="message" role="status">{startMessage}</p>}
        {startSource === 'form' && startError && <p className="error" role="alert">{startError}</p>}
      </form>
    </section>

    <div className="campaign-grid">
      <section className="panel-glass campaign-list-col" aria-label="Kampanya listesi">
        <div className="section-head"><div><h2>Kampanyalar</h2><p className="muted" role="note">Orchestrator'ın kendi veri klasöründeki kampanyalar.</p></div></div>
        {listState === 'loading' && <p className="muted">Kampanyalar yükleniyor…</p>}
        {listState === 'failed' && <p className="error" role="alert">{listError}</p>}
        {listState === 'ready' && names.length === 0 && <p className="empty-note">Henüz kampanya yok. Yukarıdaki formdan bir tane başlatın.</p>}
        <ul className="campaign-list">
          {names.map(item => <li key={item.id}>
            <button type="button" className={`campaign-row ${selected === item.id ? 'active' : ''}`} aria-pressed={selected === item.id} onClick={() => select(item.id)}>
              <span className="campaign-title">{item.goal || 'Hedef kayıtlı değil'}</span>
              {item.created_at && <span className="campaign-date">{formatAt(item.created_at)}</span>}
              <span className="campaign-id">{item.id}</span>
            </button>
          </li>)}
        </ul>
      </section>

      <div className="campaign-detail">
        {!selected && <p className="empty-note">Ayrıntıyı görmek için soldan bir kampanya seçin.</p>}
        {selected && statusState === 'loading' && <p className="muted">Durum yükleniyor…</p>}
        {selected && statusState === 'failed' && <p className="error" role="alert">{statusError}</p>}
        {selected && statusState === 'ready' && summary && <CampaignDetail id={selected} status={status as CampaignStatusResponse} summary={summary} />}
      </div>
    </div>

    <details className="campaign-advanced">
      <summary>Gelişmiş: kampanya spec'i gönder</summary>
      <p className="muted">Spec motora JSON olarak gönderilir. Düğmeye basmadan hiçbir şey başlamaz; onayladığınızda gerçek işçiler çalışır.</p>
      <label htmlFor="campaign-spec">Kampanya spec'i (JSON)</label>
      <textarea id="campaign-spec" rows={10} value={spec} spellCheck={false}
        onChange={event => setSpec(event.target.value)}
        placeholder={'{\n  "goal": "…",\n  "workstreams": [ … ]\n}'} />
      <div className="campaign-actions">
        <button type="button" className="cta" disabled={startBusy || !spec.trim()} onClick={() => void submitSpec()}>
          {startBusy ? 'Gönderiliyor…' : 'Kampanyayı başlat'}
        </button>
      </div>
      {startSource === 'advanced' && startMessage && <p className="message" role="status">{startMessage}</p>}
      {startSource === 'advanced' && startError && <p className="error" role="alert">{startError}</p>}
    </details>
  </div>;
}
