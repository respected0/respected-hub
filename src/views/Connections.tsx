import { useEffect, useState } from 'react';
import { loadSettings, native, products, saveSettings, type Settings } from '../api';
import Products from './Products.tsx';

// Development source connections (python/product roots) plus every product's RPP settings.
export default function Connections() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [development, setDevelopment] = useState(false);   // source connections exist only in development builds
  useEffect(() => { if (native) products().then(p => setDevelopment(p.development)).catch(() => setDevelopment(false)); }, []);
  useEffect(() => { if (native) loadSettings().then(setSettings).catch(reason => setError(String(reason))); }, []);
  async function action(work: () => Promise<void>) {
    setBusy(true); setError(''); setMessage('');
    try { await work(); } catch (reason) { setError(String(reason)); } finally { setBusy(false); }
  }
  return <>
    <Products />
    {error && <p role="alert" className="error">{error}</p>}
    {message && <p role="status" className="message">{message}</p>}
      {development && <details className="panel settings"><summary>Gelişmiş: ürün kaynak bağlantıları</summary>
        {settings ? <form onSubmit={event => { event.preventDefault(); void action(async () => { await saveSettings(settings); setMessage('Yerel ürün bağlantıları kaydedildi.'); }); }}>
          {(['python', 'orchestrator_root', 'brain_root', 'data_root'] as const).filter(key => key in settings).map(key => <label key={key}>{({ python: 'Python executable', orchestrator_root: 'Orchestrator kaynak dizini', brain_root: 'Brain kaynak dizini', data_root: 'Orchestrator bağımsız DataRoot' } as Record<string, string>)[key]}<input value={settings[key as keyof Settings]} onChange={e => setSettings({ ...settings, [key]: e.target.value })} required /></label>)}
          <button disabled={busy}>Kaydet</button></form> : <p>Ayarlar masaüstü uygulamasında kullanılabilir.</p>}
        <p className="muted">Hub'dan kurulan ürünler yönetilen kopyalarından çalışır ve bu kaynak klasörlerini yok sayar; kaynak klasörleri yalnızca ürün Hub'dan kurulu değilken kullanılır. İmzalı uzak katalog ve AI araç entegrasyonlarının uygulanması henüz yoktur.</p>
      </details>}
  </>;
}
