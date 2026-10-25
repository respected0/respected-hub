// What the Orchestrator's Settings page reads, kept warm: the Hub asks once after it opens, and the page shows the last answer at
// once (peek) while it refreshes (every product call starts a program, see rppCache.ts).
import { rppCall } from './productClient.ts';
import type { RppProduct } from './productClient.ts';
import type { RppResponse } from './rpp.ts';
import { cachedReply, forget, peek } from './rppCache.ts';
import { mcpPlan } from '../products/orchestrator/mcpAccess.ts';
import type { McpPlan } from '../products/orchestrator/mcpAccess.ts';
import { memoryAccessStatus } from '../products/brain/memoryAccess.ts';

const SCHEMA_MAX_AGE = 10 * 60_000; // the schema only changes with the installed version
const key = (product: RppProduct, op: string) => `${product}:${op}`;

const ask = (product: RppProduct, op: string, maxAge: number) => cachedReply(key(product, op), () => rppCall(product, op), maxAge).catch(() => null);

export const settingsSchema = (product: RppProduct) => ask(product, 'settings.schema', SCHEMA_MAX_AGE);
export const settingsReadback = (product: RppProduct) => ask(product, 'settings.readback', 0);
export const peekSettings = (product: RppProduct) => ({ schema: peek<RppResponse>(key(product, 'settings.schema')), readback: peek<RppResponse>(key(product, 'settings.readback')) });
export const forgetReadback = (product: RppProduct) => forget(key(product, 'settings.readback'));

export const orchestratorMcpStatus = () => cachedReply(key('orchestrator', 'mcp.status'), () => mcpPlan({}));
export const peekMcpStatus = () => peek<{ ok: true; result: McpPlan }>(key('orchestrator', 'mcp.status'));
export const forgetMcpStatus = () => forget(key('orchestrator', 'mcp.status'));

export const brainAccessStatus = () => cachedReply('brain:memory.access', () => memoryAccessStatus());
export const peekBrainAccess = () => peek<Awaited<ReturnType<typeof memoryAccessStatus>>>('brain:memory.access');
export const forgetBrainAccess = () => forget('brain:memory.access');

let warmed = false;
/** Starts all the reads of the Orchestrator's Settings page in the background, once; the page refreshes them itself when it opens. */
export function warmOrchestratorSettings(): void {
  if (warmed) return;
  warmed = true;
  void settingsSchema('orchestrator'); void settingsReadback('orchestrator'); void orchestratorMcpStatus(); void brainAccessStatus();
}
