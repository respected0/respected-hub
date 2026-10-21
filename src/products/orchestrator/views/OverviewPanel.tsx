import { native, type Run } from '../../../shared/api.ts';
import { deriveOverview, type RunGroup } from '../overviewModel.ts';
import './overview.css';

interface OverviewPanelProps {
  runs: Run[];
  workers: Record<string, string[] | null> | null;
  onOpenTasks?: () => void;
  onOpenRun?: (id: string) => void;
}

const COUNT_LABELS: { key: RunGroup; label: string }[] = [
  { key: 'active', label: 'Sürüyor' },
  { key: 'succeeded', label: 'Tamamlandı' },
  { key: 'failed', label: 'Başarısız' },
  { key: 'other', label: 'Diğer' },
];

// Orchestrator workspace overview: real engine run counts, latest runs and
// worker CLI availability. No timers here; the shell refreshes and passes data.
export default function OverviewPanel({ runs, workers, onOpenTasks, onOpenRun }: OverviewPanelProps) {
  if (!native) {
    return <div className="notice warning">Genel bakış için Tauri masaüstü uygulaması gerekli. Tarayıcı önizlemesinde gerçek motor verisi gösterilmez.</div>;
  }

  const model = deriveOverview({ workers }, runs);
  const { runCounts } = model;
  const total = runCounts.active + runCounts.succeeded + runCounts.failed + runCounts.other;

  return <section aria-label="Genel bakış" className="overview-view">
    <div className="panel-glass">
      <h2>Görev durumu</h2>
      <ul className="overview-counts" aria-label="Görev sayıları">
        {COUNT_LABELS.map(({ key, label }) => <li key={key}><strong>{runCounts[key]}</strong><span>{label}</span></li>)}
      </ul>
      {total === 0 && <p className="muted" role="status" aria-live="polite">Henüz çalıştırma yok.</p>}
    </div>

    <div className="panel-glass">
      <h2>Son çalıştırmalar</h2>
      {model.latestRuns.length > 0
        ? <ul className="overview-runs" aria-label="Son çalıştırmalar">
            {model.latestRuns.map((run) => <li key={run.run_id}>
              <div>
                <span className="muted">{run.request?.task ?? run.run_id}</span>
                <span className={`status ${run.status}`}>{run.status}</span>
              </div>
              {onOpenRun && <button className="cta" onClick={() => onOpenRun(run.run_id)}>Görevi aç</button>}
            </li>)}
          </ul>
        : <p className="muted" role="status" aria-live="polite">Henüz çalıştırma yok.</p>}
      {onOpenTasks && <button className="cta" onClick={onOpenTasks}>Tüm görevler</button>}
    </div>

    <div className="panel-glass">
      <h2>Worker CLI&apos;lar</h2>
      {model.workersAvailable.length + model.workersMissing.length > 0
        ? <ul className="capabilities" aria-label="Worker CLI durumu">
            {model.workersAvailable.map((name) => <li key={name}>{name}</li>)}
            {model.workersMissing.map((name) => <li key={name} className="off">{name} · bulunamadı</li>)}
          </ul>
        : <p className="muted" role="status" aria-live="polite">Worker bilgisi alınamadı.</p>}
    </div>

    {model.warnings.length > 0 &&
      <ul className="overview-warnings notice warning" role="status" aria-live="polite" aria-label="Uyarılar">
        {model.warnings.map((warning) => <li key={warning}>{warning}</li>)}
      </ul>}
  </section>;
}