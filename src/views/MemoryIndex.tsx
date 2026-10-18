import { useEffect, useState } from 'react';
import { describeIndexError, describeIndexStatus, describeRefreshResult, memoryIndexRefresh, memoryIndexStatus } from '../memoryIndex.ts';
import type { IndexStatus } from '../memoryIndex.ts';
import { describeSemantic, semanticDisable, semanticEnable, semanticStatus } from '../memorySemantic.ts';
import type { SemanticStatus } from '../memorySemantic.ts';
import { memoryCaptureRun } from '../memoryIndex.ts';

// Shows SecondBrain's search index state and refreshes it after explicit confirmation.
export default function MemoryIndex() {
  const [status, setStatus] = useState<IndexStatus | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  const [captureBusy, setCaptureBusy] = useState(false);
  const [captureAsk, setCaptureAsk] = useState(false);
  async function capture() {
    setCaptureAsk(false); setCaptureBusy(true); setError(''); setMessage('');
    const reply = await memoryCaptureRun();
    if (reply.ok) setMessage(`${reply.result.captured_sessions} oturum hafızaya yakalandı.`); else setError(describeIndexError(reply.error));
    setCaptureBusy(false);
  }
  const [sem, setSem] = useState<SemanticStatus | null>(null);
  const [semConfirm, setSemConfirm] = useState(false);
  const [semBusy, setSemBusy] = useState(false);
  async function reloadSem() { const reply = await semanticStatus(); if (reply.ok) setSem(reply.result); }
  async function runSem(enable: boolean) {
    setSemBusy(true); setError(''); setSemConfirm(false);
    const reply = await (enable ? semanticEnable() : semanticDisable());
    await reloadSem();
    if (!reply.ok) setError(describeIndexError(reply.error));
    setSemBusy(false);
  }

  async function reload() {
    const reply = await memoryIndexStatus();
    if (reply.ok) setStatus(reply.result); else setError(describeIndexError(reply.error));
  }
  useEffect(() => { void reload(); void reloadSem(); }, []);

  async function run() {
    setBusy(true); setError(''); setMessage(''); setConfirming(false);
    const reply = await memoryIndexRefresh(false);
    await reload();  // reload must not wipe the action's outcome, so the outcome is set last
    if (reply.ok) setMessage(describeRefreshResult(reply.result)); else setError(describeIndexError(reply.error));
    setBusy(false);
  }

  const info = status ? describeIndexStatus(status) : null;
  const blocked = !info || info.state === 'unsupported' || info.state === 'no-vault';
  return <section className="panel-glass" aria-label="SecondBrain hafıza dizini">
    <div className="section-head"><h2>SecondBrain hafıza dizini</h2>{info && <span className={`badge-status ${info.state === 'fresh' ? 'ok' : info.state === 'stale' || info.state === 'invalid' ? 'warn' : 'unreachable'}`}>{info.title}</span>}</div>
    {info && <p className="muted" role="status">{info.detail}</p>}
    {status?.last_refresh && <p className="muted">Son yenileme: {new Date(status.last_refresh.at).toLocaleString()}</p>}
    {message && <p role="status">{message}</p>}
    {error && <p className="error" role="alert">{error}</p>}
    {confirming && <div role="note">
      <p>SecondBrain kasa notlarını okuyup yalnızca kendi arama dizinini günceller; kasa değiştirilmez. Büyük kasalarda bu işlem dakikalar sürebilir.</p>
    </div>}
    <div className="actions">
      {!confirming && <button type="button" className="cta" disabled={busy || blocked} onClick={() => setConfirming(true)}>{busy ? 'Yenileniyor…' : 'Dizini yenile'}</button>}
      {confirming && <>
        <button type="button" className="cta" disabled={busy || blocked} onClick={() => void run()}>Onayla ve yenile</button>
        <button type="button" disabled={busy} onClick={() => setConfirming(false)}>Vazgeç</button></>}
    </div>
    <div className="actions"><button type="button" disabled={captureBusy || blocked} onClick={() => captureAsk ? void capture() : setCaptureAsk(true)} title="Bitmiş ama henüz yakalanmamış AI oturumlarını kasaya özetler">{captureBusy ? 'Yakalanıyor… (dakikalar sürebilir)' : captureAsk ? 'Onayla: oturumlar AI sağlayıcınıza gönderilip özetlenecek' : 'Bekleyen oturumları şimdi yakala'}</button>{captureAsk && !captureBusy && <button type="button" onClick={() => setCaptureAsk(false)}>Vazgeç</button>}</div>
    {sem && sem.state !== 'unsupported' && sem.state !== 'no-vault' && (() => { const d = describeSemantic(sem); return <div className="semantic-block" aria-label="Anlamsal arama (isteğe bağlı)">
      <div className="section-head"><h3>Anlamsal arama (isteğe bağlı)</h3><span className={`badge-status ${d.tone}`}>{d.title}</span></div>
      <p className="muted" role="status">{d.detail}</p>
      {semConfirm && <div role="note"><p>Hub, yaklaşık 460 MB’lık yerel bir çalışma zamanı (llama.cpp) ve çok dilli (Türkçe dahil) bir model (bge-m3) internetten indirecek; başka bir program kurmanız gerekmez. Notlarınız bilgisayarınızdan çıkmaz. Büyük kasalarda ilk hazırlık dakikalar sürebilir.</p></div>}
      <div className="actions">
        {sem.state === 'off' && !semConfirm && <button type="button" className="cta" disabled={semBusy} onClick={() => setSemConfirm(true)}>Etkinleştir</button>}
        {semConfirm && <><button type="button" className="cta" disabled={semBusy} onClick={() => void runSem(true)}>Onayla, indir ve etkinleştir</button><button type="button" disabled={semBusy} onClick={() => setSemConfirm(false)}>Vazgeç</button></>}
        {(sem.state === 'provider_missing' || sem.state === 'model_missing' || sem.state === 'building') && <button type="button" className="cta" disabled={semBusy} onClick={() => void runSem(true)}>{semBusy ? 'Çalışıyor…' : 'Yeniden dene'}</button>}
        {sem.state !== 'off' && <button type="button" disabled={semBusy} onClick={() => void runSem(false)}>Kapat</button>}
      </div>
    </div>; })()}
  </section>;
}
