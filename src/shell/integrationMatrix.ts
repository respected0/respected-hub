import type { AssetKind, ClientId, Owner, ResolvedEntry, ResolutionState, Scope } from '../shared/integrationsTypes.ts';

export const INTEGRATION_KINDS: AssetKind[] = ['mcp', 'skill', 'plugin'];
export const INTEGRATION_CLIENT_ORDER: ClientId[] = ['claude', 'codex', 'cursor', 'gemini', 'open_code'];

export const INTEGRATION_KIND_LABEL: Record<AssetKind, string> = {
  mcp: 'MCP', skill: 'Skill', plugin: 'Plugin', hook: 'Hook', command: 'Komut', agent: 'Ajan',
};
export const INTEGRATION_CLIENT_LABEL: Record<ClientId, string> = {
  claude: 'Claude Code', codex: 'Codex', cursor: 'Cursor', gemini: 'Gemini CLI', open_code: 'OpenCode', unknown: 'Bilinmeyen',
};
export const INTEGRATION_SCOPE_LABEL: Record<Scope, string> = { global: 'Global', project: 'Proje' };
export const INTEGRATION_STATE_LABEL: Record<ResolutionState, string> = {
  effective: 'Etkin', overridden: 'Gölgelendi', disabled: 'Devre dışı', unknown: 'Kaynağı okunamadı', untrusted: 'Güvenilmez',
};

export interface IntegrationFilter {
  query?: string;
  kind?: AssetKind | 'all';
  client?: ClientId | 'all';
  scope?: Scope | 'all';
}
export interface IntegrationGroup { key: string; name: string; kind: AssetKind; entries: ResolvedEntry[] }

const fold = (value: string) => value.toLocaleLowerCase('tr');

export function ownerLabel(owner: Owner): string {
  if (owner.kind === 'user') return 'Kullanıcı';
  if (owner.kind === 'product') return `${owner.product} ürünü`;
  if (owner.kind === 'catalog') return 'Katalog';
  return 'Bilinmiyor';
}

/** Local filter. A duplicate binding is kept: the matrix shows every observed source. */
export function filterIntegrations(entries: ResolvedEntry[], filter: IntegrationFilter): ResolvedEntry[] {
  const query = fold(filter.query?.trim() ?? '');
  const kind = filter.kind ?? 'all';
  const client = filter.client ?? 'all';
  const scope = filter.scope ?? 'all';
  return entries.filter(entry =>
    (kind === 'all' || entry.kind === kind)
    && (client === 'all' || entry.client === client)
    && (scope === 'all' || entry.scope === scope)
    && (!query || [entry.name, entry.source_path, entry.project?.id ?? '', entry.project?.root ?? '',
      INTEGRATION_CLIENT_LABEL[entry.client], INTEGRATION_KIND_LABEL[entry.kind], INTEGRATION_STATE_LABEL[entry.state], ownerLabel(entry.owner)]
      .some(value => fold(value).includes(query))));
}

/** One card per asset; every client and scope keeps its own row inside it. */
export function groupIntegrations(entries: ResolvedEntry[]): IntegrationGroup[] {
  const groups: IntegrationGroup[] = [];
  const byKey = new Map<string, IntegrationGroup>();
  for (const entry of entries) {
    let group = byKey.get(entry.asset_key);
    if (!group) {
      group = { key: entry.asset_key, name: entry.name, kind: entry.kind, entries: [] };
      byKey.set(entry.asset_key, group);
      groups.push(group);
    }
    group.entries.push(entry);
  }
  return groups;
}
