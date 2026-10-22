import { test } from 'node:test';
import assert from 'node:assert/strict';
import { fromCurrent, toDesired, toggle, summarize } from '../../src/products/brain/brainClients.ts';

test('toDesired: flags follow the per-client lists', () => {
  const desired = toDesired({ sources: ['codex', 'claude'], mcp: [], schedule: true, shortcut: false });
  assert.deepEqual(desired, { global: true, mcp: false, schedule: true, shortcut: false, clients: ['claude', 'codex'], mcp_clients: [] });
});

test('fromCurrent: stored lists, legacy "all clients" and switched-off flags', () => {
  assert.deepEqual(fromCurrent({ global: true, mcp: true, clients: ['claude'], mcp_clients: ['cursor'] }), { sources: ['claude'], mcp: ['cursor'], schedule: false, shortcut: false });
  const legacy = fromCurrent({ global: true, mcp: true });
  assert.ok(legacy.sources.includes('gemini') && !legacy.sources.includes('windsurf') && legacy.mcp.includes('windsurf') && !legacy.mcp.includes('codex'));
  assert.deepEqual(fromCurrent({ global: false, mcp: false, clients: ['claude'] }).sources, []);
  assert.deepEqual(fromCurrent({ mcp: true, clients: ['claude'] }).mcp, ['claude']);  // mcp_clients falls back to clients
  assert.deepEqual(fromCurrent({ global: true, clients: ['nope', 'codex'] }).sources, ['codex']);
});

test('toggle and summarize', () => {
  assert.deepEqual(toggle(['a'], 'b', true), ['a', 'b']);
  assert.deepEqual(toggle(['a', 'b'], 'a', false), ['b']);
  assert.equal(summarize([]), 'Hiçbiri');
  assert.equal(summarize(['claude', 'x']), 'Claude Code, x');
});
