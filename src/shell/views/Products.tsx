import { useCallback, useEffect, useState } from 'react';
import { native } from '../../shared/api.ts';
import { Icon } from '../../shared/ui.tsx';
import { plainCheck } from '../../shared/healthText.ts';
import { rppCall } from '../../shared/productClient.ts';
import type { RppProduct } from '../../shared/productClient.ts';
import { forgetReadback, peekSettings, settingsReadback, settingsSchema } from '../../shared/settingsData.ts';
import { describeError, SECRET_REDACTED } from '../../shared/rpp.ts';
import type { RppResponse, SettingsSchema, SettingsValidation } from '../../shared/rpp.ts';
import { changedSettings, formFromSchema, parseFieldValue, verifyReadback } from '../../shared/productsModel.ts';
import type { ProductSummary } from '../../shared/productsModel.ts';
import type { FormField } from '../../shared/productsModel.ts';

// `scope` says what the Hub's quick checks cover, so the list is not mistaken for a check of the product's content.
const PRODUCTS: { id: RppProduct; name: string; description: string; short: string; scope: string }[] = [
  { id: 'orchestrator', name: 'Respected Orchestrator', description: 'Bağımsız görev motoru.', short: 'Orchestrator', scope: 'Hub, Orchestrator\'a şu hızlı kontrolleri yapar: veri klasörü, git, çalışan araçlar ve motor anahtarı hazır mı. Görevlerin sonuçlarını denetlemez; onlar Kanıtlar sayfasındadır.' },
  { id: 'brain', name: 'RespectedBrain', description: 'AI hafıza motoru.', short: 'Brain', scope: 'Hub, Brain\'e şu hızlı kontrolleri yapar: ulaşılıyor mu, ayarlar ve veri klasörü okunuyor mu, not kasası bağlı mı. Notlarınızın içeriğini denetlemez; onun için brain-doktor becerisi var.' },
];

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

// The product's settings form (its workspace Settings page).
function ProductCard({ product }: { product: (typeof PRODUCTS)[number] }) {
  // the last answers (kept warm since the Hub opened) fill the form in the very first frame, so it never flashes empty
  const [last] = useState(() => { const known = peekSettings(product.id); return known.schema?.ok && known.readback?.ok ? known : null; });
  const [schema, setSchema] = useState<SettingsSchema | null>(() => last && typeof last.schema?.result === 'object' ? last.schema.result as SettingsSchema : null);
  const [original, setOriginal] = useState<Record<string, unknown>>(() => settingsFrom(last?.readback ?? null));
  const [fields, setFields] = useState<FormField[]>(() => last ? formFromSchema(schema, original) : []);
  const [settingsState, setSettingsState] = useState<'loading' | 'ready' | 'failed'>(last ? 'ready' : 'loading');
  const [settingsLoadError, setSettingsLoadError] = useState('');
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const [settingsMessage, setSettingsMessage] = useState('');
  const [settingsError, setSettingsError] = useState('');

  const loadSettings = useCallback(async () => {
    setSettingsLoadError('');
    const show = (schemaResponse: RppResponse | null, readbackResponse: RppResponse | null) => {
      const schemaResult = schemaResponse?.ok && typeof schemaResponse.result === 'object' && schemaResponse.result !== null ? schemaResponse.result as SettingsSchema : null;
      const settings = settingsFrom(readbackResponse);
      setSchema(schemaResult);
      setOriginal(settings);
      setFields(formFromSchema(schemaResult, settings));
      setSettingsState('ready');
    };
    // the fresh answers replace the ones already shown (see `last`) only if they differ
    const [schemaResponse, readbackResponse] = await Promise.all([settingsSchema(product.id), settingsReadback(product.id)]);
    if (!schemaResponse || !schemaResponse.ok) {
      setSettingsState('failed');
      setSettingsLoadError(failureText(schemaResponse) || 'Ayarlar şeması alınamadı.');
      return;
    }
    if (!last || JSON.stringify(settingsFrom(readbackResponse)) !== JSON.stringify(settingsFrom(last.readback ?? null))) show(schemaResponse, readbackResponse);
    if (!readbackResponse || !readbackResponse.ok) setSettingsLoadError(`${failureText(readbackResponse)} Şema varsayılan değerlerle gösteriliyor.`);
  }, [product.id, last]);

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
      if (!result || result.valid !== true || Object.keys(errors).length) setSettingsMessage('Bazı alanlar geçersiz; düzeltip tekrar deneyin.');
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
      forgetReadback(product.id);  // the stored answer is from before this change
      const readbackResponse = await settingsReadback(product.id);
      if (!readbackResponse || !readbackResponse.ok) {
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

  useEffect(() => { void loadSettings(); }, [loadSettings]);

  const settingsBusy = busy || settingsState !== 'ready';

  const settingsForm = <>
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
  </>;

  return <article className="card"><div className="card-head"><div><h2>{product.name} ayarları</h2><p>Değerler ürünün kendi şemasıyla doğrulanır ve uygulandıktan sonra geri okunur.</p></div></div>{settingsForm}</article>;
}

function StatusCard({ product, summary }: { product: (typeof PRODUCTS)[number]; summary: ProductSummary }) {
  // the Overview greeting already says how the product is doing; this card lists what was checked, in plain words
  return <article className="card">
    <div className="card-head"><div><h2>{product.short} çalışma durumu</h2><p>{product.scope}</p></div></div>
    {summary.checks.length > 0 && <p className="status-line">{summary.checks.filter(check => check.ok).length} / {summary.checks.length} kontrol geçti.</p>}
    {summary.checks.length > 0 && <ul className="checks">{summary.checks.map((check, index) => { const plain = plainCheck(check.name, check.ok, check.detail);
      return <li key={`${check.name}-${index}`} className={check.ok ? 'ok' : 'bad'}><Icon name={check.ok ? 'check' : 'x'} /><span><strong>{plain.label}</strong>{plain.text ? `: ${plain.text}` : ''}</span></li>; })}</ul>}
    {summary.errorText && <p className="error" role="alert">{summary.errorText}</p>}
  </article>;
}

// One product's settings form, for use inside that app's own workspace.
export function ProductPanel({ id }: { id: RppProduct }) {
  const product = PRODUCTS.find(item => item.id === id);
  if (!native) return <div className="notice warning">Ayarlar için Tauri masaüstü uygulaması gerekli.</div>;
  return product ? <ProductCard product={product} /> : null;
}

// One product's health checks, from the summary the shell already read (no second call to the product).
export function ProductStatusPanel({ id, summary }: { id: RppProduct; summary: ProductSummary }) {
  const product = PRODUCTS.find(item => item.id === id);
  if (!native) return <div className="notice warning">Ürün durumunu görmek için Tauri masaüstü uygulaması gerekli. Tarayıcı önizlemesinde gerçek ürün verisi gösterilmez.</div>;
  return product ? <StatusCard product={product} summary={summary} /> : null;
}
