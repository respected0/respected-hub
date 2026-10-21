// SecondBrain memory access for the Orchestrator. The grant token lives only in the Rust backend;
// nothing here receives, stores or displays it.
import { invoke } from '@tauri-apps/api/core';
import { native } from '../../shared/api.ts';
import type { LifecycleError } from '../../shell/lifecycle.ts';

export interface MemoryStatus { supported: boolean; enabled: boolean; write?: boolean; write_supported?: boolean; grant_id: string | null; token_set: boolean; pending_revoke: boolean; problems: string[]; }
export interface MemoryEnableResult { enabled: boolean; grant_id: string | null; }
export interface MemoryDisableResult { enabled: false; revoked: boolean; pending_revoke: boolean; }
export type MemoryReply<T> = { ok: true; result: T; error: null } | { ok: false; result: null; error: LifecycleError };

const unavailable = (): MemoryReply<never> => ({ ok: false, result: null, error: { code: 'UNAVAILABLE', message: 'Bu işlem yalnızca masaüstü uygulamasında kullanılabilir.', hint: 'Tauri masaüstü sürümünü çalıştırın.' } });

async function call<T>(command: string, args?: Record<string, unknown>): Promise<MemoryReply<T>> {
  if (!native) return unavailable();
  try { return await invoke<MemoryReply<T>>(command, args); }
  catch (reason) { return { ok: false, result: null, error: { code: 'INTERNAL', message: String(reason), hint: 'Uygulamayı yeniden başlatın.' } }; }
}

export const memoryAccessStatus = () => call<MemoryStatus>('memory_access_status');
export const memoryAccessEnable = (write = false) => call<MemoryEnableResult>('memory_access_enable', { write });
export const memoryAccessDisable = () => call<MemoryDisableResult>('memory_access_disable');

export type MemoryState = 'unsupported' | 'off' | 'on' | 'pending-revoke';
export interface MemoryDescription { state: MemoryState; title: string; detail: string; }

export function describeMemoryStatus(status: MemoryStatus): MemoryDescription {
  if (status.pending_revoke) return { state: 'pending-revoke', title: 'Erişim iptali tamamlanamadı', detail: 'Hafıza erişimi kapatıldı ancak SecondBrain tarafındaki izin henüz iptal edilemedi. İptali yeniden deneyin.' };
  if (!status.supported) return { state: 'unsupported', title: 'Desteklenmiyor', detail: 'Kurulu SecondBrain hafıza erişimini (memory_recall) desteklemiyor. SecondBrain’i güncelleyin.' };
  if (status.enabled) return { state: 'on', title: 'Açık', detail: status.write ? 'Orchestrator, hafızayı kullanan görevlerde kısa not özetleri okuyabilir ve istendiğinde yeniden kullanılabilir notları SecondBrain gelen kutusuna yazabilir.' : 'Orchestrator, hafızayı kullanan görevlerde SecondBrain’den salt okunur kısa not özetleri alabilir.' };
  return { state: 'off', title: 'Kapalı', detail: 'Orchestrator SecondBrain hafızasına erişemez.' };
}

/** Engine message and hint only; never anything beyond what the backend returned. */
export function describeMemoryError(error: LifecycleError): string {
  const head = error.message || 'Hafıza erişimi işlemi başarısız.';
  return error.hint ? `${head} ${error.hint}` : head;
}
