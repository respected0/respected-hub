import assert from 'node:assert/strict';
import { test } from 'node:test';
import { changedOnly, clientLabel, initialDesired } from '../src/mcpAccess.ts';
import type { McpPlan } from '../src/mcpAccess.ts';

const row = (client: string, state: McpPlan['clients'][number]['state']) => ({ client, config: `C:/${client}`, detected: true, state, action: 'unchanged' as const, reason: '' });

test('switches start from owned registrations and only differences are sent', () => {
  const plan: McpPlan = { clients: [row('claude-code', 'owned'), row('codex', 'absent'), row('cursor', 'foreign')], plan_digest: 'd', writes_nothing: true };
  const initial = initialDesired(plan);
  assert.deepEqual(initial, { 'claude-code': true, codex: false, cursor: false });
  assert.deepEqual(changedOnly({ ...initial, codex: true, 'claude-code': false }, initial), { 'claude-code': false, codex: true });
  assert.deepEqual(changedOnly(initial, initial), {});
  assert.equal(clientLabel('claude-code'), 'Claude Code');
  assert.equal(clientLabel('x'), 'x');
});
