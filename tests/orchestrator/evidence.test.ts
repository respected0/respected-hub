import { test } from 'node:test';
import assert from 'node:assert/strict';
import { evidenceFor } from '../../src/products/orchestrator/evidenceModel.ts';

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
test('manual commands use the installed program and data folder, never Python', () => {
  const cli = '& "C:\Respected\respected-orchestrator.exe" --data-root "D:\data"';
  const campaign = { status: 'LANES_NEED_REVIEW', result: { lanes: [{ status: 'HUMAN_APPROVAL_REQUIRED', run_id: 'run-7' }] } };
  const text = evidenceFor(null, campaign, '', cli).nextStepText;
  assert.ok(text.includes(`${cli} maf-approve run-7`));
  assert.ok(!text.includes('python'));
});

// ---- evidence.view (FIX-03C) ----
import { evidenceFromView } from '../../src/products/orchestrator/evidenceModel.ts';

const goodView = (extra: Record<string, unknown> = {}) => ({
  schema_version: 2, task_id: 'run-5', status: 'completed', reason: 'completed',
  patch_summary: { sha256: 'a'.repeat(64), changed_files: ['solution.py'] },
  checks: [{ command: ['python', '-m', 'unittest'], returncode: 0, network: 'deny' }],
  reviewer: { profile: 'claude', decision: 'approved' }, tests: { status: 'passed', duration_seconds: 1.5 }, risks: [],
  delivery: { state: 'ok', url: 'https://github.com/o/r/pull/7', pr_number: 7, head_sha: 'b'.repeat(40), ci: { conclusion: 'success', completed_at: '2026-11-05T10:00:00Z' } },
  ...extra,
});

test('current evidence shows patch, checks with network state, tester, reviewer and the PR/CI result', () => {
  const evidence = evidenceFromView(goodView());
  assert.equal(evidence.verdict, 'approved');
  const labels = Object.fromEntries(evidence.evidence.map(item => [item.label, item.text]));
  assert.match(labels['Kontrol 1'], /ağ: deny/);
  assert.match(labels['Taslak PR'], /pull\/7 #7/);
  assert.match(labels['CI sonucu'], /^success/);
  assert.equal(labels['Test aşaması'], 'passed · 1.5 sn');
  assert.equal(evidence.reviews[0].decision, 'approved');
  assert.ok(evidence.nextStepText.includes('apply run-5'));
});

test('stale, failed and tampered evidence is never ready to apply', () => {
  for (const view of [goodView({ status: 'stale', reason: 'patch_changed' }), goodView({ status: 'failed', reason: 'test_failed' }),
    goodView({ delivery: { state: 'tampered', receipt: 'github-pr-x.json' } })]) {
    const evidence = evidenceFromView(view);
    assert.equal(evidence.verdict, 'failed');
    assert.equal(evidence.hasPatch, false);
    assert.ok(!evidence.nextStepText.includes('apply run-5'));
  }
});

test('an unknown schema or an unusable payload is refused with a plain message', () => {
  assert.throws(() => evidenceFromView(goodView({ schema_version: 99 })), /uyumsuz/);
  assert.throws(() => evidenceFromView(null), /okunamadı/);
  assert.throws(() => evidenceFromView('text'), /okunamadı/);
});

test('secrets are masked again and fields outside the whitelist never reach the card', () => {
  const evidence = evidenceFromView(goodView({
    risks: ['api_key=SUPERSECRETVALUE leaked'], unreviewed_field: 'EXTRA-DATA',
    checks: [{ command: ['run', 'token=abcd1234efgh'], returncode: 1, network: 'deny' }],
  }));
  const shown = JSON.stringify(evidence);
  assert.ok(!shown.includes('SUPERSECRETVALUE') && !shown.includes('abcd1234efgh') && !shown.includes('EXTRA-DATA'));
  assert.ok(shown.includes('GİZLENDİ'));
});

test('the card offers no approval or apply action of its own', () => {
  const evidence = evidenceFromView(goodView());
  assert.ok(evidence.nextStepText.includes('Hub güvenlik nedeniyle değişiklikleri kendisi uygulamaz'));
  assert.ok(!('approve' in evidence) && !('apply' in evidence));
});
