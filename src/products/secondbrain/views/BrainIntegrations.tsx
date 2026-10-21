import { useEffect, useRef, useState } from 'react';
import { describeLifecycleError, lifecycleApply, lifecyclePlan } from '../../../shell/lifecycle.ts';
import type { LifecyclePlan } from '../../../shell/lifecycle.ts';
import { fromCurrent, toDesired } from '../brainClients.ts';
import type { Selection } from '../brainClients.ts';
import ClientPicker from './ClientPicker.tsx';
import IntegrationPreview from './IntegrationPreview.tsx';

// Installed SecondBrain: choose, per AI client, where memory is captured (Sources) and who may use it over MCP (Permissions),
// with an explicit, read-only preview first. `initial` are the stored on/off flags; the exact client lists come from the product.
export default function BrainIntegrations({ initial, onApplied }: { initial: Record<string, boolean>; onApplied: () => void }) {
  const [selection, setSelection] = useState<Selection>(() => fromCurrent(initial));
  const [saved, setSaved] = useState<string>(() => JSON.stringify(toDesired(fromCurrent(initial))));
  const [plan, setPlan] = useState<LifecyclePlan | null>(null);
  const [planError, setPlanError] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [done, setDone] = useState('');
  const adopted = useRef(false);
  const desired = toDesired(selection);
  const changed = JSON.stringify(desired) !== saved;

  useEffect(() => {
    let active = true;
    setPlanError('');
    void lifecyclePlan('secondbrain', 'integrations', { desired }).then(reply => {
      if (!active) return;
      if (!reply.ok) { setPlan(null); setPlanError(describeLifecycleError(reply.error)); return; }
      setPlan(reply.result);
      const current = reply.result.integrations?.current;
      if (!adopted.current && current) {  // first answer: show what the product really has stored, not the on/off guess
        adopted.current = true;
        const stored = fromCurrent(current);
        setSaved(JSON.stringify(toDesired(stored)));
        setSelection(stored);
      }
    });
    return () => { active = false; };
  }, [JSON.stringify(desired)]);

  async function apply() {
    setBusy(true); setError(''); setDone('');
    const reply = await lifecycleApply('secondbrain', 'integrations', { desired });
    setBusy(false);
    if (reply.ok) { setSaved(JSON.stringify(desired)); setDone('Entegrasyonlar uygulandı ve SecondBrain tarafından doğrulandı.'); onApplied(); } else setError(describeLifecycleError(reply.error));
  }

  const detected = plan?.integrations?.clients ? Object.fromEntries(plan.integrations.clients.map(item => [item.id, item.detected])) : null;
  const changes = (plan?.integrations?.changes ?? []).filter(change => change.action !== 'unchanged');
  return <div className="step-body"><h2>AI araç entegrasyonları</h2>
    <p className="muted">Değişiklikler SecondBrain'in kendi kurucusuyla yapılır. Yalnızca Respected'a ait kayıtlar yönetilir; diğer araçların ve senin ayarların korunur, bir aracı kapatınca onun dosyaları byte byte geri yüklenir.</p>
    <h3>Kaynaklar <small className="muted">oturumları hafızaya yakalanan araçlar</small></h3>
    <ClientPicker kind="hooks" selected={selection.sources} detected={detected} onChange={sources => setSelection({ ...selection, sources })} />
    <h3>İzinler <small className="muted">hafızayı MCP ile kullanabilen araçlar</small></h3>
    <ClientPicker kind="mcp" selected={selection.mcp} detected={detected} onChange={mcp => setSelection({ ...selection, mcp })} />
    <label className="choice"><span><strong>Sabah özeti zamanlanmış görevi</strong><small>Windows Görev Zamanlayıcı’ya günlük bir görev ekler.</small></span>
      <input type="checkbox" role="switch" checked={selection.schedule} onChange={event => setSelection({ ...selection, schedule: event.target.checked })} /></label>
    <label className="choice"><span><strong>Masaüstü kısayolu</strong><small>Masaüstüne not kasası klasörünü açan bir kısayol koyar.</small></span>
      <input type="checkbox" role="switch" checked={selection.shortcut} onChange={event => setSelection({ ...selection, shortcut: event.target.checked })} /></label>
    {planError && <p className="error" role="alert">{planError}</p>}
    {plan?.integrations && <IntegrationPreview plan={plan.integrations} hideUnchanged />}
    {error && <p className="error" role="alert">{error}</p>}
    {done && <p className="message" role="status">{done}</p>}
    <div className="actions"><button type="button" className="cta" disabled={busy || !!planError || (!changed && changes.length === 0)} onClick={() => void apply()}>{busy ? 'Uygulanıyor…' : 'Entegrasyonları uygula'}</button></div>
  </div>;
}
