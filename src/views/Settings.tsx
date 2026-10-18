import { useEffect, useState } from 'react';
import { loadSettings, native, saveSettings, type Settings } from '../api.ts';
import { lifecycleStatus } from '../lifecycle.ts';

type SaveState = 'idle' | 'saving' | 'applied' | 'verified' | 'failed';

const fields: { key: keyof Settings; label: string; help: string }[] = [
  { key: 'python', label: 'Python programı', help: 'Geliştirme kaynaklarını çalıştırmak için kullanılır.' },
  { key: 'orchestrator_root', label: 'Orchestrator kaynak klasörü', help: 'Hub’ın geliştirme sürümünde Orchestrator’ı arayacağı klasör.' },
  { key: 'brain_root', label: 'SecondBrain kaynak klasörü', help: 'Hub’ın geliştirme sürümünde SecondBrain’ı arayacağı klasör.' },
  { key: 'data_root', label: 'Görev verisi klasörü', help: 'Orchestrator görevlerinin durumlarını sakladığı klasör.' },
];

const sameSettings = (left: Settings, right: Settings) => fields.every(field => left[field.key] === right[field.key]);

export default function Settings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [state, setState] = useState<SaveState>('idle');
  const [managed, setManaged] = useState<string[]>([]);
  useEffect(() => { if (native) void lifecycleStatus().then(reply => { if (reply.ok) setManaged(reply.result.products.filter(item => item?.installed).map(item => item!.product)); }); }, []);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!native) return;
    let cancelled = false;
    loadSettings().then(value => { if (!cancelled) setSettings(value); }).catch(reason => { if (!cancelled) setError(String(reason)); });
    return () => { cancelled = true; };
  }, []);

  async function apply() {
    if (!settings) return;
    setState('saving'); setError('');
    try {
      await saveSettings(settings);
      setState('applied');
      const readback = await loadSettings();
      setState(sameSettings(settings, readback) ? 'verified' : 'failed');
      if (!sameSettings(settings, readback)) setError('Kayıt doğrulaması farklı değerler döndürdü.');
    } catch (reason) {
      setState('failed'); setError(String(reason));
    }
  }

  return <div className="page">
    <div className="section-head"><div><span className="eyebrow">Ayarlar</span><h1>Hub ayarları</h1></div></div>
    <section className="card settings-card" aria-labelledby="settings-sources">
      <h2 id="settings-sources">Kaynak arama yolları</h2>
      <p>Bu ayarlar Hub’ın geliştirme sürümünde ürün kaynaklarını nerede arayacağını ve görev verilerini nerede saklayacağını belirler. Canlı AI yapılandırmalarına dokunmaz.</p>
      {managed.length > 0 && <div className="notice" role="note">Hub'dan kurulu ürünler ({managed.join(', ')}) yönetilen kopyalarından çalışır; aşağıdaki kaynak klasörleri bu ürünler için kullanılmaz.</div>}
      {!native && <div className="empty-note">Ayarlar yalnızca masaüstü uygulamasında okunabilir ve uygulanabilir. Tarayıcı önizlemesinde değer uydurulmaz.</div>}
      {native && !settings && !error && <div className="settings-loading"><progress aria-label="Ayarlar okunuyor" /><p>Kayıtlı ayarlar okunuyor…</p></div>}
      {native && settings && <form onSubmit={event => { event.preventDefault(); void apply(); }}>
        <div className="settings-fields">
          {fields.map(field => <label key={field.key}>{field.label}
            <input value={settings[field.key]} onChange={event => setSettings({ ...settings, [field.key]: event.target.value })} required />
            <small>{field.help}</small>
          </label>)}
        </div>
        <div className="settings-actions">
          <button className="button primary" type="submit" disabled={state === 'saving'}>{state === 'saving' ? 'Uygulanıyor…' : 'Kaydet ve Uygula'}</button>
          {state !== 'idle' && <output className={`toast state-${state}`} role="status">
            {state === 'saving' && 'Ayarlar kaydediliyor ve doğrulanıyor.'}
            {state === 'applied' && 'Ayarlar kaydedildi; doğrulama sürüyor.'}
            {state === 'verified' && 'Ayarlar uygulandı ve doğrulandı.'}
            {state === 'failed' && 'Ayarlar uygulanamadı veya doğrulanamadı.'}
          </output>}
        </div>
      </form>}
      {error && <p className="error" role="alert">{error}</p>}
      <p className="muted">Kaydetme işlemi önce doğrulama kurallarını çalıştırır, sonra ayarları yeniden okuyarak sonucu kontrol eder.</p>
    </section>
  </div>;
}
