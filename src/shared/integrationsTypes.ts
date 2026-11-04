// Mirror of src-tauri/src/integrations/model.rs. Keep both sides in sync.
export type AssetKind = 'mcp' | 'skill' | 'plugin' | 'hook' | 'command' | 'agent';
export type ClientId = 'claude' | 'codex' | 'cursor' | 'gemini' | 'open_code' | 'unknown';
export type Scope = 'global' | 'project';
export type Owner =
  | { kind: 'user' }
  | { kind: 'product'; product: string }
  | { kind: 'catalog' }
  | { kind: 'unknown' };
export type SourceFormat = 'json' | 'jsonc' | 'toml' | 'skill_directory' | 'markdown' | 'manifest' | 'unknown';
export type SourceIntegrity = 'ok' | 'missing' | 'broken_link' | 'unreadable' | 'unsupported';
export type ObservedStatus =
  | 'unknown' | 'installed' | 'configured' | 'enabled' | 'connection_verified' | 'usage_proven';

export interface Project { id: string; root: string }
export interface Source {
  path: string; path_id: string; fingerprint: string | null; owner: Owner;
  symlink_target?: string | null;
  format: SourceFormat; integrity: SourceIntegrity;
}
export interface Asset { key: string; name: string; kind: AssetKind; identity: string; description: string | null }
export interface Binding {
  asset_key: string; client: ClientId; scope: Scope; project: Project | null;
  source: Source; status: ObservedStatus; enabled: boolean | null; managed: boolean;
  parent_asset_key: string | null;
}
export interface ClientState {
  client: ClientId; installed: boolean; configured: boolean; status: ObservedStatus;
  version: string | null; executable: string | null; config_paths: string[];
}
export interface Capability { client: ClientId; read: boolean; plan: boolean; apply: boolean; undo: boolean; enable: boolean; disable: boolean; toggle_reason: string | null }
export interface ScanIssue { code: string; message: string; source_path: string | null }
export interface Inventory {
  assets: Asset[]; bindings: Binding[]; clients: ClientState[];
  capabilities: Capability[]; issues: ScanIssue[];
}

export type ResolutionState = 'effective' | 'overridden' | 'disabled' | 'unknown' | 'untrusted';
export interface ResolvedEntry {
  asset_key: string; name: string; kind: AssetKind; client: ClientId; scope: Scope;
  project: Project | null; source_path: string; configured: boolean; effective: boolean;
  enabled: boolean | null; managed: boolean; owner: Owner; state: ResolutionState;
}
export interface Resolution { entries: ResolvedEntry[]; issues: ScanIssue[]; untrusted: ScanIssue[] }
export interface InventoryResult { read_only: boolean; inventory: Inventory; resolution: Resolution }

// Mirror of src-tauri/src/integrations/api.rs.
export type IntegrationAction = 'inventory' | 'clients' | 'catalog';
export interface IntegrationRequest { action: IntegrationAction }
export interface ClientsResult { read_only: boolean; clients: ClientState[]; issues: ScanIssue[] }
export type CatalogKind = 'mcp' | 'skill' | 'plugin' | 'rule' | 'other';
export interface CatalogEntry {
  id: string; name: string; type: CatalogKind; description: string; category: string;
  source_url: string; supported_clients: ClientId[]; license: string | null; provenance: string | null;
}
export interface CatalogResult { read_only: boolean; entries: CatalogEntry[] }
export interface IntegrationReply {
  ok: boolean;
  result: InventoryResult | ClientsResult | CatalogResult | null;
  error: { code: string; message: string } | null;
}
