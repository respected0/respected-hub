// Orchestrator MCP access in AI clients. The Orchestrator writes the client configs; the Hub plans, asks approval and journals.
import { invoke } from '@tauri-apps/api/core';
import { native } from '../../shared/api.ts';
import type { JournalEntry, LifecycleError, LifecycleReply } from '../../shell/lifecycle.ts';

interface McpClientRow { client: string; config: string; detected: boolean; state: 'absent' | 'owned' | 'foreign' | 'modified'; action: 'unchanged' | 'create' | 'update' | 'remove' | 'refuse'; reason: string; }
export interface McpPlan { clients: McpClientRow[]; plan_digest: string; writes_nothing: boolean; }

const CLIENT_LABEL: Record<string, string> = { 'claude-code': 'Claude Code', codex: 'Codex', cursor: 'Cursor', gemini: 'Gemini CLI', windsurf: 'Windsurf', 'claude-desktop': 'Claude Desktop' };
export const clientLabel = (id: string) => CLIENT_LABEL[id] ?? id;

async function call<T>(command: string, args: Record<string, unknown>): Promise<LifecycleReply<T>> {
  if (!native) return { ok: false, result: null, error: { code: 'UNAVAILABLE', message: 'Bu işlem yalnızca masaüstü uygulamasında kullanılabilir.', hint: 'Tauri masaüstü sürümünü çalıştırın.' } };
  try { return await invoke<LifecycleReply<T>>(command, args); }
  catch (reason) { return { ok: false, result: null, error: { code: 'INTERNAL', message: String(reason), hint: 'Uygulamayı yeniden başlatın.' } }; }
}

export const mcpPlan = (desired: Record<string, boolean>) => call<McpPlan>('orchestrator_mcp_plan', { desired });
export const mcpApply = (desired: Record<string, boolean>, planDigest: string) =>
  call<JournalEntry>('orchestrator_mcp_apply', { desired, planDigest, operationId: `mcp-${crypto.randomUUID()}`.slice(0, 64) });

/** The switches start from what Respected already owns; only clients that differ from that are sent. */
export function initialDesired(plan: McpPlan): Record<string, boolean> {
  return Object.fromEntries(plan.clients.map(row => [row.client, row.state === 'owned']));
}
export function changedOnly(desired: Record<string, boolean>, initial: Record<string, boolean>): Record<string, boolean> {
  return Object.fromEntries(Object.entries(desired).filter(([client, on]) => on !== initial[client]));
}
export const describeMcpError = (error: LifecycleError) => `${error.message}${error.hint ? ` ${error.hint}` : ''}`;
