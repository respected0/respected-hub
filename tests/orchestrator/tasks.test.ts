import test from 'node:test';
import assert from 'node:assert/strict';
import { buildRequest, terminal } from '../../src/products/orchestrator/tasks.ts';

const form = { projectRoot: 'C:/fixture', task: 'Fix task', worker: 'codex', kind: 'write', ownership: 'src/file.py\ntests',
  forbidden: '.env', acceptance: '[["python","-m","unittest"]]', customCommand: '', timeout: 120 };

test('visual submission produces explicit structured engine request', () => {
  const request = buildRequest(form);
  assert.deepEqual(request.ownership, ['src/file.py', 'tests']);
  assert.deepEqual(request.acceptance, [['python', '-m', 'unittest']]);
  assert.equal(request.project_root, 'C:/fixture');
});

test('write cannot silently omit verification or execution scope', () => {
  for (const override of [{ acceptance: '[]' }, { ownership: '' }, { acceptance: 'pytest' }, { timeout: 0 }]) {
    assert.throws(() => buildRequest({ ...form, ...override }));
  }
});

test('read permits zero acceptance commands and custom uses argv arrays', () => {
  assert.deepEqual(buildRequest({ ...form, kind: 'read', acceptance: '[]' }).acceptance, []);
  assert.deepEqual(buildRequest({ ...form, worker: 'custom', customCommand: '["tool","{prompt}"]' }).command, ['tool', '{prompt}']);
  assert.throws(() => buildRequest({ ...form, worker: 'custom', customCommand: 'tool --run' }));
});

test('queued, cancellation requested and running remain pending', () => {
  for (const status of ['queued', 'running', 'cancellation_requested']) assert.equal(terminal(status), false);
  for (const status of ['completed', 'failed', 'timed_out', 'cancelled', 'interrupted', 'invalid']) assert.equal(terminal(status), true);
});
