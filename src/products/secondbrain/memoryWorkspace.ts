// SecondBrain memory workspace: the user's own note search, note reader, quick capture and briefing. Calls go through trusted Rust commands.
import { invoke } from '@tauri-apps/api/core';
import { native } from '../../shared/api.ts';
import type { LifecycleError } from '../../shell/lifecycle.ts';

interface SearchHit { path: string; title: string; category: string; tags: string; snippet: string; score: number | null; }
interface SearchInfo { mode: 'fts' | 'semantic' | 'hybrid'; semantic: string; reason?: string; }
export interface SearchResult { results: SearchHit[]; count: number; dropped: number; dropped_stale: number; search: SearchInfo; index: { state: string }; }
export interface NoteContent { path: string; title: string; content: string; truncated: boolean; size: number; modified: string; }
export interface CaptureResult { path: string; replayed: boolean; indexed: boolean; }
interface BriefingResult { status: number; }
type WorkspaceReply<T> = { ok: true; result: T; error: null } | { ok: false; result: null; error: LifecycleError };

const unavailable = (): WorkspaceReply<never> => ({ ok: false, result: null, error: { code: 'UNAVAILABLE', message: 'Bu işlem yalnızca masaüstü uygulamasında kullanılabilir.', hint: 'Tauri masaüstü sürümünü çalıştırın.' } });

async function call<T>(command: string, args?: Record<string, unknown>): Promise<WorkspaceReply<T>> {
  if (!native) return unavailable();
  try { return await invoke<WorkspaceReply<T>>(command, args); }
  catch (reason) { return { ok: false, result: null, error: { code: 'INTERNAL', message: String(reason), hint: 'Uygulamayı yeniden başlatın.' } }; }
}

export const searchNotes = (query: string, limit = 15) => call<SearchResult>('memory_search', { query, limit });
export const readNote = (path: string) => call<NoteContent>('memory_note_read', { path });
export const captureNote = (content: string, title?: string) => call<CaptureResult>('memory_capture_note', { title: title?.trim() || null, content });
export const runBriefing = () => call<BriefingResult>('memory_briefing_run');

/** Notes the SecondBrain engine itself maintains; shown as one-click reads next to the search. */
export const KNOWN_NOTES: { label: string; path: (today: string) => string }[] = [
  { label: 'Son oturum', path: () => '🔮 850-Companion/Last-Session.md' },
  { label: 'Açık konular', path: () => '🔮 850-Companion/Threads.md' },
  { label: 'Kurallar', path: () => '🔮 850-Companion/Kurallar.md' },
  { label: 'Bugünün günlüğü', path: today => `daily/${today}.md` },
  { label: 'Bugünün brifingi', path: today => `🎯 100-Command-Center/Briefings/${today}.md` },
];

/** Engine message and hint only; never anything beyond what the backend returned. */
export function describeWorkspaceError(error: LifecycleError): string {
  const head = error.message || 'İşlem başarısız.';
  return error.hint ? `${head} ${error.hint}` : head;
}

export function describeSearchMode(info: SearchInfo): string {
  if (info.mode === 'fts') return info.semantic === 'unavailable' ? 'Tam metin araması (anlamsal arama şu an kullanılamıyor)' : 'Tam metin araması';
  return info.mode === 'semantic' ? 'Anlamsal arama' : 'Tam metin + anlamsal arama';
}
