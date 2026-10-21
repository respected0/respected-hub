import { test } from 'node:test';
import assert from 'node:assert/strict';
import { AGENT_ROLES, ROLE_NOTE, deriveAgentsModel, deriveProfiles, deriveWorkers } from '../../src/products/orchestrator/views/agentsModel.ts';

test('deriveWorkers: marks present CLIs available and null as missing', () => {
  const workers = deriveWorkers({ codex: 'C:/bin/codex.exe', claude: null, antigravity: '' });
  assert.deepEqual(workers, [
    { name: 'codex', available: true },
    { name: 'claude', available: false },
    { name: 'antigravity', available: false },
  ]);
});

test('deriveWorkers: missing or malformed input yields an empty list', () => {
  assert.deepEqual(deriveWorkers(null), []);
  assert.deepEqual(deriveWorkers(undefined), []);
  assert.deepEqual(deriveWorkers({}), []);
});

test('deriveProfiles: only real readback fields are shown', () => {
  const settings = { default_worker: 'codex', default_reviewer_profile: 'claude-opus', unrelated: 'secret' };
  assert.deepEqual(deriveProfiles(settings), [
    { key: 'default_worker', label: 'Varsayılan worker (CLI)', value: 'codex' },
    { key: 'default_reviewer_profile', label: 'Varsayılan inceleyici profili', value: 'claude-opus' },
  ]);
});

test('deriveProfiles: absent, empty or malformed settings yield nothing', () => {
  assert.deepEqual(deriveProfiles(null), []);
  assert.deepEqual(deriveProfiles({}), []);
  assert.deepEqual(deriveProfiles({ default_worker: '' }), []);
  assert.deepEqual(deriveProfiles({ default_worker_profile: { nested: true } }), []);
});

test('deriveAgentsModel: roles are the fixed engine roles and never invented', () => {
  const model = deriveAgentsModel({ codex: 'C:/bin/codex.exe' }, { default_worker: 'codex' });
  assert.equal(model.roles, AGENT_ROLES);
  assert.deepEqual(model.roles.map((role) => role.id), ['implementer', 'reviewer', 'master']);
  assert.equal(model.workers.length, 1);
  assert.equal(model.profiles.length, 1);
  assert.equal(model.note, ROLE_NOTE);
});

test('deriveAgentsModel: no real data means empty lists with an honest note', () => {
  const model = deriveAgentsModel(null, null);
  assert.deepEqual(model.workers, []);
  assert.deepEqual(model.profiles, []);
  assert.equal(model.roles.length, 3);
  assert.ok(model.note.includes('kampanya spec'));
});