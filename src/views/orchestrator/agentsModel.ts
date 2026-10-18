// Pure, testable model for the Orchestrator "agents vs models" panel.
// Real engine data only: worker CLIs come from engine `status.workers`, and
// model/provider profiles come from RPP `settings.readback` fields that really
// exist. No React, no Tauri imports. No invented providers or live metrics.

export interface WorkerCli {
  name: string;
  available: boolean;
}

export interface AgentRole {
  id: string;
  name: string;
  description: string;
}

export interface ModelProfile {
  key: string;
  label: string;
  value: string;
}

export interface AgentsModel {
  workers: WorkerCli[];
  roles: AgentRole[];
  profiles: ModelProfile[];
  note: string;
}

// The three roles the orchestrator engine defines internally. Hub shows them
// read-only; it never invents extra roles.
export const AGENT_ROLES: AgentRole[] = [
  { id: 'implementer', name: 'Uygulayıcı', description: 'Şeritteki işi kodlayan ajan.' },
  { id: 'reviewer', name: 'Bağımsız inceleyici', description: 'Uygulayıcıdan bağımsız olarak değişikliği inceleyen ajan.' },
  { id: 'master', name: 'Son onaycı (master)', description: 'Tüm şeritler bittiğinde son incelemeyi yapan ajan.' },
];

// Settings keys the engine may expose for model/provider profiles. Only keys
// that really appear in the readback are shown; nothing is invented and no
// provider list is assumed.
const PROFILE_KEYS: { key: string; label: string }[] = [
  { key: 'default_worker', label: 'Varsayılan worker (CLI)' },
  { key: 'default_worker_profile', label: 'Varsayılan uygulayıcı profili' },
  { key: 'default_reviewer_profile', label: 'Varsayılan inceleyici profili' },
];

export const ROLE_NOTE = 'Kampanya bazında rol → model ataması kampanya spec dosyasında tanımlanır; Hub burada atama düzenlemez.';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function describeValue(value: unknown): string {
  if (typeof value === 'string') return value;
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  return '';
}

/** Maps engine `status.workers` (name -> resolved CLI path or null) to a list. */
export function deriveWorkers(workers: Record<string, unknown> | null | undefined): WorkerCli[] {
  if (!isRecord(workers)) return [];
  return Object.entries(workers).map(([name, value]) => ({ name, available: Boolean(value) }));
}

/** Reads only the profile keys that really exist in the readback settings. */
export function deriveProfiles(settings: Record<string, unknown> | null | undefined): ModelProfile[] {
  if (!isRecord(settings)) return [];
  const profiles: ModelProfile[] = [];
  for (const { key, label } of PROFILE_KEYS) {
    if (!(key in settings)) continue;
    const value = describeValue(settings[key]);
    if (!value) continue;
    profiles.push({ key, label, value });
  }
  return profiles;
}

export function deriveAgentsModel(
  workers: Record<string, unknown> | null | undefined,
  settings: Record<string, unknown> | null | undefined,
): AgentsModel {
  return {
    workers: deriveWorkers(workers),
    roles: AGENT_ROLES,
    profiles: deriveProfiles(settings),
    note: ROLE_NOTE,
  };
}