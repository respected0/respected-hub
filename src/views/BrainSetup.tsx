import { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { rppCall } from '../productClient.ts';
import BrainIntegrations from './BrainIntegrations.tsx';
import { describeError } from '../rpp.ts';
import type { RppResponse, SettingsSchema } from '../rpp.ts';
import { changedSettings, formFromSchema, parseFieldValue, verifyReadback } from './productsModel.ts';
import type { FormField } from './productsModel.ts';

const NAME_KEY = 'hub.secondbrain.assistantName';
export const readAssistantName = () => { try { return localStorage.getItem(NAME_KEY) ?? ''; } catch { return ''; } };
const writeAssistantName = (name: string) => { try { localStorage.setItem(NAME_KEY, name); } catch { /* optional personalization */ } };

// Plain-language help for the engine's real settings; unknown keys fall back to the schema label.
const HELP: Record<string, string> = {
  summary_provider: 'Oturumları günlük notlara özetleyen yerel AI CLI. "auto" kurulu olanı seçer.',
  provider_fallback: 'Seçilen sağlayıcı başarısız olursa diğerlerini dene.',
  capture_mode: 'auto: oturumlar bitince kendiliğinden özetlenir. manual: hiçbir şey kendiliğinden yakalanmaz; Ayarlar → “Bekleyen oturumları şimdi yakala” ile işlenir.',
  'integrations.global': 'Kullanıcı düzeyinde AI aracı entegrasyonu.',
  'integrations.mcp': 'Hafızayı MCP sunucusu olarak AI araçlarına aç.',
  'integrations.schedule': 'Zamanlanmış özetleme/derleme görevi.',
  'integrations.shortcut': 'Hızlı erişim kısayolu.',
};
const STEPS = ['Karşılama', 'Özetleyici', 'Entegrasyon tercihleri', 'Onay'];

const failure = (response: RppResponse | null) => {
  if (!response) return 'Ürüne ulaşılamadı.';
  if (response.ok) return '';
  const plain = describeError(response.error);
  return plain.hint ? `${plain.message} ${plain.hint}` : plain.message;
};

// engine: SecondBrain is installed by its own installer, which applies AI-tool integrations (see BrainIntegrations).
export default function BrainSetup({ configured, onDone, engine = false }: { configured: boolean; onDone: () => void; engine?: boolean }) {
  const [step, setStep] = useState(0);
  const [fields, setFields] = useState<FormField[]>([]);
  const [original, setOriginal] = useState<Record<string, unknown>>({});
  const [schema, setSchema] = useState<SettingsSchema | null>(null);
  const [name, setName] = useState(readAssistantName);
  const [state, setState] = useState<'loading' | 'ready' | 'failed'>('loading');
  const [error, setError] = useState('');
  const [done, setDone] = useState('');
  const [busy, setBusy] = useState(false);
  const [vault, setVault] = useState<string | null>(null);
  const [parent, setParent] = useState('');
  const [vaultName, setVaultName] = useState('SecondBrain');

  useEffect(() => {
    let active = true;
    (async () => {
      try {
        const [schemaResponse, readback, manifest] = await Promise.all([rppCall('secondbrain', 'settings.schema'), rppCall('secondbrain', 'settings.readback'), rppCall('secondbrain', 'manifest')]);
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
  const values = () => Object.fromEntries(fields.filter(item => !(engine && item.key.startsWith('integrations.'))).map(item => [item.key, parseFieldValue(item, typeof item.value === 'boolean' ? String(item.value) : item.value)]));
  // A fresh install has no config file yet: write every value once so the engine reports source "file".
  const intended = () => configured ? changedSettings(original, values()) : values();
  const providerFields = fields.filter(item => !item.key.startsWith('integrations.'));
  const integrationFields = fields.filter(item => item.key.startsWith('integrations.'));

  const separator = parent.includes('\\') ? '\\' : '/';
  const vaultPath = parent && vaultName.trim() ? `${parent.replace(/[\\/]+$/, '')}${separator}${vaultName.trim()}` : '';

  async function chooseParent() {
    try { const picked = await open({ directory: true, multiple: false, title: 'Kasanın oluşturulacağı klasörü seç' }); if (typeof picked === 'string') setParent(picked); }
    catch (reason) { setError(String(reason)); }
  }

  async function createVault() {
    setBusy(true); setError('');
    try {
      const response = await rppCall('secondbrain', 'lifecycle.apply', { action: 'install', target: { vault_path: vaultPath, profile: name.trim() ? { USER_NAME: name.trim() } : {} } }, crypto.randomUUID());
      if (!response.ok) { setError(failure(response)); return; }
      const manifest = await rppCall('secondbrain', 'manifest');
      const registered = manifest.ok ? ((manifest.result as { roots?: { vault?: string | null } }).roots?.vault ?? null) : null;
      if (!registered) { setError('Kasa oluşturuldu ama SecondBrain kayıtlı kasayı bildirmedi.'); return; }
      setVault(registered); setDone('Not kasası oluşturuldu ve SecondBrain tarafından doğrulandı.'); onDone();
    } catch (reason) { setError(String(reason)); } finally { setBusy(false); }
  }

  async function apply() {
    setBusy(true); setError('');
    try {
      const payload = intended();
      const applied = await rppCall('secondbrain', 'settings.apply', { settings: payload }, crypto.randomUUID());
      if (!applied.ok) { setError(failure(applied)); return; }
      const readback = await rppCall('secondbrain', 'settings.readback');
      if (!readback.ok) { setError(`Uygulandı ama geri okunamadı. ${failure(readback)}`); return; }
      const current = (readback.result as { settings?: Record<string, unknown> }).settings ?? {};
      const verdict = verifyReadback(payload, current);
      if (!verdict.verified) { setError(`Geri okuma farklı: ${verdict.mismatches.map(m => m.key).join(', ')}`); return; }
      writeAssistantName(name.trim());
      setOriginal(current); setFields(formFromSchema(schema, current));
      setDone('SecondBrain yapılandırıldı ve motordan geri okunarak doğrulandı.');
      onDone();
    } catch (reason) { setError(String(reason)); } finally { setBusy(false); }
  }

  const control = (item: FormField) => <label className="choice" key={item.key}>
    <span><strong>{item.label}</strong><small>{HELP[item.key] ?? ''}</small></span>
    {item.type === 'boolean'
      ? <input type="checkbox" role="switch" checked={item.value === true} onChange={event => set(item.key, event.target.checked)} />
      : item.type === 'enum'
        ? <select value={String(item.value)} onChange={event => set(item.key, event.target.value)}>{item.options.map(option => <option key={option}>{option}</option>)}</select>
        : <input value={String(item.value)} onChange={event => set(item.key, event.target.value)} />}
  </label>;

  if (state === 'loading') return <div className="panel-glass"><p className="muted">Kurulum seçenekleri SecondBrain'den okunuyor…</p></div>;
  if (state === 'failed') return <div className="panel-glass"><p className="error" role="alert">{error}</p><p className="muted">SecondBrain'e ulaşılamadığı için kurulum başlatılamaz. Bağlantılar sayfasından kaynak yolunu kontrol et.</p></div>;

  const pending = Object.keys(intended());
  return <div className="wizard panel-glass">
    <ol className="steps" aria-label="Kurulum adımları">{STEPS.map((label, index) => <li key={label} className={index === step ? 'current' : index < step ? 'past' : ''} aria-current={index === step ? 'step' : undefined}><span>{index + 1}</span>{label}</li>)}</ol>
    {step === 0 && <div className="step-body">
      <h2>{configured ? 'SecondBrain ayarlarını gözden geçir' : "SecondBrain'i hazırlayalım"}</h2>
      <p>Bu sihirbaz yalnızca SecondBrain'in desteklediği ayarları yazar; dosya düzenlemen ya da komut satırı gerekmez.</p>
      <label>Hafıza asistanının adı <small>İsteğe bağlı. Yalnızca Hub'da görünen kişiselleştirmedir; hafıza motorunu etkilemez.</small>
        <input value={name} maxLength={40} placeholder="ör. Jarvis" onChange={event => setName(event.target.value)} /></label>
      <h3>Not kasası</h3>
      {vault ? <p>Kayıtlı kasa: <strong className="mono">{vault}</strong> <small>(mevcut notların korunur; Hub kasayı değiştirmez)</small></p>
        : <div className="vault-create"><p className="muted">Henüz kayıtlı kasa yok. Boş veya yeni bir klasör seç; mevcut notların üzerine yazılmaz.</p>
          <label>Klasör<input value={parent} placeholder="ör. C:\Kullanıcı\Belgeler" onChange={event => setParent(event.target.value)} /></label><div className="actions"><button type="button" disabled={busy} onClick={() => void chooseParent()}>Gözat…</button></div>
          <label>Kasa klasörünün adı<input value={vaultName} maxLength={60} onChange={event => setVaultName(event.target.value)} /></label>
          {vaultPath && <p className="muted">Oluşturulacak: <span className="mono">{vaultPath}</span></p>}
          <button type="button" className="cta" disabled={busy || !vaultPath} onClick={() => void createVault()}>{busy ? 'Oluşturuluyor…' : 'Kasayı oluştur'}</button></div>}
      <p className="muted">SecondBrain'in AI araçlarına bağlanması (kanca/MCP) varsayılan olarak kapalıdır; Hub bu aşamada AI araçlarının yapılandırma dosyalarına yazmaz.</p>
    </div>}
    {step === 1 && <div className="step-body"><h2>Özetleyici sağlayıcı</h2>{providerFields.map(control)}</div>}
    {step === 2 && engine && <BrainIntegrations initial={Object.fromEntries(integrationFields.map(item => [item.key.replace('integrations.', ''), item.value === true]))} onApplied={onDone} />}
    {step === 2 && !engine && <div className="step-body"><h2>Entegrasyon tercihleri</h2><p className="muted">Bu seçimler SecondBrain yapılandırmasına tercih olarak kaydedilir. AI araçlarının kendi ayarlarına uygulanması SecondBrain'in kendi kurucusu tarafından yapılır; Hub şu an AI araç yapılandırmalarını değiştirmez.</p>{integrationFields.map(control)}</div>}
    {step === 3 && <div className="step-body"><h2>Onay</h2>
      {name.trim() && <p>Asistan adı: <strong>{name.trim()}</strong> <small>(Hub'da saklanır)</small></p>}
      <ul className="review">{fields.map(item => <li key={item.key}><span>{item.label}</span><strong>{typeof item.value === 'boolean' ? (item.value ? 'Açık' : 'Kapalı') : item.value}</strong></li>)}</ul>
      <p className="muted">{configured ? `${pending.length} değişiklik uygulanacak.` : 'İlk kurulum: tüm değerler SecondBrain yapılandırmasına yazılacak.'}</p></div>}
    {error && <p className="error" role="alert">{error}</p>}
    {done && <p className="message" role="status">{done}</p>}
    <div className="wizard-actions">
      <button type="button" disabled={step === 0 || busy} onClick={() => setStep(step - 1)}>Geri</button>
      {step < 3 ? <button type="button" className="cta" onClick={() => setStep(step + 1)}>Devam</button>
        : <button type="button" className="cta" disabled={busy || (configured && !pending.length)} onClick={() => void apply()}>{busy ? 'Uygulanıyor…' : configured ? 'Değişiklikleri uygula' : 'Kurulumu tamamla'}</button>}
    </div>
  </div>;
}
