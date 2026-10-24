import { useEffect, useState } from 'react';
import notes from '../../../common/release/UPDATE_NOTES.md?raw';

const KEY = 'hub.whatsnew.seen';
const items = notes.split('\n').filter(line => line.startsWith('- ')).map(line => line.slice(2).trim());

function seen(): string | null { try { return localStorage.getItem(KEY); } catch { return null; } }
function remember(version: string) { try { localStorage.setItem(KEY, version); } catch { /* shown again next start */ } }

/** Shown once after the Hub changed version. A first start only records the version: nothing was updated. */
export default function WhatsNew({ version }: { version: string }) {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    const last = seen();
    if (last === version) return;
    remember(version);
    setOpen(last !== null && items.length > 0);
  }, [version]);
  if (!open) return null;
  return <div className="whatsnew-scrim" onClick={() => setOpen(false)}>
    <div className="whatsnew" role="dialog" aria-modal="true" aria-labelledby="whatsnew-title" onClick={event => event.stopPropagation()}>
      <h2 id="whatsnew-title">Respected Hub {version} sürümüne güncellendi</h2>
      <ul>{items.map(item => <li key={item}>{item}</li>)}</ul>
      <button className="cta" autoFocus onClick={() => setOpen(false)}>Tamam</button>
    </div>
  </div>;
}
