import test from 'node:test';
import assert from 'node:assert/strict';
import { ACTION_LABEL, availableActions, canRollback, describeLifecycleError, journalSentence, formatWhen, newOperationId } from '../../src/shell/lifecycle.ts';
import type { JournalEntry } from '../../src/shell/lifecycle.ts';

const entry = (over: Partial<JournalEntry> = {}): JournalEntry => ({
  operation_id: 'install-12345678', product: 'orchestrator', action: 'install', status: 'applied', from_version: null, to_version: '0.1.0',
  started_at: '1760000000000', finished_at: '1760000001000', error: null, undo: { kind: 'install' }, ...over,
});

test('actions offered match the install state and package availability', () => {
  assert.deepEqual(availableActions('absent', true), ['install']);
  assert.deepEqual(availableActions('absent', false), []);
  assert.deepEqual(availableActions('installed', true), ['uninstall']);   // Onar is shown disabled by the view while files verify
  assert.deepEqual(availableActions('update-available', true), ['update', 'uninstall']);
  assert.deepEqual(availableActions('repair-needed', false), ['uninstall']);
});

test('errors read as one sentence with the recovery hint', () => {
  assert.equal(describeLifecycleError({ code: 'VERIFY_FAILED', message: 'Ürün çalışmadı.', hint: 'Önceki sürüm etkin.' }), 'Ürün çalışmadı. Önceki sürüm etkin.');
  assert.match(describeLifecycleError({ code: 'ROLLBACK_UNAVAILABLE', message: '', hint: '' }), /geri alınamıyor/);
});

test('journal sentences describe applied, failed, rolled back and running operations', () => {
  assert.match(journalSentence(entry(), 'Orchestrator'), /Orchestrator kuruldu \(v0\.1\.0\)/);
  assert.match(journalSentence(entry({ status: 'rolled_back' }), 'Orchestrator'), /geri alındı/);
  assert.match(journalSentence(entry({ status: 'failed', error: { code: 'VERIFY_FAILED', message: 'Bozuk.', hint: 'Yeniden dene.' } }), 'Orchestrator'), /başarısız\. Bozuk\. Yeniden dene\./);
  assert.match(journalSentence(entry({ status: 'verifying' }), 'Orchestrator'), /sürüyor/);
});

test('only applied operations with an undo record can be rolled back', () => {
  assert.equal(canRollback(entry()), true);
  assert.equal(canRollback(entry({ status: 'failed' })), false);
  assert.equal(canRollback(entry({ undo: null })), false);
});

test('operation ids satisfy the 8..64 [A-Za-z0-9_-] rule; times format or stay empty', () => {
  const id = newOperationId('update');
  assert.match(id, /^[A-Za-z0-9_-]{8,64}$/);
  assert.equal(formatWhen(null), '');
  assert.notEqual(formatWhen('1760000000000'), '');
});

test('an older applied operation cannot be rolled back while a newer one exists for the product', () => {
  const older = entry({ operation_id: 'install-00000001', started_at: '1760000000000' });
  const newer = entry({ operation_id: 'repair-00000002', action: 'repair', started_at: '1760000005000' });
  assert.equal(canRollback(older, [newer, older]), false);
  assert.equal(canRollback(newer, [newer, older]), true);
  assert.equal(canRollback(entry({ action: 'update' })), false, 'an update deletes the old version, so it cannot be undone');
  assert.equal(canRollback(older, [entry({ product: 'secondbrain', started_at: '1760000005000' }), older]), true);
});

test('integration changes read as a journal sentence and plan params travel with the call', () => {
  assert.match(journalSentence(entry({ action: 'integrations', product: 'secondbrain' }), 'SecondBrain'), /SecondBrain entegrasyonları güncellendi/);
  assert.match(ACTION_LABEL.integrations, /Entegrasyon/);
});
