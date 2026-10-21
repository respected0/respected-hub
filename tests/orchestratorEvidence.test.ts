import { test } from 'node:test';
import assert from 'node:assert/strict';
import { evidenceFor } from '../src/products/orchestrator/views/evidenceModel.ts';

test('approved lane with a patch gives the exact manual apply command', () => {
  const campaign = { status: 'LANES_APPROVED', result: { lanes: [
    { name: 'api', status: 'MASTER_APPROVED', run_id: 'run-42', changed_files: ['a.ts', 'b.ts'], token_usage: { master: 10, worker: 5 } },
  ] } };
  const evidence = evidenceFor(null, campaign);
  assert.equal(evidence.verdict, 'approved');
  assert.equal(evidence.hasPatch, true);
  assert.ok(evidence.nextStepText.includes('apply run-42'));
  assert.ok(evidence.nextStepText.includes('çalıştırmaz'));
  assert.ok(evidence.evidence.some(item => item.label === 'Toplam token' && item.text === '15'));
  assert.ok(evidence.evidence.some(item => item.label === 'Şerit: api'));
});

test('human approval points at the interactive approve command', () => {
  const campaign = { status: 'LANES_NEED_REVIEW', result: { lanes: [
    { name: 'ui', status: 'HUMAN_APPROVAL_REQUIRED', run_id: 'run-7' },
  ] } };
  const evidence = evidenceFor(null, campaign);
  assert.equal(evidence.verdict, 'human_approval');
  assert.ok(evidence.nextStepText.includes('maf-approve run-7'));
  assert.ok(evidence.nextStepText.includes('interaktif'));
});

test('failed run never suggests applying a patch', () => {
  const evidence = evidenceFor({ run_id: 'run-9', status: 'failed', error: 'worker crash' }, null);
  assert.equal(evidence.verdict, 'failed');
  assert.ok(evidence.evidence.some(item => item.label === 'Hata' && item.text === 'worker crash'));
  assert.ok(!evidence.nextStepText.includes('apply'));
});

test('a review requesting changes yields needs_changes and is listed', () => {
  const campaign = { result: { lanes: [
    { name: 'x', status: 'LANES_APPROVED', reviews: [{ stage: 'post', profile: 'strict', decision: 'request_changes', reason: 'test yok' }] },
  ] } };
  const evidence = evidenceFor(null, campaign);
  assert.equal(evidence.verdict, 'needs_changes');
  assert.equal(evidence.reviews.length, 1);
  assert.deepEqual(evidence.reviews[0], { stage: 'post', profile: 'strict', decision: 'request_changes', reason: 'test yok' });
});

test('inspect payload exposes task, worker, logs and the patch preview', () => {
  const inspect = { run_id: 'run-1', status: 'completed', phase: 'review', patch_preview: 'diff --git a/x',
    request: { task: 'görev', worker: 'codex', project_root: '/p' }, logs: { engine: 'satır' } };
  const evidence = evidenceFor(inspect, null);
  assert.equal(evidence.verdict, 'approved');
  assert.equal(evidence.hasPatch, true);
  assert.ok(evidence.evidence.some(item => item.label === 'Görev' && item.text === 'görev'));
  assert.ok(evidence.evidence.some(item => item.label === 'Log: engine' && item.text === 'satır'));
  assert.ok(evidence.nextStepText.includes('apply run-1'));
});

test('the fallback run id is used when the payload has none', () => {
  const evidence = evidenceFor(null, { result: { lanes: [{ status: 'MASTER_APPROVED' }] } }, 'run-fallback');
  assert.ok(evidence.nextStepText.includes('apply run-fallback'));
});

test('malformed and empty payloads never throw and stay unknown', () => {
  const empty = evidenceFor(undefined, null);
  assert.equal(empty.verdict, 'unknown');
  assert.equal(empty.evidence.length, 0);
  assert.equal(empty.reviews.length, 0);
  assert.equal(empty.hasPatch, false);
  assert.ok(empty.nextStepText.length > 0);

  assert.equal(evidenceFor('nope', 5).verdict, 'unknown');
  assert.equal(evidenceFor(null, { result: { lanes: 'x' } }).verdict, 'unknown');
  assert.equal(evidenceFor({ request: 'bad', logs: 'bad' }, null).verdict, 'unknown');
});