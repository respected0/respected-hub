import type { IntegrationPlan } from '../../../shell/lifecycle.ts';

const KIND: Record<string, string> = { file: 'Dosya', mcp: 'MCP kaydı', task: 'Zamanlanmış görev', shortcut: 'Kısayol', registry: 'Kayıt' };
const ACTION: Record<string, string> = { create: 'Oluşturulur', modify: 'Değiştirilir', restore: 'Eski haline döner', unchanged: 'Aynı' };

// The product's read-only plan: what is found, exactly what would change, and what is never touched.
export default function IntegrationPreview({ plan, hideUnchanged = false }: { plan: IntegrationPlan; hideUnchanged?: boolean }) {
  const changes = plan.changes.filter(change => !(hideUnchanged && change.action === 'unchanged'));
  const found = plan.discovered.filter(item => item.present);
  return <section aria-label="Entegrasyon planı"><h3>Değişecekler</h3>
    {changes.length === 0 ? <p className="muted">Hiçbir AI aracı yapılandırması değişmeyecek.</p>
      : <ul className="plan-changes">{changes.map(change => <li key={`${change.kind}:${change.target}`}><span className={`pill ${change.action}`}>{ACTION[change.action] ?? change.action}</span> <small>{KIND[change.kind] ?? change.kind}</small> <span className="mono">{change.target}</span></li>)}</ul>}
    {plan.external_effects.map(effect => <p className="muted" key={effect}>Ayrıca değişir: {effect}</p>)}
    <details><summary>Algılanan AI araç yapılandırmaları ({found.length})</summary>
      {found.length === 0 ? <p className="muted">Desteklenen araçlardan hiçbirinin yapılandırması bulunamadı.</p> : <ul className="plan-changes">{found.map(item => <li key={item.client_config}><span className="mono">{item.client_config}</span></li>)}</ul>}
      <p className="muted">Yalnızca bu planda adı geçen dosyalardaki Respected kayıtları yönetilir; listede olmayan veya desteklenmeyen araçlara hiç dokunulmaz.</p></details>
    <p className="muted">Bu önizleme hiçbir dosya yazmaz; onaylamadan önce ne değişeceğini gösterir.</p></section>;
}
