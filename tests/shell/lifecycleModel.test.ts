import test from 'node:test';
import assert from 'node:assert/strict';
import { ACTION_LABEL, availableActions, describeLifecycleError, journalSentence, formatWhen, newOperationId } from '../../src/shell/lifecycle.ts';
import type { JournalEntry } from '../../src/shell/lifecycle.ts';

const entry = (over: Partial<JournalEntry> = {}): JournalEntry => ({
  operation_id: 'install-12345678', product: 'orchestrator', action: 'install', status: 'applied', from_version: null, to_version: '0.1.0',
  started_at: '1760000000000', finished_at: '1760000001000', error: null, ...over,
});

test('actions offered match the install state and the installer in the catalog', () => {
  assert.deepEqual(availableActions('absent', true), ['install']);
  assert.deepEqual(availableActions('absent', false), []);
  assert.deepEqual(availableActions('installed', true), ['repair', 'uninstall'], 'running the installer again is the repair');
  assert.deepEqual(availableActions('installed', false), ['uninstall'], 'a removal needs no catalog');
  assert.deepEqual(availableActions('update-available', true), ['update', 'uninstall']);
  assert.deepEqual(availableActions('repair-needed', false), ['uninstall']);
});

test('errors read as one sentence with the recovery hint', () => {
  assert.equal(describeLifecycleError({ code: 'VERIFY_FAILED', message: 'Ürün çalışmadı.', hint: 'Onarın.' }), 'Ürün çalışmadı. Onarın.');
  assert.match(describeLifecycleError({ code: 'VERIFY_FAILED', message: '', hint: '' }), /Doğrulama başarısız/);
});

test('journal sentences describe applied, failed and running operations', () => {
  assert.match(journalSentence(entry(), 'Orchestrator'), /Orchestrator kuruldu \(v0\.1\.0\)/);
  assert.match(journalSentence(entry({ status: 'failed', error: { code: 'VERIFY_FAILED', message: 'Bozuk.', hint: 'Yeniden dene.' } }), 'Orchestrator'), /başarısız\. Bozuk\. Yeniden dene\./);
  assert.match(journalSentence(entry({ status: 'verifying' }), 'Orchestrator'), /sürüyor/);
  assert.equal(Object.keys(ACTION_LABEL).sort().join(), 'install,repair,uninstall,update', 'nothing the installer route cannot do');
});

test('operation ids satisfy the 8..64 [A-Za-z0-9_-] rule; times format or stay empty', () => {
  assert.match(newOperationId('update'), /^[A-Za-z0-9_-]{8,64}$/);
  assert.equal(formatWhen(null), '');
  assert.notEqual(formatWhen('1760000000000'), '');
});
