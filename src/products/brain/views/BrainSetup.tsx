import { useEffect, useState } from 'react';
import { rppCall } from '../../../shared/productClient.ts';
import BrainIntegrations from './BrainIntegrations.tsx';
import { describeError } from '../../../shared/rpp.ts';
import type { RppResponse, SettingsSchema } from '../../../shared/rpp.ts';
import { changedSettings, formFromSchema, parseFieldValue, verifyReadback } from '../../../shared/productsModel.ts';
import type { FormField } from '../../../shared/productsModel.ts';

// Installed Brain's settings, drawn from its own schema (labels and help come from the product). The same settings
// exist without the Hub: `respected-brain config set <key>=<value>` or the installer's "Ayarları değiştir".
const GROUPS: [string, (key: string) => boolean][] = [
  ['Kimlik', key => ['user_name', 'assistant_name', 'user_bio'].includes(key)],
  ['Hafıza', key => !key.startsWith('integrations.') && !['user_name', 'assistant_name', 'user_bio'].includes(key)],
];

const failure = (response: RppResponse | null) => {
  if (!response) return 'Ürüne ulaşılamadı.';
  if (response.ok) return '';
  const plain = describeError(response.error);
  return plain.hint ? `${plain.message} ${plain.hint}` : plain.message;
};

export default function BrainSetup({ onDone }: { onDone: () => void }) {
  const [fields, setFields] = useState<FormField[]>([]);
  const [original, setOriginal] = useState<Record<string, unknown>>({});
  const [schema, setSchema] = useState<SettingsSchema | null>(null);
  const [state, setState] = useState<'loading' | 'ready' | 'failed'>('loading');
  const [error, setError] = useState('');
  const [done, setDone] = useState('');
  const [busy, setBusy] = useState(false);
  const [vault, setVault] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    (async () => {
      try {
        const [schemaResponse, readback, manifest] = await Promise.all([rppCall('brain', 'settings.schema'), rppCall('brain', 'settings.readback'), rppCall('brain', 'manifest')]);
        if (!active) return;
        if (manifest.ok) setVault(((manifest.result as { roots?: { vault?: string | null } }).roots?.vault) ?? null);
        if (!schemaResponse.ok || !readback.ok) { setError(failure(schemaResponse) || failure(readback)); setState('failed'); return; }
        const current = (readback.result as { settings?: Record<string, unknown> }).settings ?? {};
        setSchema(schemaResponse.result as SettingsSchema); setOriginal(current);
        setFields(formFromSchema(schemaResponse.result as SettingsSchema, current)); setState('ready');
      } catch (reason) { if (active) { setError(String(reason)); setState('failed'); } }
    })();
    return () => { active = false; };
  }, []);

  const set = (key: string, value: string | boolean) => setFields(list => list.map(item => item.key === key ? { ...item, value } : item));
  const values = () => Object.fromEntries(fields.filter(item => !item.key.startsWith('integrations.'))
    .map(item => [item.key, parseFieldValue(item, typeof item.value === 'boolean' ? String(item.value) : item.value)]));
  const pending = changedSettings(original, values());

  async function apply() {
    setBusy(true); setError(''); setDone('');
    try {
      const applied = await rppCall('brain', 'settings.apply', { settings: pending }, crypto.randomUUID());
      if (!applied.ok) { setError(failure(applied)); return; }
      const readback = await rppCall('brain', 'settings.readback');
      if (!readback.ok) { setError(`Uygulandı ama geri okunamadı. ${failure(readback)}`); return; }
      const current = (readback.result as { settings?: Record<string, unknown> }).settings ?? {};
      const verdict = verifyReadback(pending, current);
      if (!verdict.verified) { setError(`Geri okuma farklı: ${verdict.mismatches.map(m => m.key).join(', ')}`); return; }
      setOriginal(current); setFields(formFromSchema(schema, current));
      setDone('Ayarlar Brain tarafından kaydedildi ve geri okunarak doğrulandı.');
      onDone();
    } catch (reason) { setError(String(reason)); } finally { setBusy(false); }
  }

  const control = (item: FormField) => <label className="choice" key={item.key}>
    <span><strong>{item.label}</strong><small>{item.help}</small></span>
    {item.type === 'boolean'
      ? <input type="checkbox" role="switch" checked={item.value === true} onChange={event => set(item.key, event.target.checked)} />
      : item.type === 'enum'
        ? <select value={String(item.value)} onChange={event => set(item.key, event.target.value)}>{item.options.map(option => <option key={option}>{option}</option>)}</select>
        : <input value={String(item.value)} onChange={event => set(item.key, event.target.value)} />}
  </label>;

  if (state === 'loading') return <div className="panel-glass"><p className="muted">Ayarlar Brain'den okunuyor…</p></div>;
  if (state === 'failed') return <div className="panel-glass"><p className="error" role="alert">{error}</p><p className="muted">Brain'e ulaşılamadı; kurulu değilse Uygulamalarım'dan kurun, kuruluysa onarın.</p></div>;

  return <div className="panel-glass">
    <div className="step-body"><h2>Brain ayarları</h2>
      <p className="muted">Not kasası: {vault ? <strong className="mono">{vault}</strong> : 'kayıtlı değil'} <small>(kasayı kurucu oluşturur; Hub kasaya dokunmaz)</small></p>
      {GROUPS.map(([title, inGroup]) => <section key={title}><h3>{title}</h3>{fields.filter(item => inGroup(item.key)).map(control)}</section>)}
      {error && <p className="error" role="alert">{error}</p>}
      {done && <p className="message" role="status">{done}</p>}
      <div className="actions"><button type="button" className="cta" disabled={busy || !Object.keys(pending).length} onClick={() => void apply()}>{busy ? 'Kaydediliyor…' : `Değişiklikleri kaydet${Object.keys(pending).length ? ` (${Object.keys(pending).length})` : ''}`}</button></div>
    </div>
    <BrainIntegrations settings={original} onApplied={onDone} />
  </div>;
}
