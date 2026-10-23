import { useEffect, useState } from 'react';
import { developmentBuild, loadSettings, native, saveSettings, type Settings } from '../../shared/api.ts';
import InstallFacts from './InstallFacts.tsx';
import { catalogRefresh, catalogSetChannel, describeLifecycleError, hubUpdateInstall, hubUpdateStatus, lifecycleStatus, type Channel, type HubUpdate } from '../lifecycle.ts';

type SaveState = 'idle' | 'saving' | 'applied' | 'verified' | 'failed';

const fields: { key: keyof Settings; label: string; help: string }[] = [
  { key: 'python', label: 'Python programı', help: 'Geliştirme kaynaklarını çalıştırmak için kullanılır.' },
  { key: 'orchestrator_root', label: 'Orchestrator kaynak klasörü', help: 'Hub’ın geliştirme sürümünde Orchestrator’ı arayacağı klasör.' },
  { key: 'brain_root', label: 'Brain kaynak klasörü', help: 'Hub’ın geliştirme sürümünde Brain’ı arayacağı klasör.' },
  { key: 'data_root', label: 'Görev verisi klasörü', help: 'Orchestrator görevlerinin durumlarını sakladığı klasör.' },
];

const sameSettings = (left: Settings, right: Settings) => fields.every(field => left[field.key] === right[field.key]);

// Where updates come from and which releases this Hub offers. Nothing is installed from here: updates are offered as notices.
function Updates({ onCatalogChanged }: { onCatalogChanged?: () => void }) {
  const [info, setInfo] = useState<HubUpdate | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  useEffect(() => { if (native) void hubUpdateStatus().then(reply => { if (reply.ok) setInfo(reply.result); }); }, []);
  async function check() {
    setBusy(true); setMessage('');
    const reply = await catalogRefresh();
    setBusy(false);
    if (!reply.ok) { setMessage(describeLifecycleError(reply.error)); return; }
    setInfo(reply.result);
    setMessage(reply.result.available ? `Respected Hub ${reply.result.available.version} hazır.` : 'Katalog güncel; ürün güncellemeleri Kütüphane’de görünür.');
    onCatalogChanged?.();
  }
  // Downloads and verifies the signed installer; the Hub then closes and reopens by itself.
  async function install() {
    setBusy(true); setMessage('Hub güncellemesi indiriliyor ve doğrulanıyor…');
    const reply = await hubUpdateInstall();
    setBusy(false);
    setMessage(reply.ok ? 'Güncelleme başlatıldı; Hub kapanıp yeniden açılacak.' : describeLifecycleError(reply.error));
  }
  async function choose(channel: Channel) {
    const reply = await catalogSetChannel(channel);
    if (reply.ok) { setInfo(reply.result); onCatalogChanged?.(); } else setMessage(describeLifecycleError(reply.error));
  }
  return <section className="card settings-card" aria-labelledby="settings-updates">
    <h2 id="settings-updates">Güncellemeler</h2>
    <p>Hub ve ürünler GitHub'daki imzalı Respected kataloğundan gelir. Güncelleme olduğunda size sorulur; hiçbir şey kendiliğinden kurulmaz.</p>
    {!native && <div className="empty-note">Yalnızca masaüstü uygulamasında kullanılabilir.</div>}
    {native && info && <div className="settings-fields">
      <label>Hub sürümü<input value={info.current} readOnly /></label>
      <label>Sürüm kanalı
        <select value={info.channel} onChange={event => void choose(event.target.value as Channel)}>
          <option value="beta">Beta: beta ve kararlı sürümler</option>
          <option value="stable">Kararlı: yalnızca kararlı sürümler</option>
        </select>
        <small>Kararlıya geçmek kurulu sürümü geri almaz; bir sonraki kararlı sürüm geldiğinde güncelleme önerilir.</small></label>
    </div>}
    {native && info?.available && <div className="notice" role="status"><span>Respected Hub {info.available.version} hazır (şu an {info.current}).</span>
      <span className="actions"><button className="cta" disabled={busy} onClick={() => void install()}>Hub'ı güncelle</button></span></div>}
    {native && <div className="settings-actions"><button className="button primary" disabled={busy} onClick={() => void check()}>{busy ? 'Denetleniyor…' : 'Güncellemeleri denetle'}</button>
      {message && <output className="toast" role="status">{message}</output>}</div>}
  </section>;
}

export default function Settings({ onCatalogChanged }: { onCatalogChanged?: () => void }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [state, setState] = useState<SaveState>('idle');
  const [managed, setManaged] = useState<string[]>([]);
  useEffect(() => { if (native) void lifecycleStatus().then(reply => { if (reply.ok) setManaged(reply.result.products.filter(item => item?.installed).map(item => item!.product)); }); }, []);
  const [error, setError] = useState('');
  const [development, setDevelopment] = useState(false);   // source search paths only mean something in a development build
  useEffect(() => { if (native) developmentBuild().then(setDevelopment).catch(() => setDevelopment(false)); }, []);

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

  return <div className="settings-stack">
    <Updates onCatalogChanged={onCatalogChanged} />
    {development && <section className="card settings-card" aria-labelledby="settings-sources">
      <h2 id="settings-sources">Kaynak arama yolları</h2>
      <p>Bu ayarlar Hub’ın geliştirme sürümünde ürün kaynaklarını nerede arayacağını belirler. Canlı AI yapılandırmalarına dokunmaz.</p>
      {managed.length > 0 && <div className="notice" role="note">Kurulu ürünler ({managed.join(', ')}) kendi programlarından çalışır; aşağıdaki kaynak klasörleri bu ürünler için kullanılmaz.</div>}
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
    </section>}
    <section className="card settings-card" aria-label="Hub'ın kurduğu şeyler"><InstallFacts subject="hub" /></section>
  </div>;
}
