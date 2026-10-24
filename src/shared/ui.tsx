import hubMark from '../../common/brand/hub.svg';
import hubSmall from '../../common/brand/hub-small.svg';
import brainMark from '../../common/brand/brain.svg';
import brainSmall from '../../common/brand/brain-small.svg';
import orchestratorMark from '../../common/brand/orchestrator.svg';
import orchestratorSmall from '../../common/brand/orchestrator-small.svg';
// Shared presentational pieces: icon set, generative app emblems and plain-language status labels.
const PATHS: Record<string, string> = {
  home: 'M3 11l9-8 9 8v9a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z',
  apps: 'M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z',
  discover: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zm3.5 5.5l-2 5-5 2 2-5z',
  connections: 'M9 7H7a4 4 0 0 0 0 8h2M15 7h2a4 4 0 0 1 0 8h-2M8 11h8',
  activity: 'M3 12h4l3-8 4 16 3-8h4',
  settings: 'M4 7h10M18 7h2M4 17h2M10 17h10M14 4v6M6 14v6',
  refresh: 'M20 11a8 8 0 1 0-2.3 5.7M20 4v7h-7',
  arrow: 'M5 12h14M13 6l6 6-6 6',
  check: 'M5 12.5l4.5 4.5L19 7.5',
  minimize: 'M6 12h12',
  filter: 'M4 7h16M7 12h10M10 17h4',
  maximize: 'M6 6h12v12H6z',
  restore: 'M9 9V6h9v9h-3M6 9h9v9H6z',
  x: 'M6 6l12 12M18 6L6 18',
  info: 'M12 8h.01M11 12h1v5h1M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z',
  shield: 'M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z',
  download: 'M12 4v11M7 11l5 5 5-5M5 20h14',
  sync: 'M4 8h13l-3-3M20 16H7l3 3',
  wrench: 'M14.5 6a4 4 0 0 0 4.8 5.2L21 13l-2 2-1.8-1.7A4 4 0 0 1 12 8L8 12l-5 5 3 3 5-5',
  trash: 'M5 7h14M10 7V4h4v3M7 7l1 13h8l1-13',
};
export function Icon({ name }: { name: string }) {
  return <svg className={`ico ${name}`} viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={PATHS[name] ?? PATHS.apps} /></svg>;
}

// The product marks (common/brand, built by common/brand/make_brand.py). The small variants carry one bold shape instead of fine detail.
const MARKS: Record<string, { full: string; small: string }> = {
  hub: { full: hubMark, small: hubSmall },
  brain: { full: brainMark, small: brainSmall },
  orchestrator: { full: orchestratorMark, small: orchestratorSmall },
};
export function Emblem({ app, className = '', small = false }: { app: string; className?: string; small?: boolean }) {
  const mark = MARKS[app] ?? MARKS.hub;
  return <span className={`emblem-tile brand ${className}`} aria-hidden="true"><img src={small ? mark.small : mark.full} alt="" draggable={false} /></span>;
}
export function Weave() {
  return <svg className="weave" viewBox="0 0 400 160" preserveAspectRatio="xMidYMid slice" aria-hidden="true" fill="none" stroke="currentColor" style={{ color: 'var(--accent)' }}>
    {[0, 1, 2, 3, 4, 5].map(i => <path key={i} d={`M-20 ${20 + i * 26} C 90 ${-10 + i * 34}, 210 ${70 + i * 12}, 420 ${i * 24}`} strokeWidth="1" opacity={0.16 + i * 0.05} />)}
    <circle cx="330" cy="40" r="46" strokeWidth="1" opacity=".25" /><circle cx="330" cy="40" r="74" strokeWidth="1" opacity=".12" />
  </svg>;
}

const STATUS: Record<string, string> = {
  queued: 'Sırada', running: 'Çalışıyor', completed: 'Tamamlandı', failed: 'Başarısız',
  timed_out: 'Zaman aşımı', cancelled: 'İptal edildi', interrupted: 'Durduruldu', invalid: 'Geçersiz',
};
export const statusLabel = (status: string) => STATUS[status] ?? status;
export const statusTone = (status: string) => status === 'completed' ? 'ok' : ['failed', 'timed_out', 'invalid'].includes(status) ? 'error' : ['running', 'queued'].includes(status) ? 'live' : 'idle';
