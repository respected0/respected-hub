import { useCallback, useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { addProject, listProjects, removeProject, thirdPartyInventory } from '../../shared/api.ts';
import type { InventoryResult, Project } from '../../shared/integrationsTypes.ts';
import { INTEGRATION_CLIENT_LABEL, INTEGRATION_CLIENT_ORDER, INTEGRATION_KIND_LABEL } from '../integrationMatrix.ts';
import { projectBindings, projectIssue } from '../projectScope.ts';

export default function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [result, setResult] = useState<InventoryResult | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    setError('');
    const [registered, inventory] = await Promise.all([listProjects(), thirdPartyInventory()]);
    setProjects(registered); setResult(inventory);
  }, []);

  useEffect(() => { load().catch(reason => setError(String(reason))); }, [load]);

  const change = async (work: Promise<Project[]>) => {
    setBusy(true); setError('');
    try {
      setProjects(await work);
      setResult(await thirdPartyInventory());
    } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)); }
    finally { setBusy(false); }
  };

  const add = async () => {
    const chosen = await open({ directory: true, multiple: false, title: 'Proje kökü seç' });
    if (typeof chosen === 'string') await change(addProject(chosen));
  };

  return <section className="project-scope" aria-label="Kayıtlı projeler ve etkin kapsam">
    <div className="section-head" style={{ marginBottom: 'var(--s-4)' }}>
      <div><h2>Projeler</h2>
        <p className="meta">Yalnızca burada açıkça seçtiğiniz proje kökleri taranır; Hub diski kendiliğinden taramaz.</p></div>
      <span className="badge tone-idle">Salt okunur</span>
    </div>
    <div className="notice" role="note"><strong>Salt okunur.</strong> Proje eklemek yalnızca Hub'ın kendi proje listesini değiştirir.
      Hiçbir AI aracı ayarı yazılmaz; etkin kayıtlar istemcinin kendi bağlamında okunur.</div>
    <div className="project-actions">
      <button type="button" className="button primary" disabled={busy} onClick={() => void add()}>Proje ekle</button>
      {error && <output className="toast tone-error" role="alert">{error}</output>}
    </div>
    {!result && !error ? <p className="catalog-empty">Projeler okunuyor…</p>
      : projects.length === 0 ? <p className="catalog-empty">Henüz kayıtlı proje yok. Proje ekle ile bir proje kökü seçin.</p>
      : <div className="project-grid">{projects.map(project => {
        const issue = result ? projectIssue(project, result.resolution.issues) : null;
        const bindings = result ? projectBindings(result.resolution, project.id) : [];
        return <article key={project.id} className={`project-card ${issue ? 'project-missing' : ''}`}>
          <header><div><h3>{project.root.split(/[\\/]/).filter(Boolean).pop() ?? project.id}</h3>
            <code className="project-root" title={project.root}>{project.root}</code></div>
            <button type="button" className="button" disabled={busy} onClick={() => void change(removeProject(project.id))}
              aria-label={`${project.root} projesini kaldır`}>Kaldır</button></header>
          {issue && <p className="project-warning" role="alert">{issue.message}</p>}
          <div className="project-clients">{INTEGRATION_CLIENT_ORDER.map(client => {
            const entries = bindings.filter(entry => entry.client === client);
            return <div key={client} className="project-client">
              <strong>{INTEGRATION_CLIENT_LABEL[client]}</strong>
              {entries.length === 0 ? <span className="muted">Etkin kayıt yok</span>
                : entries.map((entry, index) => <span key={`${entry.asset_key}-${index}`} className="badge tone-live"
                    title={entry.source_path}>{entry.name} · {INTEGRATION_KIND_LABEL[entry.kind]} · {entry.scope === 'project' ? 'proje' : 'global'}</span>)}
            </div>;
          })}</div>
        </article>;
      })}</div>}
  </section>;
}
