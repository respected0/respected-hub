import { useState } from 'react';
import {
  ACTION_DONE, ACTION_LABEL, STATE_LABEL, availableActions, canRollback, describeLifecycleError, lifecycleApply, lifecyclePlan, lifecyclePurgeBackups, lifecycleRollback,
} from '../lifecycle.ts';
import type { JournalEntry, LifecycleAction, LifecyclePlan, ProductLifecycle } from '../lifecycle.ts';
import { Icon } from '../ui.tsx';

const ICON: Record<LifecycleAction, string> = { install: 'download', update: 'sync', repair: 'wrench', uninstall: 'trash', integrations: 'connections' };

// Install / update / repair / uninstall for one Hub-managed product: plan first, confirm, apply, then offer undo.
export default function LifecycleControls({ name, lc, onChanged, onWizard }: { name: string; lc: ProductLifecycle | null; onChanged: () => void; onWizard?: () => void }) {
  const [pending, setPending] = useState<LifecyclePlan | null>(null);
  const [busy, setBusy] = useState(false);
  const [purgeAsk, setPurgeAsk] = useState(false);
  const [error, setError] = useState('');
  const [done, setDone] = useState<JournalEntry | null>(null);
  const [note, setNote] = useState('');

  if (!lc) return <p className="muted">Yönetilen kurulum durumu okunamadı.</p>;
  const actions = availableActions(lc.install_state, lc.available !== null);

  async function preview(action: LifecycleAction) {
    // SecondBrain's install needs a vault and integration choices: the setup wizard collects them and runs the plan.
    if (action === 'install' && onWizard) { onWizard(); return; }
    setBusy(true); setError(''); setDone(null); setNote('');
    const reply = await lifecyclePlan(lc!.product, action);
    setBusy(false);
    if (reply.ok) setPending(reply.result); else setError(describeLifecycleError(reply.error));
  }

  async function confirm() {
    if (!pending) return;
    setBusy(true); setError('');
    const reply = await lifecycleApply(pending.product, pending.action);
    setBusy(false); setPending(null);
    if (reply.ok) setDone(reply.result); else setError(describeLifecycleError(reply.error));
    onChanged();
  }

  async function undo() {
    if (!done) return;
    setBusy(true); setError('');
    const reply = await lifecycleRollback(done.operation_id);
    setBusy(false);
    if (reply.ok) { setDone(null); setNote('İşlem geri alındı.'); } else setError(describeLifecycleError(reply.error));
    onChanged();
  }

  async function purge() {
    setPurgeAsk(false); setBusy(true); setError('');
    const reply = await lifecyclePurgeBackups(lc!.product);
    setBusy(false);
    if (reply.ok) setNote(`${reply.result.removed_entries} yedek kalıcı olarak silindi.`); else setError(describeLifecycleError(reply.error));
    onChanged();
  }

  return <div className="lifecycle" aria-label={`${name} yaşam döngüsü`}>
    <p className="meta"><span className={`badge-status ${lc.install_state === 'installed' ? 'ok' : lc.install_state === 'absent' ? 'unreachable' : 'warn'}`}>{STATE_LABEL[lc.install_state]}</span>
      {lc.version && <span className="ver">v{lc.version}</span>}
      {lc.signed === false && <span className="pill" title="Yerel olarak üretilmiş, imzasız paket">İmzasız yerel paket</span>}
      {lc.available && lc.install_state !== 'installed' && <span className="ver">Kataloğta v{lc.available.version}</span>}</p>
    {lc.problems.length > 0 && <ul className="checks compact" aria-label="Bütünlük sorunları">{lc.problems.slice(0, 4).map(problem => <li key={problem} className="bad"><span aria-hidden="true">✕</span>{problem}</li>)}</ul>}
    {!lc.available && lc.install_state === 'absent' && <p className="muted">Katalogta kurulabilir paket yok{lc.catalog_skipped.length ? ` (${lc.catalog_skipped.length} geçersiz paket atlandı)` : ''}. Paket oluşturmak için <code>python tools/build_bundle.py</code> kullanın.</p>}
    {!pending && <div className="actions">{actions.map(action => <button key={action} className={action === 'install' || action === 'update' || action === 'repair' ? 'cta' : ''} disabled={busy} onClick={() => void preview(action)}><Icon name={ICON[action]} />{ACTION_LABEL[action]}</button>)}
      {!pending && lc.install_state === 'installed' && lc.available !== null && <button className="cta" disabled title="Dosyalar doğrulandı; onarım gerekmiyor. Bozulma olursa bu düğme etkinleşir."><Icon name={ICON.repair} />Onar</button>}
      {lc.removed_available && !purgeAsk && <button disabled={busy} onClick={() => setPurgeAsk(true)} title="Kaldırma ve güncelleme yedeklerini kalıcı siler">Yedekleri temizle</button>}
      {lc.removed_available && purgeAsk && <><button className="cta" disabled={busy} onClick={() => void purge()}>Yedekleri kalıcı olarak sil (geri alınamaz)</button><button disabled={busy} onClick={() => setPurgeAsk(false)}>Vazgeç</button></>}</div>}
    {pending && <div className="panel-glass" role="group" aria-label="İşlem planı">
      <h3>{ACTION_LABEL[pending.action]}: {name}{pending.to_version ? ` → v${pending.to_version}` : ''}</h3>
      <ol className="plan-steps">{pending.plan.steps.map(step => <li key={step.id}>{step.summary}{!step.reversible && <em className="pill"> Geri alınamaz: {step.irreversible_reason}</em>}</li>)}</ol>
      {pending.action === 'uninstall' && <p className="muted">Program, ürüne ait tüm veri ve yedekler ile AI araçlarındaki Respected kayıtları (kanca / MCP / zamanlanmış görev / kısayol) kaldırılır. Yalnızca not kasanız ve notlarınız kalır. Bu kaldırma geri alınamaz; yeniden kurulabilir.</p>}
      <div className="actions"><button className="cta" disabled={busy} onClick={() => void confirm()}>{busy ? 'Uygulanıyor…' : 'Onayla ve uygula'}</button><button disabled={busy} onClick={() => setPending(null)}>Vazgeç</button></div></div>}
    {done && <div className="message" role="status">{name} {ACTION_DONE[done.action]}{done.to_version && done.action !== 'uninstall' ? ` (v${done.to_version})` : ''}. {canRollback(done) && <button disabled={busy} onClick={() => void undo()}>Geri al</button>}</div>}
    {note && <p className="message" role="status">{note}</p>}
    {error && <p className="error" role="alert">{error}</p>}
  </div>;
}
