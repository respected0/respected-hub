// Brain memory access for the Orchestrator: the Orchestrator connects itself (`memory.connect`, the same as
// `respected-orchestrator memory connect` without the Hub). The grant token never leaves the two products.
import { invoke } from '@tauri-apps/api/core';
import { native } from '../../shared/api.ts';
import type { LifecycleError } from '../../shell/lifecycle.ts';

/** The Orchestrator's own answer (`memory.status`). */
interface LinkStatus { connected: boolean; write: boolean; grant_id: string | null; brain: string; }
export interface MemoryStatus { enabled: boolean; write: boolean; grant_id: string | null; pending_revoke: boolean; }
type MemoryReply<T> = { ok: true; result: T; error: null } | { ok: false; result: null; error: LifecycleError };

const unavailable = (): MemoryReply<never> => ({ ok: false, result: null, error: { code: 'UNAVAILABLE', message: 'Bu işlem yalnızca masaüstü uygulamasında kullanılabilir.', hint: 'Tauri masaüstü sürümünü çalıştırın.' } });

async function call<T>(command: string, args?: Record<string, unknown>): Promise<MemoryReply<T>> {
  if (!native) return unavailable();
  try { return await invoke<MemoryReply<T>>(command, args); }
  catch (reason) { return { ok: false, result: null, error: { code: 'INTERNAL', message: String(reason), hint: 'Uygulamayı yeniden başlatın.' } }; }
}

// a disconnect that could not reach Brain keeps the grant id, so the revoke can be retried
const view = (link: LinkStatus): MemoryStatus => ({ enabled: link.connected, write: link.write, grant_id: link.grant_id, pending_revoke: !link.connected && !!link.grant_id });
const mapped = async (reply: Promise<MemoryReply<LinkStatus>>): Promise<MemoryReply<MemoryStatus>> => {
  const answer = await reply;
  return answer.ok ? { ok: true, result: view(answer.result), error: null } : answer;
};

export const memoryAccessStatus = () => mapped(call<LinkStatus>('memory_access_status'));
export const memoryAccessEnable = (write = false) => mapped(call<LinkStatus>('memory_access_enable', { write }));
export const memoryAccessDisable = () => mapped(call<LinkStatus>('memory_access_disable'));

type MemoryState = 'off' | 'on' | 'pending-revoke';
interface MemoryDescription { state: MemoryState; title: string; detail: string; }

export function describeMemoryStatus(status: MemoryStatus): MemoryDescription {
  if (status.pending_revoke) return { state: 'pending-revoke', title: 'Erişim iptali tamamlanamadı', detail: 'Hafıza erişimi kapatıldı ancak Brain tarafındaki izin henüz iptal edilemedi. İptali yeniden deneyin.' };
  if (status.enabled) return { state: 'on', title: 'Açık', detail: status.write ? 'Orchestrator, hafızayı kullanan görevlerde kısa not özetleri okuyabilir ve istendiğinde yeniden kullanılabilir notları Brain gelen kutusuna yazabilir.' : 'Orchestrator, hafızayı kullanan görevlerde Brain’den salt okunur kısa not özetleri alabilir.' };
  return { state: 'off', title: 'Kapalı', detail: 'Orchestrator Brain hafızasına erişemez.' };
}

/** Engine message and hint only; never anything beyond what the backend returned. */
export function describeMemoryError(error: LifecycleError): string {
  const head = error.message || 'Hafıza erişimi işlemi başarısız.';
  return error.hint ? `${head} ${error.hint}` : head;
}
