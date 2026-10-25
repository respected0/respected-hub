// What the Orchestrator's Settings page reads, kept warm: the Hub asks once after it opens, and the page shows the last answer at
// once (peek) while it refreshes (every product call starts a program, see rppCache.ts).
import { rppCall } from './productClient.ts';
import type { RppProduct } from './productClient.ts';
import type { RppResponse } from './rpp.ts';
import { cached, cachedReply, forget, peek } from './rppCache.ts';
import { mcpPlan } from '../products/orchestrator/mcpAccess.ts';
import type { McpPlan } from '../products/orchestrator/mcpAccess.ts';
import { memoryAccessStatus } from '../products/brain/memoryAccess.ts';
import { campaignNames } from '../products/orchestrator/campaigns.ts';
import { engine } from './api.ts';
import { fromSettings, toDesired } from '../products/brain/brainClients.ts';
import type { IntegrationPlan } from '../shell/lifecycle.ts';

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

export const brainManifest = () => ask('brain', 'manifest', 5 * 60_000);
export const peekBrainManifest = () => peek<RppResponse>(key('brain', 'manifest'));

const planKey = (desired: Record<string, unknown>) => `brain:integrations.plan:${JSON.stringify(desired)}`;
/** What the Brain connections would change for `desired` (read-only); asked again each time, the last answer is shown meanwhile. */
export const integrationsPlan = (desired: Record<string, unknown>) => cachedReply(planKey(desired), () => rppCall('brain', 'integrations.plan', { target: { desired } })).catch(() => null);
export const peekIntegrationsPlan = (desired: Record<string, unknown>) => peek<{ ok: true; result: IntegrationPlan }>(planKey(desired))?.result ?? null;

type Workers = Record<string, string[] | null> | null;
/** The engine's worker status (which models are installed): one question shared by the Overview and the Agents page. */
export const orchestratorWorkers = () => cached('orchestrator:workers', () => engine<{ workers?: Record<string, string[] | null> }>('status').then(status => status.workers ?? null));
export const peekWorkers = () => peek<Workers>('orchestrator:workers');

let brainWarmed = false;
export const forgetBrainManifest = () => forget(key('brain', 'manifest'));
/** The same for the Brain's Settings page: schema, settings, manifest and the connections plan for the stored choice. */
export function warmBrainSettings(): void {
  if (brainWarmed) return;
  brainWarmed = true;
  void settingsSchema('brain'); void brainManifest();
  void settingsReadback('brain').then(reply => {
    const stored = reply?.ok && typeof reply.result === 'object' && reply.result ? (reply.result as { settings?: Record<string, unknown> }).settings : undefined;
    if (stored) void integrationsPlan(toDesired(fromSettings(stored)));
  });
}

let warmed = false;
/** A product changed under us (an update, a repair, a removal by an installer): its kept answers are the old program's. They are dropped and warmed again by the next refresh. */
export function coolSettings(product: RppProduct): void {
  forget(`${product}:`);
  if (product === 'brain') brainWarmed = false; else warmed = false;
}
/** Starts all the reads of the Orchestrator's Settings page in the background, once; the page refreshes them itself when it opens. */
export function warmOrchestratorSettings(): void {
  if (warmed) return;
  warmed = true;
  void settingsSchema('orchestrator'); void settingsReadback('orchestrator'); void orchestratorMcpStatus(); void brainAccessStatus();
  void orchestratorWorkers().catch(() => undefined); void campaignNames().catch(() => undefined);
}
