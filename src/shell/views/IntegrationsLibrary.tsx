import { useEffect, useMemo, useState } from 'react';
import { thirdPartyInventory } from '../../shared/api.ts';
import type { AssetKind, ClientId, ClientState, DriftCode, InventoryResult, ResolutionState, Scope, Capability } from '../../shared/integrationsTypes.ts';
import {
  filterIntegrations, groupIntegrations, INTEGRATION_CLIENT_LABEL, INTEGRATION_CLIENT_ORDER,
  INTEGRATION_KINDS, INTEGRATION_KIND_LABEL, INTEGRATION_SCOPE_LABEL, INTEGRATION_STATE_LABEL, ownerLabel,
} from '../integrationMatrix.ts';

const STATE_TONE: Record<ResolutionState, string> = {
  effective: 'tone-ok', overridden: 'tone-idle', disabled: 'tone-idle', unknown: 'tone-error', untrusted: 'tone-error',
};

const DRIFT_LABEL: Record<DriftCode, string> = {
  source_changed: 'Kaynak değişti', duplicate_copy: 'Kopya çakışması', shadowed: 'Gölgeleniyor',
  broken_symlink: 'Bozuk symlink', orphan_target: 'Hedef silinmiş', target_missing: 'Hedef yok',
  owner_protected: 'Sahipli kayıt',
};

const clientStatus = (client: ClientState | undefined) =>
  client ? (client.configured ? 'ayar var' : client.installed ? 'kurulu' : 'bulunamadı') : 'bilinmiyor';

const ToggleBadge = ({ capability }: { capability: Capability | undefined }) => {
  const toggle = Boolean(capability?.enable && capability?.disable);
  return <span className={`badge ${toggle ? 'tone-ok' : 'tone-idle'}`}
    title={toggle ? 'Native enabled alanı güvenli biçimde değiştirilebilir.' : capability?.toggle_reason ?? 'Bu istemci için güvenli toggle yok.'}>
    {toggle ? 'Aç/kapa destekleniyor' : 'Toggle yok'}
  </span>;
};

export default function IntegrationsLibrary() {
  const [result, setResult] = useState<InventoryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [reload, setReload] = useState(0);
  const [query, setQuery] = useState('');
  const [kind, setKind] = useState<AssetKind | 'all'>('all');
  const [client, setClient] = useState<ClientId | 'all'>('all');
  const [scope, setScope] = useState<Scope | 'all'>('all');

  useEffect(() => {
    let live = true;
    setResult(null); setError(null);
    thirdPartyInventory().then(value => { if (live) setResult(value); },
      reason => { if (live) setError(reason instanceof Error ? reason.message : 'Envanter okunamadı.'); });
    return () => { live = false; };
  }, [reload]);

  const entries = useMemo(() => result?.resolution.entries ?? [], [result]);
  const groups = useMemo(() => groupIntegrations(filterIntegrations(entries, { query, kind, client, scope })),
    [entries, query, kind, client, scope]);
  const shown = groups.reduce((total, group) => total + group.entries.length, 0);

  return <section className="integration-library" aria-label="Kurulu entegrasyon matrisi" aria-busy={!result && !error}>
    <div className="section-head" style={{ marginBottom: 'var(--s-4)' }}>
      <div><h2>Kurulu entegrasyonlar</h2>
        <p className="meta">MCP, skill ve plugin'lerin hangi AI aracında, hangi kapsamda tanımlı olduğunu gösterir.</p></div>
      <span className="badge tone-idle">Salt okunur</span>
    </div>
    <div className="notice" role="note">
      <strong>Salt okunur.</strong> Hub bu sürümde AI aracı ayarlarını değiştirmez; plan, uygula ve geri al yoktur.
      "Etkin" yalnızca istemcinin kendi bağlamında kullanacağı kaydı anlatır; gerçekten çağrıldığını kanıtlamaz.
    </div>
    <div className="catalog-toolbar">
      <input type="search" value={query} onChange={event => setQuery(event.target.value)} placeholder="Ara: ad, kaynak yolu, proje…" aria-label="Entegrasyonlarda ara" />
      <select value={kind} onChange={event => setKind(event.target.value as AssetKind | 'all')} aria-label="Tür">
        <option value="all">Tüm türler</option>
        {INTEGRATION_KINDS.map(value => <option key={value} value={value}>{INTEGRATION_KIND_LABEL[value]}</option>)}
      </select>
      <select value={client} onChange={event => setClient(event.target.value as ClientId | 'all')} aria-label="İstemci">
        <option value="all">Tüm istemciler</option>
        {INTEGRATION_CLIENT_ORDER.map(value => <option key={value} value={value}>{INTEGRATION_CLIENT_LABEL[value]}</option>)}
      </select>
      <select value={scope} onChange={event => setScope(event.target.value as Scope | 'all')} aria-label="Kapsam">
        <option value="all">Tüm kapsamlar</option>
        <option value="global">Global</option><option value="project">Proje</option>
      </select>
    </div>
    {result && <>
      <div className="integration-clients" aria-label="İzlenen AI istemcileri">
        {INTEGRATION_CLIENT_ORDER.map(id => {
          const state = result.inventory.clients.find(item => item.client === id);
          return <span key={id} className={`badge ${state?.configured ? 'tone-live' : 'tone-idle'}`}>
            {INTEGRATION_CLIENT_LABEL[id]} · {clientStatus(state)}
          </span>;
        })}
      </div>
      <p className="meta integration-count" role="status">{shown} bağlantı · {groups.length} varlık</p>
      {result.resolution.issues.length > 0 && <details className="integration-issues">
        <summary>Okuma notları ({result.resolution.issues.length})</summary>
        <ul>{result.resolution.issues.map((issue, index) => <li key={`${issue.code}-${index}`}>
          <strong>{issue.code}</strong> {issue.message}{issue.source_path && <> <code>{issue.source_path}</code></>}
        </li>)}</ul>
      </details>}
      {result.resolution.drift.length > 0 && <details className="integration-issues" open>
        <summary>Drift ve çakışmalar ({result.resolution.drift.length})</summary>
        <ul>{result.resolution.drift.map((finding, index) => <li key={`${finding.code}-${index}`}>
          <strong>{DRIFT_LABEL[finding.code]}</strong> {finding.message}
          {finding.source_path && <> <code>{finding.source_path}</code></>}
        </li>)}</ul>
        <p className="meta">Bu analiz salt okunurdur; Hub hiçbir kaynağı otomatik düzeltmez.</p>
      </details>}
    </>}
    {error ? <div className="catalog-empty" role="alert"><p>{error}</p>
        <button type="button" className="button" onClick={() => setReload(value => value + 1)}>Yeniden dene</button></div>
      : !result ? <p className="catalog-empty">Entegrasyon envanteri okunuyor…</p>
      : groups.length === 0 ? <p className="catalog-empty">{entries.length === 0
          ? 'Kurulu MCP, skill veya plugin bulunamadı.'
          : 'Bu filtreye uyan kayıt yok.'}</p>
      : <div className="integration-groups">{groups.map(group => <article key={group.key} className="integration-group">
          <header className="integration-group-head">
            <span className="badge tone-live">{INTEGRATION_KIND_LABEL[group.kind]}</span>
            <h3>{group.name}</h3>
            <span className="meta">{group.entries.length} bağlantı</span>
          </header>
          <div className="integration-rows">{group.entries.map((entry, index) => <div key={`${group.key}-${index}`} className="integration-row">
            <div className="integration-where">
              <strong>{INTEGRATION_CLIENT_LABEL[entry.client]}</strong>
              <span className="muted">{INTEGRATION_SCOPE_LABEL[entry.scope]}{entry.project ? ` · ${entry.project.id}` : ''}</span>
              {entry.project && <span className="mono integration-root" title={entry.project.root}>{entry.project.root}</span>}
            </div>
            <div className="integration-state"><span className={`badge ${STATE_TONE[entry.state]}`}>{INTEGRATION_STATE_LABEL[entry.state]}</span>
              {entry.managed && <span className="badge tone-live">Hub yönetiyor</span>}<ToggleBadge capability={result.inventory.capabilities.find(item => item.client === entry.client)} /></div>
            <div className="integration-owner"><span>{ownerLabel(entry.owner)}</span>
              <small className="muted">{entry.managed ? 'Hub tarafından yazıldı' : 'Dış kaynak'}</small></div>
            <details className="integration-source"><summary>Kaynağı göster</summary><code>{entry.source_path}</code></details>
          </div>)}</div>
        </article>)}</div>}
  </section>;
}
