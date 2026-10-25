import { useEffect, useRef, useState } from 'react';
import { rppCall } from '../../../shared/productClient.ts';
import { integrationsPlan, peekIntegrationsPlan } from '../../../shared/settingsData.ts';
import { detectAiClients } from '../../../shell/lifecycle.ts';
import { describeError } from '../../../shared/rpp.ts';
import type { RppResponse } from '../../../shared/rpp.ts';
import type { IntegrationPlan } from '../../../shell/lifecycle.ts';
import { fromCurrent, fromSettings, toDesired, toSettings } from '../brainClients.ts';
import type { Selection } from '../brainClients.ts';
import ClientPicker from './ClientPicker.tsx';
import IntegrationPreview from './IntegrationPreview.tsx';

const failure = (response: RppResponse) => {
  if (response.ok) return '';
  const plain = describeError(response.error);
  return plain.hint ? `${plain.message} ${plain.hint}` : plain.message;
};

// Installed Brain: choose, per AI client, where memory is captured (Sources) and who may use it over MCP (Permissions).
// The product shows exactly what would change (`integrations.plan`, read-only) and applies the choice as its own settings
// (`integrations.*`), the same keys `respected-brain config set` and its installer take.
export default function BrainIntegrations({ settings, onApplied }: { settings: Record<string, unknown>; onApplied: () => void }) {
  const [selection, setSelection] = useState<Selection>(() => fromSettings(settings));
  const [saved, setSaved] = useState<string>(() => JSON.stringify(toSettings(fromSettings(settings))));
  const [plan, setPlan] = useState<IntegrationPlan | null>(() => peekIntegrationsPlan(toDesired(fromSettings(settings))));  // the last answer (kept warm), so the client rows are not "Aranıyor…" while the program starts
  const [planError, setPlanError] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [done, setDone] = useState('');
  const [local, setLocal] = useState<Record<string, boolean> | null>(null);  // read by the Hub itself, at once; the product's own answer (below) replaces it
  const adopted = useRef(false);
  useEffect(() => { void detectAiClients().then(setLocal, () => setLocal(null)); }, []);
  const desired = toDesired(selection);
  const changed = JSON.stringify(toSettings(selection)) !== saved;

  useEffect(() => {
    let active = true;
    setPlanError('');
    void integrationsPlan(desired).then(reply => {
      if (!active) return;
      if (!reply) { setPlan(null); setPlanError('Brain’e ulaşılamadı.'); return; }
      if (!reply.ok) { setPlan(null); setPlanError(failure(reply)); return; }
      const result = reply.result as IntegrationPlan;
      setPlan(result);
      if (!adopted.current && result.current) {  // first answer: show what the product really has stored
        adopted.current = true;
        const stored = fromCurrent(result.current);
        setSaved(JSON.stringify(toSettings(stored)));
        setSelection(stored);
      }
    });
    return () => { active = false; };
  }, [JSON.stringify(desired)]);

  async function apply() {
    setBusy(true); setError(''); setDone('');
    const reply = await rppCall('brain', 'settings.apply', { settings: toSettings(selection) }, crypto.randomUUID());
    setBusy(false);
    if (!reply.ok) { setError(failure(reply)); return; }
    setSaved(JSON.stringify(toSettings(selection))); setDone('Bağlantılar Brain tarafından uygulandı.'); onApplied();
    // the "Değişecekler" list was planned against the files as they were: ask again, now there is nothing left to change (the choice itself did not change, so no effect reruns)
    const fresh = await integrationsPlan(desired);
    if (fresh?.ok) setPlan(fresh.result as IntegrationPlan);
  }

  const detected = plan?.clients ? Object.fromEntries(plan.clients.map(item => [item.id, item.detected])) : local;
  const changes = (plan?.changes ?? []).filter(change => change.action !== 'unchanged');
  return <div className="step-body"><h2>AI araç bağlantıları</h2>
    <p className="muted">Değişiklikleri Brain kendisi yapar: yalnızca Respected'a ait kayıtlar yönetilir, diğer araçların ve sizin ayarlarınız korunur, bir aracı kapatınca Respected'ın o araçtaki kayıtları kaldırılır.</p>
    <h3>Kaynaklar <small className="muted">oturumları hafızaya yakalanan araçlar</small></h3>
    <ClientPicker kind="hooks" selected={selection.sources} detected={detected} failed={!!planError} onChange={sources => setSelection({ ...selection, sources })} />
    <h3>İzinler <small className="muted">hafızayı MCP ile kullanabilen araçlar</small></h3>
    <ClientPicker kind="mcp" selected={selection.mcp} detected={detected} failed={!!planError} onChange={mcp => setSelection({ ...selection, mcp })} />
    {selection.mcp.includes('codex') && !selection.sources.includes('codex') && <p className="muted">Codex'e hafıza MCP'si eklentiyle gelir: Codex'i Kaynaklar'da da açın.</p>}
    <label className="choice"><span><strong>Sabah özeti zamanlanmış görevi</strong><small>Windows Görev Zamanlayıcı’ya günlük bir görev ekler.</small></span>
      <input type="checkbox" role="switch" checked={selection.schedule} onChange={event => setSelection({ ...selection, schedule: event.target.checked })} /></label>
    <label className="choice"><span><strong>Masaüstü kısayolu</strong><small>Masaüstüne not kasasını Obsidian'da açan bir kısayol koyar. Çalışması için kasanın Obsidian'da en az bir kez açılmış olması gerekir.</small></span>
      <input type="checkbox" role="switch" checked={selection.shortcut} onChange={event => setSelection({ ...selection, shortcut: event.target.checked })} /></label>
    {planError && <p className="error" role="alert">{planError}</p>}
    {plan && <IntegrationPreview plan={plan} hideUnchanged />}
    {error && <p className="error" role="alert">{error}</p>}
    {done && <p className="message" role="status">{done}</p>}
    <div className="actions"><button type="button" className="cta" disabled={busy || !!planError || (!changed && changes.length === 0)} onClick={() => void apply()}>{busy ? 'Uygulanıyor…' : 'Bağlantıları uygula'}</button></div>
  </div>;
}
