import { useEffect, useState } from 'react';
import { ACTION_DONE, ACTION_LABEL, STATE_LABEL, availableActions, describeLifecycleError, lifecycleApply, lifecyclePlan } from '../lifecycle.ts';
import type { JournalEntry, LifecycleAction, LifecyclePlan, ProductLifecycle } from '../lifecycle.ts';
import { Icon } from '../../shared/ui.tsx';

const ICON: Record<LifecycleAction, string> = { install: 'download', update: 'sync', repair: 'wrench', uninstall: 'trash' };

// Install / update / repair / remove one product by running its own installer or uninstaller: plan first, confirm, apply.
// `autoAction`: the person already chose this elsewhere (an update notice, a product installer), so its plan opens directly.
export default function LifecycleControls({ name, lc, onChanged, onWizard, autoAction, onAutoDone }: { name: string; lc: ProductLifecycle | null; onChanged: () => void; onWizard?: () => void; autoAction?: LifecycleAction; onAutoDone?: () => void }) {
  const [pending, setPending] = useState<LifecyclePlan | null>(null);
  const [purge, setPurge] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [done, setDone] = useState<JournalEntry | null>(null);
  const [note, setNote] = useState('');

  const actions = lc ? availableActions(lc.install_state, lc.available !== null) : [];
  useEffect(() => {
    if (!autoAction || !lc) return;
    onAutoDone?.();
    if (actions.includes(autoAction)) void preview(autoAction);
    else setNote(`${name}: bu işlem şu an yapılamaz (${STATE_LABEL[lc.install_state].toLowerCase()}).`);
  }, [autoAction, lc?.install_state]); // eslint-disable-line react-hooks/exhaustive-deps
  if (!lc) return <p className="muted">Kurulum durumu okunamadı.</p>;

  async function preview(action: LifecycleAction) {
    // Brain's install needs a vault and its first settings: the setup wizard collects them and runs the plan.
    if (action === 'install' && onWizard) { onWizard(); return; }
    setBusy(true); setError(''); setDone(null); setNote(''); setPurge(false);
    const reply = await lifecyclePlan(lc!.product, action);
    setBusy(false);
    if (reply.ok) setPending(reply.result); else setError(describeLifecycleError(reply.error));
  }

  async function confirm() {
    if (!pending) return;
    setBusy(true); setError('');
    const reply = await lifecycleApply(pending.product, pending.action, pending.action === 'uninstall' ? { purge } : undefined);
    setBusy(false); setPending(null);
    if (reply.ok) setDone(reply.result); else setError(describeLifecycleError(reply.error));
    onChanged();
  }

  return <div className="lifecycle" aria-label={`${name} kurulumu`}>
    <p className="meta"><span className={`badge-status ${lc.install_state === 'installed' ? 'ok' : lc.install_state === 'absent' ? 'unreachable' : 'warn'}`}>{STATE_LABEL[lc.install_state]}</span>
      {lc.version && <span className="ver">v{lc.version}</span>}
      {lc.available && lc.install_state !== 'installed' && <span className="ver">{lc.install_state === 'absent' ? 'Sürüm' : 'Yeni sürüm'}: v{lc.available.version}</span>}</p>
    {!lc.available && lc.install_state === 'absent' && <p className="muted">Ürün kataloğu henüz alınamadı{lc.catalog_skipped.length ? ` (${lc.catalog_skipped.length} geçersiz paket atlandı)` : ''}. İnternete bağlıyken Ayarlar › Güncellemeleri denetle'yi kullanın.</p>}
    {busy && !pending && <p className="muted" role="status">Hazırlanıyor… Gerekirse ürünün kurucusu indirilip doğrulanır.</p>}
    {!pending && <div className="actions">{actions.map(action => <button key={action} className={action === 'uninstall' ? '' : 'cta'} disabled={busy} onClick={() => void preview(action)}><Icon name={ICON[action]} />{ACTION_LABEL[action]}</button>)}</div>}
    {pending && <div className="panel-glass" role="group" aria-label="İşlem planı">
      <h3>{ACTION_LABEL[pending.action]}: {name}{pending.to_version && pending.action !== 'uninstall' ? ` → v${pending.to_version}` : ''}</h3>
      <ol className="plan-steps">{pending.plan.steps.map(step => <li key={step.id}>{step.summary}{!step.reversible && <em className="pill"> Geri alınamaz: {step.irreversible_reason}</em>}</li>)}</ol>
      {pending.action === 'uninstall' && <label className="choice"><span><strong>Ayarlar ve çalışma verileri de silinsin</strong><small>Kapalıysa yeniden kurduğunda kaldığın yerden devam edersin. Not kasan hiçbir durumda silinmez.</small></span>
        <input type="checkbox" role="switch" checked={purge} disabled={busy} onChange={event => setPurge(event.target.checked)} /></label>}
      <div className="actions"><button className="cta" disabled={busy} onClick={() => void confirm()}>{busy ? 'Uygulanıyor…' : 'Onayla ve uygula'}</button><button disabled={busy} onClick={() => setPending(null)}>Vazgeç</button></div></div>}
    {done && <div className="message" role="status">{name} {ACTION_DONE[done.action]}{done.to_version && done.action !== 'uninstall' ? ` (v${done.to_version})` : ''}.</div>}
    {note && <p className="message" role="status">{note}</p>}
    {error && <p className="error" role="alert">{error}</p>}
  </div>;
}
