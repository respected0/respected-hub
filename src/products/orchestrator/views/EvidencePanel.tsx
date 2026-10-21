import { useCallback, useRef, useState } from 'react';
import { engine, native, products } from '../../../shared/api.ts';
import type { Run } from '../../../shared/api.ts';
import { campaignStatus } from '../campaigns.ts';
import { evidenceFor } from '../evidenceModel.ts';
import type { Evidence, Verdict } from '../evidenceModel.ts';
import './evidence.css';

function errorText(reason: unknown): string {
  return reason instanceof Error ? reason.message : String(reason);
}

function verdictLabel(verdict: Verdict): string {
  switch (verdict) {
    case 'approved': return 'Onaylandı';
    case 'needs_changes': return 'Değişiklik gerekli';
    case 'failed': return 'Başarısız';
    case 'human_approval': return 'Elle onay gerekli';
    case 'unknown': return 'Belirsiz';
  }
}

function verdictKind(verdict: Verdict): string {
  if (verdict === 'approved') return 'ok';
  if (verdict === 'failed') return 'error';
  return 'warn';
}

function EvidenceCard({ evidence }: { evidence: Evidence }) {
  return <article className="panel-glass evidence-card" aria-label="Kanıt özeti">
    <div className="section-head">
      <div>
        <h2>{evidence.headline}</h2>
        <p className="muted">{evidence.hasPatch ? 'Bu çalışmada uygulanacak bir patch var.' : 'Uygulanacak patch görünmüyor.'}</p>
      </div>
      <span className={`badge-status ${verdictKind(evidence.verdict)}`}>{verdictLabel(evidence.verdict)}</span>
    </div>

    <section aria-label="Kanıt">
      <h3>Kanıt</h3>
      {evidence.evidence.length === 0
        ? <p className="muted">Motor ek kanıt vermedi.</p>
        : <ul className="evidence-list">{evidence.evidence.map((item, index) => <li key={index}>
            <strong>{item.label}</strong><span>{item.text}</span>
          </li>)}</ul>}
    </section>

    <section aria-label="İncelemeler">
      <h3>İncelemeler</h3>
      {evidence.reviews.length === 0
        ? <p className="muted">İnceleme kaydı yok.</p>
        : <table className="evidence-reviews">
            <thead><tr><th>Aşama</th><th>Profil</th><th>Karar</th><th>Gerekçe</th></tr></thead>
            <tbody>{evidence.reviews.map((review, index) => <tr key={index}>
              <td>{review.stage}</td><td>{review.profile}</td><td>{review.decision}</td><td>{review.reason || '—'}</td>
            </tr>)}</tbody>
          </table>}
    </section>

    <section aria-label="Sonraki adım">
      <h3>Sonraki adım</h3>
      <pre className="evidence-next" tabIndex={0}>{evidence.nextStepText}</pre>
    </section>
  </article>;
}

export default function EvidencePanel({ runs, campaignIds }: { runs: Run[]; campaignIds: string[] }) {
  const [selectedRun, setSelectedRun] = useState('');
  const [selectedCampaign, setSelectedCampaign] = useState('');
  const [evidence, setEvidence] = useState<Evidence | null>(null);
  const [state, setState] = useState<'idle' | 'loading' | 'ready' | 'failed'>('idle');
  const [error, setError] = useState('');
  const requestRef = useRef(0);

  const load = useCallback(async (kind: 'run' | 'campaign', id: string) => {
    const token = ++requestRef.current;
    setState('loading'); setError(''); setEvidence(null);
    try {
      const cli = (await products().catch(() => null))?.orchestrator_cli ?? undefined;
      const result = kind === 'run'
        ? evidenceFor(await engine<unknown>('inspect', id), null, id, cli)
        : evidenceFor(null, await campaignStatus(id), '', cli);
      if (requestRef.current !== token) return;
      setEvidence(result); setState('ready');
    } catch (reason) {
      if (requestRef.current !== token) return;
      setState('failed'); setError(errorText(reason));
    }
  }, []);

  if (!native) {
    return <div className="notice warning">Çalışma kanıtlarını ve inceleme kararlarını görmek için Tauri masaüstü uygulaması gerekli. Tarayıcı önizlemesinde gerçek motor verisi gösterilmez.</div>;
  }

  const statusText = state === 'loading' ? 'Kanıt yükleniyor…'
    : state === 'failed' ? error
    : evidence ? evidence.headline
    : 'Bir çalışma veya kampanya seçin.';

  return <section className="page evidence-view" aria-label="İnceleme ve kanıtlar">
    <div className="section-head">
      <div>
        <span className="eyebrow">İnceleme</span>
        <h1>Kanıt ve onaylar</h1>
        <p className="muted">Motorun verdiği kanıtlar ve inceleme kararları burada gösterilir. Hub onay vermez ve patch uygulamaz; gereken komut bir terminalde sizin tarafınızdan çalıştırılır.</p>
      </div>
    </div>

    <div className="evidence-choices">
      <label htmlFor="evidence-run">Çalışma seç
        <select id="evidence-run" value={selectedRun} disabled={state === 'loading'}
          onChange={event => { const id = event.target.value; setSelectedRun(id); setSelectedCampaign(''); if (id) void load('run', id); }}>
          <option value="">— Seçin —</option>
          {runs.map(run => <option key={run.run_id} value={run.run_id}>{run.run_id} · {run.status}</option>)}
        </select>
      </label>
      <label htmlFor="evidence-campaign">Kampanya seç
        <select id="evidence-campaign" value={selectedCampaign} disabled={state === 'loading'}
          onChange={event => { const id = event.target.value; setSelectedCampaign(id); setSelectedRun(''); if (id) void load('campaign', id); }}>
          <option value="">— Seçin —</option>
          {campaignIds.map(id => <option key={id} value={id}>{id}</option>)}
        </select>
      </label>
    </div>

    <p className={state === 'failed' ? 'error' : 'muted'} role="status" aria-live="polite">{statusText}</p>

    {evidence && state === 'ready' && <EvidenceCard evidence={evidence} />}
  </section>;
}