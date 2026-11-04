import test from 'node:test';
import assert from 'node:assert/strict';
import type { ResolvedEntry } from '../../src/shared/integrationsTypes.ts';
import {
  filterIntegrations, groupIntegrations, INTEGRATION_STATE_LABEL,
} from '../../src/shell/integrationMatrix.ts';

const entry = (over: Partial<ResolvedEntry> & Pick<ResolvedEntry, 'asset_key' | 'name' | 'kind' | 'client' | 'scope'>): ResolvedEntry => ({
  project: null, source_path: 'C:/Users/test/.config.json', configured: true, effective: true,
  enabled: true, managed: false, owner: { kind: 'user' }, state: 'effective', ...over,
});

const postgresClaude = entry({ asset_key: 'mcp:postgres', name: 'PostgreSQL', kind: 'mcp', client: 'claude', scope: 'global' });
const postgresCodex = entry({
  asset_key: 'mcp:postgres', name: 'PostgreSQL', kind: 'mcp', client: 'codex', scope: 'project',
  project: { id: 'uretim', root: 'C:/Projeler/Üretim Projesi' }, source_path: 'C:/Projeler/Üretim Projesi/.codex/config.toml',
});
const securitySkill = entry({
  asset_key: 'skill:guvenlik', name: 'Güvenlik Taraması', kind: 'skill', client: 'claude', scope: 'global',
  source_path: 'C:/Users/test/.claude/skills/güvenlik/SKILL.md',
});
const githubPlugin = entry({ asset_key: 'plugin:github', name: 'GitHub', kind: 'plugin', client: 'cursor', scope: 'global', state: 'disabled', enabled: false });

const mixed = [postgresClaude, postgresCodex, securitySkill, githubPlugin];

test('same MCP in two clients is one asset with one row per client and scope', () => {
  const groups = groupIntegrations(filterIntegrations(mixed, { kind: 'mcp' }));
  assert.equal(groups.length, 1);
  assert.equal(groups[0].key, 'mcp:postgres');
  assert.deepEqual(groups[0].entries.map(item => [item.client, item.scope]), [['claude', 'global'], ['codex', 'project']]);
});

test('global and project scopes are filtered independently without hiding the project root', () => {
  assert.deepEqual(filterIntegrations(mixed, { scope: 'project' }).map(item => item.project?.id), ['uretim']);
  assert.deepEqual(filterIntegrations(mixed, { scope: 'global' }).map(item => item.asset_key), ['mcp:postgres', 'skill:guvenlik', 'plugin:github']);
  assert.equal(postgresCodex.project?.root, 'C:/Projeler/Üretim Projesi');
});

test('duplicate bindings stay visible instead of being deduplicated', () => {
  const duplicate = entry({ asset_key: 'mcp:postgres', name: 'PostgreSQL', kind: 'mcp', client: 'claude', scope: 'global', source_path: 'D:/other/config.json' });
  const groups = groupIntegrations([postgresClaude, duplicate]);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].entries.length, 2);
  assert.equal(new Set(groups[0].entries.map(item => item.source_path)).size, 2);
});

test('search is Turkish- and Unicode-aware across names, projects and state labels', () => {
  assert.deepEqual(filterIntegrations(mixed, { query: '  GÜVENLİK ' }).map(item => item.asset_key), ['skill:guvenlik']);
  assert.deepEqual(filterIntegrations(mixed, { query: 'üretim projesi' }).map(item => item.asset_key), ['mcp:postgres']);
  assert.deepEqual(filterIntegrations([githubPlugin], { query: 'devre dışı' }).map(item => item.asset_key), ['plugin:github']);
});

test('kind, client and scope filters combine exactly', () => {
  assert.deepEqual(filterIntegrations(mixed, { kind: 'skill' }).map(item => item.asset_key), ['skill:guvenlik']);
  assert.deepEqual(filterIntegrations(mixed, { client: 'codex' }).map(item => item.asset_key), ['mcp:postgres']);
  assert.deepEqual(filterIntegrations(mixed, { kind: 'mcp', client: 'codex', scope: 'project' }).map(item => item.asset_key), ['mcp:postgres']);
  assert.deepEqual(filterIntegrations(mixed, { kind: 'plugin', client: 'claude' }), []);
});

test('effective is the only label that claims the client would use the entry', () => {
  assert.equal(INTEGRATION_STATE_LABEL.effective, 'Etkin');
  assert.equal(INTEGRATION_STATE_LABEL.overridden, 'Gölgelendi');
  assert.equal(INTEGRATION_STATE_LABEL.disabled, 'Devre dışı');
  assert.equal(INTEGRATION_STATE_LABEL.unknown, 'Kaynağı okunamadı');
  assert.equal(new Set(Object.values(INTEGRATION_STATE_LABEL)).size, 5);
});

test('search stays exact on a large list', () => {
  const many = Array.from({ length: 1000 }, (_, index) => entry({
    asset_key: `mcp:server-${index}`,
    name: index === 731 ? 'Özel PostgreSQL' : `Server ${index}`,
    kind: 'mcp', client: index % 2 ? 'codex' : 'claude', scope: index % 3 ? 'global' : 'project',
    project: index % 3 ? null : { id: `proje-${index}`, root: `C:/Projeler/Proje ${index}` },
  }));
  assert.deepEqual(filterIntegrations(many, { query: 'özel postgresql' }).map(item => item.asset_key), ['mcp:server-731']);
  assert.equal(filterIntegrations(many, { client: 'codex' }).length, 500);
});
