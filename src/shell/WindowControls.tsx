import { useEffect, useState } from 'react';
import { Icon } from '../shared/ui.tsx';

// The window has no system title bar: the top navigation is the title bar, and these are its minimise / maximise / close buttons.
export default function WindowControls() {
  const [maximized, setMaximized] = useState(false);
  const win = () => import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow());
  useEffect(() => {
    let off: (() => void) | undefined;
    let cancelled = false;
    void win().then(async current => {
      const read = () => current.isMaximized().then(setMaximized).catch(() => undefined);
      await read();
      const stop = await current.onResized(() => void read());
      if (cancelled) stop(); else off = stop;
    }).catch(() => undefined);
    return () => { cancelled = true; off?.(); };
  }, []);
  const act = (run: (current: Awaited<ReturnType<typeof win>>) => Promise<void>) => () => void win().then(run).catch(() => undefined);
  return <div className="win-controls">
    <button type="button" aria-label="Küçült" title="Küçült" onClick={act(current => current.minimize())}><Icon name="minimize" /></button>
    <button type="button" aria-label={maximized ? 'Önceki boyut' : 'Büyüt'} title={maximized ? 'Önceki boyut' : 'Büyüt'} onClick={act(current => current.toggleMaximize())}><Icon name={maximized ? 'restore' : 'maximize'} /></button>
    <button type="button" className="close" aria-label="Kapat" title="Kapat" onClick={act(current => current.close())}><Icon name="x" /></button>
  </div>;
}
