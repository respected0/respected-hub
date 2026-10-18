import { useCallback, useEffect, useRef, useState } from 'react';
import { loadSettings, native } from '../api.ts';
import { campaignStatus, listCampaigns, startCampaign, summarizeCampaign } from '../campaigns.ts';
import type { CampaignPhase, CampaignStatusResponse, CampaignSummary } from '../campaigns.ts';

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
      <div><h2>{id}</h2><p className="muted" role="status" aria-live="polite">{summary.headline}</p></div>
      <span className={`badge-status ${phaseKind(summary.phase)}`}>{phaseLabel(summary.phase)}</span>
    </div>
    <p className="campaign-supervisor">
      <span className={`dot ${status.supervisor_alive ? 'ok' : 'warn'}`} aria-hidden="true" />
      {status.supervisor_alive ? 'Süpervizör çalışıyor' : 'Süpervizör çalışmıyor'}
      {typeof status.status === 'string' ? ` · motor durumu: ${status.status}` : ''}
    </p>

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
  const [names, setNames] = useState<string[]>([]);
  const [dataRoot, setDataRoot] = useState('');
  const [listState, setListState] = useState<'loading' | 'ready' | 'failed'>('loading');
  const [listError, setListError] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const [status, setStatus] = useState<CampaignStatusResponse | null>(null);
  const [statusState, setStatusState] = useState<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  const [statusError, setStatusError] = useState('');
  const [spec, setSpec] = useState('');
  const [startBusy, setStartBusy] = useState(false);
  const [startMessage, setStartMessage] = useState('');
  const [startError, setStartError] = useState('');
  const selectedRef = useRef<string | null>(null);
  selectedRef.current = selected;

  const loadList = useCallback(async () => {
    setListState('loading'); setListError('');
    try {
      setNames(await listCampaigns());
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

  useEffect(() => { if (native) void loadList(); }, [loadList]);

  // The campaign folder is the user's configured Hub data root; shown read-only here.
  useEffect(() => {
    if (!native) return;
    void loadSettings().then(settings => setDataRoot(settings.data_root)).catch(() => setDataRoot(''));
  }, []);

  // Only a running campaign is polled, and only while this view stays mounted.
  useEffect(() => {
    if (!native || !selected || !running) return;
    const handle = window.setInterval(() => { void loadStatus(selected, true); }, 5000);
    return () => window.clearInterval(handle);
  }, [selected, running, loadStatus]);

  async function submitSpec() {
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

  if (!native) {
    return <div className="notice warning">Kampanyaları izlemek ve başlatmak için Tauri masaüstü uygulaması gerekli. Tarayıcı önizlemesinde gerçek kampanya verisi gösterilmez.</div>;
  }

  return <section className="campaigns-view" aria-label="Kampanyalar">
    <div className="section-head">
      <div>
        <span className="eyebrow">Kampanyalar</span>
        <h1>Kampanya takibi</h1>
        <p className="muted">Şeritlerin durumu, olay akışı ve süpervizör canlılığı doğrudan motordan okunur.</p>
        <p className="muted" role="note">Veri klasörü: <code className="campaign-id">{dataRoot || 'ayarlı değil'}</code> — Ayarlar bölümünden değiştirilir.</p>
      </div>
      <button type="button" onClick={() => { void loadList(); if (selected) void loadStatus(selected); }} disabled={listState === 'loading'}>
        {listState === 'loading' ? 'Yükleniyor…' : 'Yenile'}
      </button>
    </div>

    <div className="campaign-grid">
      <div className="campaign-list-col" aria-label="Kampanya listesi">
        {listState === 'loading' && <p className="muted">Kampanyalar yükleniyor…</p>}
        {listState === 'failed' && <p className="error" role="alert">{listError}</p>}
        {listState === 'ready' && names.length === 0 && <p className="empty-note">Henüz kampanya yok. Seçili veri klasöründe kampanya bulunamadı; klasörü Ayarlar bölümünden değiştirebilirsiniz. Aşağıdaki gelişmiş bölümden bir kampanya spec'i gönderebilirsiniz.</p>}
        <ul className="campaign-list">
          {names.map(name => <li key={name}>
            <button type="button" className={`campaign-row ${selected === name ? 'active' : ''}`} aria-pressed={selected === name} onClick={() => select(name)}>
              <span className="campaign-id">{name}</span>
            </button>
          </li>)}
        </ul>
      </div>

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
      {startMessage && <p className="message" role="status">{startMessage}</p>}
      {startError && <p className="error" role="alert">{startError}</p>}
    </details>
  </section>;
}