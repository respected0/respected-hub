import test from 'node:test';
import assert from 'node:assert/strict';
import { buildRequest, taskTitle, terminal, usageText } from '../../src/products/orchestrator/tasks.ts';
import type { Run } from '../../src/shared/api.ts';

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

test('task title is the first task line, capped, with a run-id fallback', () => {
  const run = (task: string) => ({ run_id: 'run-1', status: 'completed', request: { task, worker: 'codex', project_root: 'C:/p' } } as Run);
  assert.equal(taskTitle(run('Fix the parser\nsecond line')), 'Fix the parser');
  assert.equal(taskTitle(run('x'.repeat(150))).length, 100);
  assert.equal(taskTitle({ run_id: 'run-2', status: 'queued' }), 'run-2');
});

test('usage text reports weighted, input, cache, output and reasoning tokens', () => {
  const text = usageText({ estimated_weighted_tokens: 1234, raw_input_tokens: 1000, cached_input_tokens: 200,
    output_tokens: 100, reasoning_tokens: 50 });
  for (const value of ['1.234', '1.000', '200', '100', '50']) assert.ok(text.includes(value), `${value} missing from ${text}`);
  assert.equal(usageText(null), 'Token kullanımı bildirilmedi');
});
