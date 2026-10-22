// Product lifecycle client: the Hub runs each product's own installer (install / update / repair) or uninstaller.
// The native backend answers { ok, result, error:{code,message,hint} }; nothing here is simulated.
import { invoke } from '@tauri-apps/api/core';
import { native } from '../shared/api.ts';

export type LifecycleAction = 'install' | 'update' | 'repair' | 'uninstall';
type InstallState = 'absent' | 'installed' | 'update-available' | 'repair-needed';

export interface LifecycleError { code: string; message: string; hint: string; }
export type LifecycleReply<T> = { ok: true; result: T; error: null } | { ok: false; result: null; error: LifecycleError };

export interface ProductLifecycle {
  product: string;
  installed: boolean;
  install_state: InstallState;
  version: string | null;
  /** When the product last wrote its install record (epoch ms): by the Hub, its installer or its own command line. */
  recorded_at?: string | null;
  /** `remote`: listed in the downloaded catalog index, fetched from GitHub when the person installs or updates. */
  available: { version: string; signed: boolean; trust?: 'production' | 'development' | 'unsigned'; remote?: boolean; size?: number } | null;
  catalog_skipped: string[];
  root: string | null;
}
export interface LifecycleOverview { products: (ProductLifecycle | null)[]; catalog_root: string; }
interface PlanStep { id: string; summary: string; reversible: boolean; irreversible_reason: string | null; }
interface IntegrationChange { kind: string; target: string; action: 'create' | 'modify' | 'restore' | 'unchanged'; outcome?: 'delete' | 'restore' | 'remove-entry'; summary: string; size_after: number; }
export interface IntegrationPlan { desired: Record<string, unknown>; current?: Record<string, unknown>; clients?: { id: string; detected: boolean }[]; launcher: string[]; discovered: { client_config: string; present: boolean }[]; changes: IntegrationChange[]; writes_nothing: boolean; external_effects: string[]; warnings?: string[]; }
export interface LifecyclePlan { product: string; action: LifecycleAction; from_version: string | null; to_version: string | null; plan: { steps: PlanStep[] }; }
type LifecycleParams = Record<string, unknown>;
export interface JournalEntry {
  operation_id: string; product: string; action: LifecycleAction;
  status: 'staging' | 'verifying' | 'switching' | 'applied' | 'failed';
  from_version: string | null; to_version: string | null; started_at: string; finished_at: string | null;
  error: LifecycleError | null;
}

const unavailable = (): LifecycleReply<never> => ({ ok: false, result: null, error: { code: 'UNAVAILABLE', message: 'Bu işlem yalnızca masaüstü uygulamasında kullanılabilir.', hint: 'Tauri masaüstü sürümünü çalıştırın.' } });

async function call<T>(command: string, args?: Record<string, unknown>): Promise<LifecycleReply<T>> {
  if (!native) return unavailable();
  try { return await invoke<LifecycleReply<T>>(command, args); }
  catch (reason) { return { ok: false, result: null, error: { code: 'INTERNAL', message: String(reason), hint: 'Uygulamayı yeniden başlatın.' } }; }
}

export const lifecycleStatus = () => call<LifecycleOverview>('lifecycle_status');
export const lifecyclePlan = (product: string, action: LifecycleAction, params?: LifecycleParams) => call<LifecyclePlan>('lifecycle_plan', { product, action, params: params ?? null });
export const lifecycleApply = (product: string, action: LifecycleAction, params?: LifecycleParams, operationId = newOperationId(action)) =>
  call<JournalEntry>('lifecycle_apply', { product, action, operationId, params: params ?? null });
export const activityList = (limit = 50) => call<JournalEntry[]>('activity_list', { limit });

export type Channel = 'beta' | 'stable';
export interface HubUpdate { current: string; channel: Channel; available: { version: string; size: number; min_hub: string | null } | null; checked: boolean; }
export const hubUpdateStatus = () => call<HubUpdate>('hub_update_status');
/** Downloads and verifies the signed catalog index again (GitHub), then answers like hubUpdateStatus. */
export const catalogRefresh = () => call<HubUpdate>('catalog_refresh');
export const catalogSetChannel = (channel: Channel) => call<HubUpdate>('catalog_set_channel', { channel });
/** Starts the verified Hub installer; the Hub closes itself and the installer reopens it. */
export const hubUpdateInstall = () => call<{ started: boolean }>('hub_update_install');

export function newOperationId(action: LifecycleAction): string {
  return `${action}-${crypto.randomUUID()}`.slice(0, 64);
}

export const ACTION_LABEL: Record<LifecycleAction, string> = { install: 'Kur', update: 'Güncelle', repair: 'Onar', uninstall: 'Kaldır' };
export const ACTION_DONE: Record<LifecycleAction, string> = { install: 'kuruldu', update: 'güncellendi', repair: 'onarıldı', uninstall: 'kaldırıldı' };

export const STATE_LABEL: Record<InstallState, string> = {
  absent: 'Kurulu değil', installed: 'Kurulu ve doğrulandı', 'update-available': 'Güncelleme var', 'repair-needed': 'Onarım gerekli',
};

const ERROR_TR: Record<string, string> = {
  LIFECYCLE_CONFLICT: 'Bu işlem şu an yapılamaz.',
  VERIFY_FAILED: 'Doğrulama başarısız oldu; hiçbir değişiklik uygulanmadı.',
  LIFECYCLE_PARTIAL: 'İşlem tamamlanamadı.',
  BAD_REQUEST: 'İstek geçersiz.',
  INTERNAL: 'Beklenmeyen bir hata oluştu.',
  CATALOG_UNAVAILABLE: 'Ürün kataloğuna ulaşılamadı.',
  HUB_TOO_OLD: 'Bu sürüm daha yeni bir Hub gerektiriyor.',
};

/** One readable sentence for a failed lifecycle reply: the engine message first, then the recovery hint. */
export function describeLifecycleError(error: LifecycleError): string {
  const head = error.message || ERROR_TR[error.code] || 'İşlem başarısız.';
  return error.hint ? `${head} ${error.hint}` : head;
}

/** Actions the UI may offer for a product in this state (matches what the backend plan accepts). */
export function availableActions(state: InstallState, hasPackage: boolean): LifecycleAction[] {
  switch (state) {
    case 'absent': return hasPackage ? ['install'] : [];
    case 'update-available': return hasPackage ? ['update', 'uninstall'] : ['uninstall'];
    case 'repair-needed': return hasPackage ? ['repair', 'uninstall'] : ['uninstall'];
    default: return hasPackage ? ['repair', 'uninstall'] : ['uninstall'];
  }
}

export function formatWhen(epochMs: string | null): string {
  const value = Number(epochMs);
  return Number.isFinite(value) && value > 0 ? new Date(value).toLocaleString('tr-TR') : '';
}

export function journalSentence(entry: JournalEntry, productName: string): string {
  const what = `${productName} ${ACTION_DONE[entry.action] ?? entry.action}`;
  switch (entry.status) {
    case 'applied': return `${what}${entry.to_version ? ` (v${entry.to_version})` : ''}.`;
    case 'failed': return `${productName}: ${ACTION_LABEL[entry.action]?.toLowerCase()} başarısız. ${entry.error ? describeLifecycleError(entry.error) : ''}`.trim();
    default: return `${productName}: ${ACTION_LABEL[entry.action]?.toLowerCase()} sürüyor…`;
  }
}
