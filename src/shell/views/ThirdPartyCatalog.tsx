import { useEffect, useMemo, useState } from 'react';
import { refreshThirdPartyCatalog, thirdPartyCatalog } from '../../shared/api.ts';
import type { CatalogEntry, CatalogKind, ClientId } from '../../shared/integrationsTypes.ts';
import { CATALOG_CLIENT_LABEL, CATALOG_KIND_LABEL, catalogCategories, filterCatalog } from '../thirdPartyCatalog.ts';

// Read-only Discover catalog. Links open GitHub; this release never installs third-party content.
export default function ThirdPartyCatalog() {
  const [entries, setEntries] = useState<CatalogEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [query, setQuery] = useState('');
  const [kind, setKind] = useState<CatalogKind | 'all'>('all');
  const [client, setClient] = useState<ClientId | 'all'>('all');
  const [category, setCategory] = useState<string | 'all'>('all');

  useEffect(() => {
    let live = true;
    thirdPartyCatalog().then(value => { if (live) setEntries(value); },
      reason => { if (live) setError(reason instanceof Error ? reason.message : 'Katalog okunamadı.'); });
    return () => { live = false; };
  }, []);

  const categories = useMemo(() => catalogCategories(entries ?? []), [entries]);
  const shown = useMemo(() => filterCatalog(entries ?? [], { query, kind, client, category }), [entries, query, kind, client, category]);

  async function refresh() {
    setRefreshing(true);
    setError(null);
    try { setEntries(await refreshThirdPartyCatalog()); }
    catch (reason) { setError(reason instanceof Error ? reason.message : 'Katalog yenilenemedi.'); }
    finally { setRefreshing(false); }
  }

  return <section aria-label="Üçüncü taraf kataloğu">
    <div className="section-head" style={{ marginBottom: 'var(--s-4)' }}>
      <div><h2>MCP, Skill ve Plugin'ler</h2><p className="meta">Bağlantılar GitHub'da açılır. Hub bu sürümde üçüncü taraf içerik kurmaz. Katalog imzası yalnızca metadata bütünlüğünü doğrular; bağlantılı kaynak kodun güvenli olduğunu garanti etmez.</p></div>
      <div className="catalog-clients">
        {entries && <p className="meta">{shown.length} / {entries.length} kayıt</p>}
        <button className="button" onClick={refresh} disabled={refreshing}>{refreshing ? 'Yenileniyor…' : 'Kataloğu yenile'}</button>
      </div>
    </div>
    <div className="catalog-toolbar">
      <input type="search" value={query} onChange={event => setQuery(event.target.value)} placeholder="Ara: MCP, skill, plugin…" aria-label="Katalogda ara" />
      <select value={kind} onChange={event => setKind(event.target.value as CatalogKind | 'all')} aria-label="Tür">
        <option value="all">Tüm türler</option>{Object.entries(CATALOG_KIND_LABEL).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
      </select>
      <select value={client} onChange={event => setClient(event.target.value as ClientId | 'all')} aria-label="İstemci">
        <option value="all">Tüm istemciler</option>{Object.entries(CATALOG_CLIENT_LABEL).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
      </select>
      <select value={category} onChange={event => setCategory(event.target.value)} aria-label="Kategori">
        <option value="all">Tüm kategoriler</option>{categories.map(value => <option key={value} value={value}>{value}</option>)}
      </select>
    </div>
    {error ? <p className="catalog-empty">{error}</p>
      : !entries ? <p className="catalog-empty">Katalog yükleniyor…</p>
      : shown.length === 0 ? <p className="catalog-empty">Bu filtreye uyan kayıt yok.</p>
      : <div className="catalog-grid">{shown.map(entry => <article key={entry.id} className="catalog-card">
        <div className="catalog-clients"><span className="badge tone-live">{CATALOG_KIND_LABEL[entry.type]}</span><span className="badge tone-idle">{entry.category}</span>{entry.license && <span className="badge tone-idle">{entry.license}</span>}</div>
        <h3>{entry.name}</h3><p>{entry.description}</p>
        <div className="catalog-card-foot">
          <div className="catalog-clients">{entry.supported_clients.map(item => <span key={item} className="badge tone-idle">{CATALOG_CLIENT_LABEL[item] ?? item}</span>)}</div>
          <a className="button" href={entry.source_url} target="_blank" rel="noreferrer">GitHub'da aç</a>
        </div>
      </article>)}</div>}
  </section>;
}
