import { useEffect, useState } from 'react';
import { describeMemoryError, describeMemoryStatus, memoryAccessDisable, memoryAccessEnable, memoryAccessStatus } from '../memoryAccess.ts';
import type { MemoryStatus } from '../memoryAccess.ts';

// Enable/disable SecondBrain memory access. The grant token never reaches this component.
export default function MemoryAccess() {
  const [status, setStatus] = useState<MemoryStatus | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  const [write, setWrite] = useState(false);
  const [error, setError] = useState('');

  async function refresh() {
    const reply = await memoryAccessStatus();
    if (reply.ok) setStatus(reply.result); else setError(describeMemoryError(reply.error));
  }
  useEffect(() => { void refresh(); }, []);

  async function run(action: typeof memoryAccessEnable | typeof memoryAccessDisable) {
    setBusy(true); setError(''); setConfirming(false);
    const reply = await (action === memoryAccessEnable ? memoryAccessEnable(write) : memoryAccessDisable());
    await refresh();  // refresh must not wipe the action's error, so the action's error is set last
    if (!reply.ok) setError(describeMemoryError(reply.error));
    setBusy(false);
  }

  const info = status ? describeMemoryStatus(status) : null;
  return <section className="panel-glass" aria-label="SecondBrain hafıza erişimi">
    <div className="section-head"><h2>SecondBrain hafıza erişimi</h2>{info && <span className={`badge-status ${info.state === 'on' ? 'ok' : info.state === 'pending-revoke' ? 'warn' : 'unreachable'}`}>{info.title}</span>}</div>
    {info && <p className="muted" role="status">{info.detail}</p>}
    {status?.pending_revoke && <p className="error" role="alert">Erişim iptali SecondBrain'e ulaşmadı; izin hâlâ geçerli olabilir.</p>}
    {error && <p className="error" role="alert">{error}</p>}
    {confirming && <div role="note">
      <p><strong>Paylaşılacaklar</strong></p>
      <ul>
        <li>Yalnızca hafızayı kullanan görevlerde, görev başına en fazla 5 kısa not özeti okunur.</li>
        <li>{write ? 'Yazma açık: yalnızca kasadaki “Inbox/Orchestrator” klasörüne yeni not eklenir; mevcut notlarınıza dokunulmaz, notlar geri alınabilir.' : 'Erişim salt okunurdur; SecondBrain hiçbir şey yazmaz.'}</li>
        <li>İzni istediğiniz zaman iptal edebilirsiniz.</li>
      </ul>
      {status?.write_supported !== false && <label className="choice"><span><strong>Notları SecondBrain’e yazmasına da izin ver</strong><small>İsteğe bağlı; Orchestrator yalnızca siz istediğinizde yeniden kullanılabilir bir not ekleyebilir.</small></span>
        <input type="checkbox" checked={write} disabled={busy} onChange={event => setWrite(event.target.checked)} /></label>}
    </div>}
    <div className="actions">
      {status?.pending_revoke && <button type="button" className="cta" disabled={busy} onClick={() => void run(memoryAccessDisable)}>{busy ? 'Deneniyor…' : 'İptali yeniden dene'}</button>}
      {status && !status.pending_revoke && status.supported && !status.enabled && !confirming && <button type="button" className="cta" disabled={busy} onClick={() => setConfirming(true)}>Erişimi aç</button>}
      {confirming && <>
        <button type="button" className="cta" disabled={busy} onClick={() => void run(memoryAccessEnable)}>{busy ? 'Açılıyor…' : 'Onayla ve aç'}</button>
        <button type="button" disabled={busy} onClick={() => setConfirming(false)}>Vazgeç</button></>}
      {status && !status.pending_revoke && status.enabled && <button type="button" disabled={busy} onClick={() => void run(memoryAccessDisable)}>{busy ? 'Kapatılıyor…' : 'Erişimi kapat ve iptal et'}</button>}
    </div>
  </section>;
}
