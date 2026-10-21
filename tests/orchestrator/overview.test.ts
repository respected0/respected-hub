import { test } from 'node:test';
import assert from 'node:assert/strict';
import { classifyRun, deriveOverview, latestRuns, DROPPED_WARNING, RUNS_WARNING, WORKERS_WARNING } from '../../src/products/orchestrator/views/overviewModel.ts';
import type { Run } from '../../src/shared/api.ts';

function run(id: string, status: string): Run {
  return { run_id: id, status };
}

test('classifyRun: groups by status group, tolerating missing fields', () => {
  assert.equal(classifyRun('running'), 'active');
  assert.equal(classifyRun('queued'), 'active');
  assert.equal(classifyRun('completed'), 'succeeded');
  assert.equal(classifyRun('failed'), 'failed');
  assert.equal(classifyRun('timed_out'), 'failed');
  assert.equal(classifyRun('cancelled'), 'other');
  assert.equal(classifyRun(''), 'other');
  assert.equal(classifyRun(undefined), 'other');
  assert.equal(classifyRun(42), 'other');
});

test('latestRuns: newest five by run_id, regardless of input order', () => {
  const runs = [
    run('run-20260101000000-a', 'completed'),
    run('run-20260301000000-c', 'completed'),
    run('run-20260201000000-b', 'completed'),
    run('run-20260601000000-f', 'completed'),
    run('run-20260501000000-e', 'completed'),
    run('run-20260401000000-d', 'completed'),
  ];
  assert.deepEqual(latestRuns(runs).map((r) => r.run_id), [
    'run-20260601000000-f',
    'run-20260501000000-e',
    'run-20260401000000-d',
    'run-20260301000000-c',
    'run-20260201000000-b',
  ]);
});

test('deriveOverview: counts runs, splits workers, no warnings with real data', () => {
  const status = { workers: { codex: 'C:/bin/codex.exe', claude: null, antigravity: '' } };
  const model = deriveOverview(status, [
    run('run-20260101000000-a', 'running'),
    run('run-20260102000000-b', 'completed'),
    run('run-20260103000000-c', 'failed'),
    run('run-20260104000000-d', 'cancelled'),
  ]);
  assert.deepEqual(model.runCounts, { active: 1, succeeded: 1, failed: 1, other: 1 });
  assert.deepEqual(model.workersAvailable, ['codex']);
  assert.deepEqual(model.workersMissing, ['claude', 'antigravity']);
  assert.deepEqual(model.warnings, []);
  assert.equal(model.latestRuns[0].run_id, 'run-20260104000000-d');
});

test('deriveOverview: accepts a raw workers record as well as a status object', () => {
  const model = deriveOverview({ codex: ['C:/bin/codex.exe'] }, []);
  assert.deepEqual(model.workersAvailable, ['codex']);
  assert.deepEqual(model.workersMissing, []);
});

test('deriveOverview: missing workers and runs yield honest warnings, never data', () => {
  const model = deriveOverview(null, null);
  assert.deepEqual(model.runCounts, { active: 0, succeeded: 0, failed: 0, other: 0 });
  assert.deepEqual(model.latestRuns, []);
  assert.deepEqual(model.workersAvailable, []);
  assert.deepEqual(model.workersMissing, []);
  assert.ok(model.warnings.includes(WORKERS_WARNING));
  assert.ok(model.warnings.includes(RUNS_WARNING));
});

test('deriveOverview: malformed run records are dropped with a warning', () => {
  const model = deriveOverview({ workers: {} }, [null, { status: 'running' }, run('run-20260101000000-a', 'completed')]);
  assert.deepEqual(model.runCounts, { active: 0, succeeded: 1, failed: 0, other: 0 });
  assert.deepEqual(model.latestRuns.map((r) => r.run_id), ['run-20260101000000-a']);
  assert.ok(model.warnings.includes(DROPPED_WARNING));
});