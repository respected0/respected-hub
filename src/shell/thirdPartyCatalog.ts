import type { CatalogEntry, CatalogKind, ClientId } from '../shared/integrationsTypes.ts';

export const CATALOG_KIND_LABEL: Record<CatalogKind, string> = {
  mcp: 'MCP', skill: 'Skill', plugin: 'Plugin', rule: 'Kural', other: 'Diğer',
};

export const CATALOG_CLIENT_LABEL: Partial<Record<ClientId, string>> = {
  claude: 'Claude Code', codex: 'Codex', cursor: 'Cursor', gemini: 'Gemini CLI', open_code: 'OpenCode',
};

export interface CatalogFilter {
  query?: string;
  kind?: CatalogKind | 'all';
  client?: ClientId | 'all';
  category?: string | 'all';
}

const fold = (value: string) => value.toLocaleLowerCase('tr');

/** Categories in the catalog, sorted for Turkish readers. */
export function catalogCategories(entries: CatalogEntry[]): string[] {
  return [...new Set(entries.map(entry => entry.category))].sort((a, b) => a.localeCompare(b, 'tr'));
}

/** Local-only filter: never touches the network, so it also works offline. */
export function filterCatalog(entries: CatalogEntry[], filter: CatalogFilter): CatalogEntry[] {
  const query = fold(filter.query?.trim() ?? '');
  const kind = filter.kind ?? 'all';
  const client = filter.client ?? 'all';
  const category = filter.category ?? 'all';
  return entries.filter(entry =>
    (kind === 'all' || entry.type === kind)
    && (client === 'all' || entry.supported_clients.includes(client))
    && (category === 'all' || entry.category === category)
    && (!query || [entry.id, entry.name, entry.description, entry.category].some(value => fold(value).includes(query))));
}