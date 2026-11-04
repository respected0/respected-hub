// Mirror of src-tauri/src/integrations/model.rs. Keep both sides in sync.
export type AssetKind = 'mcp' | 'skill' | 'plugin' | 'hook' | 'command' | 'agent';
export type ClientId = 'claude' | 'codex' | 'cursor' | 'gemini' | 'opencode' | 'unknown';
export type Scope = 'global' | 'project';
export type Owner =
  | { kind: 'user' }
  | { kind: 'product'; product: string }
  | { kind: 'catalog' }
  | { kind: 'unknown' };
export type SourceFormat = 'json' | 'jsonc' | 'toml' | 'skill_directory' | 'manifest' | 'unknown';
export type SourceIntegrity = 'ok' | 'missing' | 'broken_link' | 'unreadable' | 'unsupported';
export type ObservedStatus =
  | 'unknown' | 'installed' | 'configured' | 'enabled' | 'connection_verified' | 'usage_proven';

export interface Project { id: string; root: string }
export interface Source {
  path: string; path_id: string; fingerprint: string | null; owner: Owner;
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
export interface Capability { client: ClientId; read: boolean; plan: boolean; apply: boolean; undo: boolean }
export interface ScanIssue { code: string; message: string; source_path: string | null }
export interface Inventory {
  assets: Asset[]; bindings: Binding[]; clients: ClientState[];
  capabilities: Capability[]; issues: ScanIssue[];
}
