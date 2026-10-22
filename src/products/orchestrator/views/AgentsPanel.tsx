import { native } from '../../../shared/api.ts';
import { deriveAgentsModel } from '../agentsModel.ts';
import './agents.css';

interface AgentsPanelProps {
  workers: Record<string, string[] | null> | null;
  settings: Record<string, unknown> | null;
  loading: boolean;
  error?: string;
}

// Orchestrator workspace: engine-defined agent roles vs model/provider profiles.
// Read-only; real engine data only, honest notice outside Tauri.
export default function AgentsPanel({ workers, settings, loading, error }: AgentsPanelProps) {
  if (!native) {
    return <div className="notice warning">Ajan ve model bilgisini görmek için Tauri masaüstü uygulaması gerekli. Tarayıcı önizlemesinde gerçek motor verisi gösterilmez.</div>;
  }

  const model = deriveAgentsModel(workers, settings);

  return <div aria-label="Ajanlar ve modeller" className="ws-stack">
    {error && <p className="error" role="alert">{error}</p>}

    <div className="panel-glass">
      <h2>Ajan rolleri</h2>
      <p className="muted">Motorun tanımladığı roller; burada düzenlenemez.</p>
      <ul className="agent-roles" aria-label="Ajan rolleri">
        {model.roles.map((role) => <li key={role.id}><strong>{role.name}</strong><span>{role.description}</span></li>)}
      </ul>
    </div>

    <div className="panel-glass">
      <h2>Model / sağlayıcı profilleri</h2>
      {loading
        ? <p className="muted" role="status" aria-live="polite">Profiller yükleniyor…</p>
        : model.profiles.length > 0
          ? <ul className="agent-profiles" aria-label="Model ve sağlayıcı profilleri">
              {model.profiles.map((profile) => <li key={profile.key}><strong>{profile.label}</strong><span>{profile.value}</span></li>)}
            </ul>
          : <p className="muted" role="status" aria-live="polite">Motor bu sürümde model/sağlayıcı profili bildirmiyor.</p>}
    </div>

    <div className="panel-glass">
      <h2>Worker CLI&apos;lar</h2>
      {model.workers.length > 0
        ? <ul className="capabilities" aria-label="Worker CLI durumu">
            {model.workers.map((worker) => <li key={worker.name} className={worker.available ? '' : 'off'}>{worker.name}{worker.available ? '' : ' · bulunamadı'}</li>)}
          </ul>
        : <p className="muted" role="status" aria-live="polite">Worker bilgisi alınamadı.</p>}
    </div>

    <p className="muted" role="note">{model.note}</p>
  </div>;
}