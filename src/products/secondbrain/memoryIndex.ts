// SecondBrain memory search index: status and user-confirmed refresh. Calls go through trusted Rust commands only.
import { invoke } from '@tauri-apps/api/core';
import { native } from '../../shared/api.ts';
import type { LifecycleError } from '../../shell/lifecycle.ts';

export type IndexState = 'unsupported' | 'no-vault' | 'missing' | 'invalid' | 'fresh' | 'stale' | 'unknown';
interface LastRefresh { at: string; operation_id: string; indexed: number; skipped: number; deleted: number; total: number; duration_seconds: number; force: boolean; }
export interface IndexStatus { supported: boolean; state: IndexState; indexed_files: number; changed_files: number; vault_id: string | null; last_refresh: LastRefresh | null; problems: string[]; }
interface IndexRefreshResult { indexed: number; skipped: number; deleted: number; total: number; duration_seconds: number; state: IndexState; }
type IndexReply<T> = { ok: true; result: T; error: null } | { ok: false; result: null; error: LifecycleError };

const unavailable = (): IndexReply<never> => ({ ok: false, result: null, error: { code: 'UNAVAILABLE', message: 'Bu işlem yalnızca masaüstü uygulamasında kullanılabilir.', hint: 'Tauri masaüstü sürümünü çalıştırın.' } });

async function call<T>(command: string, args?: Record<string, unknown>): Promise<IndexReply<T>> {
  if (!native) return unavailable();
  try { return await invoke<IndexReply<T>>(command, args); }
  catch (reason) { return { ok: false, result: null, error: { code: 'INTERNAL', message: String(reason), hint: 'Uygulamayı yeniden başlatın.' } }; }
}

export const memoryIndexStatus = () => call<IndexStatus>('memory_index_status');
export interface CaptureResult { captured_sessions: number; compile_status: number; }
export const memoryCaptureRun = () => call<CaptureResult>('memory_capture_run');
export const memoryIndexRefresh = (force = false) => call<IndexRefreshResult>('memory_index_refresh', { force });

interface IndexDescription { state: IndexState; title: string; detail: string; }

export function describeIndexStatus(status: IndexStatus): IndexDescription {
  const state = status.supported ? status.state : 'unsupported';
  switch (state) {
    case 'unsupported': return { state, title: 'Desteklenmiyor', detail: 'Kurulu SecondBrain hafıza dizinini (memory_index) desteklemiyor. SecondBrain’i güncelleyin.' };
    case 'no-vault': return { state, title: 'Kasa yok', detail: 'SecondBrain’de ayarlı bir kasa yok. Önce kasayı kurun.' };
    case 'missing': return { state, title: 'Dizin yok', detail: 'Arama dizini henüz oluşturulmadı. Dizini yenileyerek oluşturabilirsiniz.' };
    case 'invalid': return { state, title: 'Dizin geçersiz', detail: 'Arama dizini okunamadı veya bozuk. Dizini yenileyerek yeniden oluşturun.' };
    case 'fresh': return { state, title: 'Güncel', detail: `Dizin kasayla uyumlu (${status.indexed_files} dosya dizinli).` };
    case 'stale': return { state, title: 'Güncel değil', detail: `${status.changed_files} dosya değişmiş; ${status.indexed_files} dosya dizinli. Dizini yenileyin.` };
    default: return { state: 'unknown', title: 'Belirlenemedi', detail: 'Çok büyük bir kasada dizinin güncel olup olmadığı belirlenemedi. Gerekirse dizini yenileyin.' };
  }
}

export function describeRefreshResult(r: IndexRefreshResult): string {
  return `Dizin yenilendi: ${r.total} dosyadan ${r.indexed} dizinlendi, ${r.skipped} atlandı, ${r.deleted} silindi (${r.duration_seconds} sn).`;
}

/** Engine message and hint only; never anything beyond what the backend returned. */
export function describeIndexError(error: LifecycleError): string {
  const head = error.message || 'Dizin işlemi başarısız.';
  return error.hint ? `${head} ${error.hint}` : head;
}
