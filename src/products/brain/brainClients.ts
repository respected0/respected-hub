// AI clients Brain can connect to. `hooks`: captures sessions and gets the memory rules/skills (Sources);
// `mcp`: gets the memory MCP server entry (Permissions). The product validates every id again.
interface ClientDef { id: string; label: string; hooks: boolean; mcp: boolean; }
export const CLIENTS: ClientDef[] = [
  { id: 'claude', label: 'Claude Code', hooks: true, mcp: true },
  { id: 'codex', label: 'Codex', hooks: true, mcp: false },
  { id: 'gemini', label: 'Gemini CLI', hooks: true, mcp: false },
  { id: 'antigravity', label: 'Antigravity', hooks: true, mcp: true },
  { id: 'cursor', label: 'Cursor', hooks: true, mcp: true },
  { id: 'windsurf', label: 'Windsurf', hooks: false, mcp: true },
  { id: 'claude-desktop', label: 'Claude Desktop', hooks: false, mcp: true },
  { id: 'vscode-agents', label: 'Cline / Roo (VS Code)', hooks: false, mcp: true },
];

export interface Selection { sources: string[]; mcp: string[]; schedule: boolean; shortcut: boolean; }
export const EMPTY_SELECTION: Selection = { sources: [], mcp: [], schedule: false, shortcut: false };

const ids = (kind: 'hooks' | 'mcp') => CLIENTS.filter(client => client[kind]).map(client => client.id);

/** The `desired` object the product expects: flags follow the per-client lists. */
export function toDesired(selection: Selection): Record<string, unknown> {
  return { global: selection.sources.length > 0, mcp: selection.mcp.length > 0, schedule: selection.schedule, shortcut: selection.shortcut,
    clients: [...selection.sources].sort(), mcp_clients: [...selection.mcp].sort() };
}

/** Selection from the product's stored integrations (`clients` absent means "every client", the pre-selection behaviour). */
export function fromCurrent(current: Record<string, unknown>): Selection {
  const list = (flag: string, key: string, kind: 'hooks' | 'mcp'): string[] => {
    if (current[flag] !== true) return [];
    const stored = current[key] ?? (key === 'mcp_clients' ? current.clients : undefined);
    return Array.isArray(stored) ? stored.filter((id): id is string => typeof id === 'string' && ids(kind).includes(id)) : ids(kind);
  };
  return { sources: list('global', 'clients', 'hooks'), mcp: list('mcp', 'mcp_clients', 'mcp'), schedule: current.schedule === true, shortcut: current.shortcut === true };
}

/** The same choice as Brain settings (`integrations.*`), what its installer (`/SET=`) and `settings.apply` take. */
export function toSettings(selection: Selection): Record<string, boolean | string> {
  return { 'integrations.global': selection.sources.length > 0, 'integrations.clients': [...selection.sources].sort().join(','),
    'integrations.mcp': selection.mcp.length > 0, 'integrations.mcp_clients': [...selection.mcp].sort().join(','),
    'integrations.schedule': selection.schedule, 'integrations.shortcut': selection.shortcut };
}

/** Selection from Brain's settings readback (an empty client list means "every client"). */
export function fromSettings(settings: Record<string, unknown>): Selection {
  const list = (value: unknown) => typeof value === 'string' && value ? value.split(',').filter(Boolean) : undefined;
  return fromCurrent({ global: settings['integrations.global'], mcp: settings['integrations.mcp'], schedule: settings['integrations.schedule'],
    shortcut: settings['integrations.shortcut'], clients: list(settings['integrations.clients']), mcp_clients: list(settings['integrations.mcp_clients']) });
}

export const toggle = (list: string[], id: string, on: boolean) => on ? [...new Set([...list, id])] : list.filter(item => item !== id);
const clientLabel = (id: string) => CLIENTS.find(client => client.id === id)?.label ?? id;
export const summarize = (list: string[]) => list.length ? list.map(clientLabel).join(', ') : 'Hiçbiri';
