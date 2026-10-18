import { useCallback, useEffect, useState } from 'react';
import { native } from '../api.ts';
import { rppCall } from '../productClient.ts';
import type { RppProduct } from '../productClient.ts';
import { describeError, SECRET_REDACTED } from '../rpp.ts';
import type { RppResponse, SettingsSchema, SettingsValidation } from '../rpp.ts';
import { changedSettings, formFromSchema, parseFieldValue, summarizeProduct, verifyReadback } from './productsModel.ts';
import type { FormField } from './productsModel.ts';

const PRODUCTS: { id: RppProduct; name: string; description: string }[] = [
  { id: 'orchestrator', name: 'Respected Orchestrator', description: 'Bağımsız görev motoru.' },
  { id: 'secondbrain', name: 'RespectedBrain', description: 'AI hafıza motoru.' },
];

async function call(product: RppProduct, op: string, payload?: Record<string, unknown>, operationId?: string): Promise<RppResponse | null> {
  try { return await rppCall(product, op, payload, operationId); }
  catch { return null; }
}

function failureText(response: RppResponse | null): string {
  if (!response) return 'Ürüne ulaşılamadı: çağrı yanıt vermedi.';
  if (response.ok) return '';
  const plain = describeError(response.error);
  return plain.hint ? `${plain.message} ${plain.hint}` : plain.message;
}

function settingsFrom(response: RppResponse | null): Record<string, unknown> {
  if (!response || !response.ok) return {};
  const result = response.result;
  if (typeof result !== 'object' || result === null || Array.isArray(result)) return {};
  const settings = (result as { settings?: unknown }).settings;
  return typeof settings === 'object' && settings !== null && !Array.isArray(settings) ? settings as Record<string, unknown> : {};
}

function ProductCard({ product }: { product: { id: RppProduct; name: string; description: string } }) {
  const [manifest, setManifest] = useState<RppResponse | null>(null);
  const [health, setHealth] = useState<RppResponse | null>(null);
  const [loading, setLoading] = useState(false);

  const [schema, setSchema] = useState<SettingsSchema | null>(null);
  const [fields, setFields] = useState<FormField[]>([]);
  const [original, setOriginal] = useState<Record<string, unknown>>({});
  const [settingsState, setSettingsState] = useState<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  const [settingsLoadError, setSettingsLoadError] = useState('');
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const [settingsMessage, setSettingsMessage] = useState('');
  const [settingsError, setSettingsError] = useState('');

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [manifestResponse, healthResponse] = await Promise.all([call(product.id, 'manifest'), call(product.id, 'health')]);
      setManifest(manifestResponse);
      setHealth(healthResponse);
    } finally { setLoading(false); }
  }, [product.id]);

  useEffect(() => { void refresh(); }, [refresh]);

  const loadSettings = useCallback(async () => {
    setSettingsState('loading');
    setSettingsLoadError('');
    const [schemaResponse, readbackResponse] = await Promise.all([call(product.id, 'settings.schema'), call(product.id, 'settings.readback')]);
    if (!schemaResponse || !schemaResponse.ok) {
      setSettingsState('failed');
      setSettingsLoadError(failureText(schemaResponse) || 'Ayarlar şeması alınamadı.');
      return;
    }
    const schemaResult = typeof schemaResponse.result === 'object' && schemaResponse.result !== null ? schemaResponse.result as SettingsSchema : null;
    const settings = settingsFrom(readbackResponse);
    setSchema(schemaResult);
    setOriginal(settings);
    setFields(formFromSchema(schemaResult, settings));
    setSettingsState('ready');
    if (!readbackResponse || !readbackResponse.ok) setSettingsLoadError(`${failureText(readbackResponse)} Şema varsayılan değerlerle gösteriliyor.`);
  }, [product.id]);

  function editedValues(): Record<string, unknown> {
    const edited: Record<string, unknown> = {};
    for (const item of fields) {
      const raw = typeof item.value === 'boolean' ? String(item.value) : item.value;
      if (item.secret && raw === '') continue;
      edited[item.key] = parseFieldValue(item, raw);
    }
    return edited;
  }

  function setFieldValue(key: string, value: string | boolean) {
    setFields((current) => current.map((item) => item.key === key ? { ...item, value } : item));
  }

  async function validate() {
    setBusy(true); setFieldErrors({}); setSettingsMessage(''); setSettingsError('');
    try {
      const response = await rppCall(product.id, 'settings.validate', { settings: editedValues() });
      if (!response.ok) { setSettingsError(failureText(response)); return; }
      const result = typeof response.result === 'object' && response.result !== null ? response.result as SettingsValidation : null;
      const errors: Record<string, string> = {};
      for (const error of result?.errors ?? []) {
        if (error && typeof error.key === 'string') errors[error.key] = error.message || 'Geçersiz değer.';
      }
      setFieldErrors(errors);
      if (!result || result.valid !== true || Object.keys(errors).length) setSettingsMessage('Bazı alanlar geçersiz; düzeltip tekrar dene.');
      else setSettingsMessage('Ayarlar geçerli. Uygulamak için "Uygula" düğmesine bas.');
    } catch (reason) { setSettingsError(String(reason)); }
    finally { setBusy(false); }
  }

  async function apply() {
    setBusy(true); setFieldErrors({}); setSettingsMessage(''); setSettingsError('');
    try {
      const intended = changedSettings(original, editedValues());
      if (!Object.keys(intended).length) { setSettingsMessage('Değiştirilmiş ayar yok.'); return; }
      const applied = await rppCall(product.id, 'settings.apply', { settings: intended }, crypto.randomUUID());
      if (!applied.ok) { setSettingsError(failureText(applied)); return; }
      const readbackResponse = await rppCall(product.id, 'settings.readback');
      if (!readbackResponse.ok) {
        setSettingsError(`Ayarlar uygulandı ama motordan geri okunamadı: ${failureText(readbackResponse)}`);
        return;
      }
      const readback = settingsFrom(readbackResponse);
      setOriginal(readback);
      setFields(formFromSchema(schema, readback));
      const verdict = verifyReadback(intended, readback);
      if (verdict.verified) setSettingsMessage('Uygulandı ve motordan geri okunarak doğrulandı.');
      else setSettingsError(`Ayarlar uygulandı ancak geri okuma farklı: ${verdict.mismatches.map((item) => `${item.key} (beklenen ${item.expected || 'boş'}, okunan ${item.actual})`).join('; ')}`);
    } catch (reason) { setSettingsError(String(reason)); }
    finally { setBusy(false); }
  }

  const summary = summarizeProduct(manifest, health);
  const settingsBusy = busy || settingsState !== 'ready';

  return <article className="card">
    <div className="card-head">
      <div><h2>{summary.title || product.name}</h2><p>{product.description}</p></div>
      <span className={`badge-status ${summary.statusKind}`}>{summary.statusText}</span>
    </div>
    <p aria-live="polite" className="status-line" role="status">{summary.statusText}{summary.version ? ` · Sürüm ${summary.version}` : ''}</p>
    <div className="card-actions"><button type="button" disabled={loading} onClick={() => { setSettingsState('idle'); void refresh(); }}>{loading ? 'Yükleniyor…' : 'Yenile'}</button></div>
    {summary.checks.length > 0 && <ul className="checks">{summary.checks.map((check, index) => <li key={`${check.name}-${index}`} className={check.ok ? 'ok' : 'bad'}><span aria-hidden="true">{check.ok ? '✓' : '✕'}</span><span><strong>{check.name}</strong>{check.detail ? `: ${check.detail}` : ''}</span></li>)}</ul>}
    {summary.capabilities.length > 0 && <ul className="capabilities" aria-label="Yetenekler">{summary.capabilities.map((capability) => <li key={capability}>{capability}</li>)}</ul>}
    {summary.errorText && <p className="error" role="alert">{summary.errorText}</p>}

    <details className="settings-block" onToggle={(event) => { if ((event.target as HTMLDetailsElement).open && settingsState === 'idle') void loadSettings(); }}>
      <summary>Ayarlar</summary>
      {settingsState === 'loading' && <p className="muted">Ayarlar yükleniyor…</p>}
      {settingsState === 'failed' && <p className="error" role="alert">{settingsLoadError}</p>}
      {settingsState === 'ready' && <>
        {settingsLoadError && <p className="muted" role="alert">{settingsLoadError}</p>}
        {!fields.length && <p className="muted">Bu ürün ayar tanımlamıyor.</p>}
        <form onSubmit={(event) => { event.preventDefault(); void apply(); }}>
          {fields.map((item) => <label key={item.key}>
            {item.label}{item.secret ? ' (gizli)' : ''}
            {item.type === 'boolean'
              ? <input type="checkbox" checked={item.value === true} onChange={(event) => setFieldValue(item.key, event.target.checked)} />
              : item.type === 'enum'
                ? <select value={String(item.value)} onChange={(event) => setFieldValue(item.key, event.target.value)}>{item.options.map((option) => <option key={option} value={option}>{option}</option>)}</select>
                : <input type={item.type === 'integer' ? 'number' : item.secret ? 'password' : 'text'} value={String(item.value)} min={item.min} max={item.max} onChange={(event) => setFieldValue(item.key, event.target.value)} placeholder={item.secret && original[item.key] === SECRET_REDACTED ? 'Değiştirmek için yaz' : ''} />}
            {(item.min !== undefined || item.max !== undefined) && <small>Geçerli aralık: {item.min ?? 'alt sınır yok'} – {item.max ?? 'üst sınır yok'}</small>}
            {fieldErrors[item.key] && <small className="field-error" role="alert">{fieldErrors[item.key]}</small>}
          </label>)}
          <div className="card-actions">
            <button type="button" disabled={settingsBusy} onClick={() => void validate()}>Doğrula</button>
            <button type="submit" className="primary-inline" disabled={settingsBusy || (!!settingsLoadError && Object.keys(original).length === 0)} title={settingsLoadError ? 'Mevcut ayarlar okunamadığı için uygulama kapalı' : undefined}>{busy ? 'İşleniyor…' : 'Uygula'}</button>
          </div>
        </form>
        {settingsMessage && <p className="message" role="status">{settingsMessage}</p>}
        {settingsError && <p className="error" role="alert">{settingsError}</p>}
        <p className="muted">Uygulama sonrası ayarlar motordan her zaman geri okunur; başarı yalnızca geri okuma eşleşirse bildirilir.</p>
      </>}
    </details>
  </article>;
}

export default function Products() {
  if (!native) return <div className="notice warning">Ürün durumunu ve ayarları görmek için Tauri masaüstü uygulaması gerekli. Tarayıcı önizlemesinde gerçek ürün verisi gösterilmez.</div>;
  return <section aria-label="Ürünler" className="products-view">{PRODUCTS.map((product) => <ProductCard key={product.id} product={product} />)}</section>;
}


// One product's status and settings, for use inside that app's own workspace.
export function ProductPanel({ id }: { id: RppProduct }) {
  const product = PRODUCTS.find(item => item.id === id);
  if (!native) return <div className="notice warning">Ayarlar için Tauri masaüstü uygulaması gerekli.</div>;
  return product ? <ProductCard product={product} /> : null;
}
