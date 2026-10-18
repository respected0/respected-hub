import { useEffect, useState } from 'react';
import { changedOnly, clientLabel, describeMcpError, initialDesired, mcpApply, mcpPlan } from '../mcpAccess.ts';
import type { McpPlan } from '../mcpAccess.ts';

const ACTION: Record<string, string> = { create: 'Eklenir', update: 'Güncellenir', remove: 'Kaldırılır' };

// Connect the Orchestrator to AI clients as an MCP server. Nothing is written before the user approves the exact plan.
export default function OrchestratorMcp() {
  const [status, setStatus] = useState<McpPlan | null>(null);
  const [initial, setInitial] = useState<Record<string, boolean>>({});
  const [desired, setDesired] = useState<Record<string, boolean>>({});
  const [plan, setPlan] = useState<McpPlan | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [done, setDone] = useState('');

  async function load() {
    const reply = await mcpPlan({});
    if (!reply.ok) { setError(describeMcpError(reply.error)); return; }
    const start = initialDesired(reply.result);
    setStatus(reply.result); setInitial(start); setDesired(start); setPlan(null);
  }
  useEffect(() => { void load(); }, []);

  const changes = changedOnly(desired, initial);
  useEffect(() => {
    let active = true;
    setPlan(null);
    if (Object.keys(changes).length === 0) return;
    void mcpPlan(changes).then(reply => { if (!active) return; if (reply.ok) { setPlan(reply.result); setError(''); } else setError(describeMcpError(reply.error)); });
    return () => { active = false; };
  }, [JSON.stringify(changes)]);

  async function apply() {
    if (!plan) return;
    setBusy(true); setError(''); setDone('');
    const reply = await mcpApply(changes, plan.plan_digest);
    setBusy(false);
    if (reply.ok) { setDone('Değişiklik uygulandı. Hub kapalıyken de çalışır; AI aracını yeniden başlatmanız gerekebilir.'); await load(); } else setError(describeMcpError(reply.error));
  }

  const refused = plan?.clients.filter(row => row.action === 'refuse') ?? [];
  const touched = plan?.clients.filter(row => ['create', 'update', 'remove'].includes(row.action)) ?? [];
  return <section className="panel-glass" aria-label="Orchestrator MCP bağlantıları">
    <div className="section-head"><h2>AI araçlarından kullan (MCP)</h2></div>
    <p className="muted">Orchestrator'ı Claude Code, Codex gibi araçlara MCP sunucusu olarak bağlar; Hub açık olmasa da çalışır. Yalnızca “respected-orchestrator” kaydı yönetilir, diğer ayarlarınız korunur.</p>
    {status?.clients.map(row => <label className="choice" key={row.client}><span><strong>{clientLabel(row.client)}</strong>
      <small>{row.detected ? row.config : 'Bu bilgisayarda bulunamadı'}{row.state === 'foreign' ? ' — aynı adlı başka bir kayıt var, değiştirilmeyecek' : ''}{row.state === 'modified' ? ' — kayıt sizin tarafınızdan değiştirilmiş, dokunulmayacak' : ''}</small></span>
      <input type="checkbox" role="switch" checked={desired[row.client] === true} disabled={busy || !row.detected || row.state === 'foreign' || row.state === 'modified'}
        onChange={event => setDesired({ ...desired, [row.client]: event.target.checked })} /></label>)}
    {touched.length > 0 && <div role="note"><p><strong>Onayınızla şu dosyalar değişecek</strong></p>
      <ul className="plan-changes">{touched.map(row => <li key={row.client}><span className={`pill ${row.action}`}>{ACTION[row.action]}</span> <span className="mono">{row.config}</span></li>)}</ul>
      <p className="muted">Yalnızca “respected-orchestrator” girdisi yazılır veya silinir; dosyanın geri kalanı olduğu gibi kalır.</p></div>}
    {refused.map(row => <p className="error" role="alert" key={row.client}>{clientLabel(row.client)}: {row.reason}</p>)}
    {error && <p className="error" role="alert">{error}</p>}
    {done && <p className="message" role="status">{done}</p>}
    <div className="actions"><button type="button" className="cta" disabled={busy || !plan || touched.length === 0 || refused.length > 0} onClick={() => void apply()}>{busy ? 'Uygulanıyor…' : 'Onayla ve uygula'}</button></div>
  </section>;
}
