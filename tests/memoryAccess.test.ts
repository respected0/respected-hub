import { test } from 'node:test';
import assert from 'node:assert/strict';
import { describeMemoryError, describeMemoryStatus, memoryAccessDisable, memoryAccessEnable, memoryAccessStatus } from '../src/products/secondbrain/memoryAccess.ts';
import type { MemoryStatus } from '../src/products/secondbrain/memoryAccess.ts';

const base: MemoryStatus = { supported: true, enabled: false, grant_id: null, token_set: false, pending_revoke: false, problems: [] };

test('describeMemoryStatus: state matrix', () => {
  assert.equal(describeMemoryStatus({ ...base, supported: false }).state, 'unsupported');
  assert.equal(describeMemoryStatus(base).state, 'off');
  assert.equal(describeMemoryStatus({ ...base, enabled: true, grant_id: 'g', token_set: true }).state, 'on');
  assert.equal(describeMemoryStatus({ ...base, pending_revoke: true }).state, 'pending-revoke');
  // a pending revoke wins over everything else
  assert.equal(describeMemoryStatus({ ...base, supported: false, enabled: true, pending_revoke: true }).state, 'pending-revoke');
  for (const s of [base, { ...base, enabled: true }]) {
    const d = describeMemoryStatus(s);
    assert.ok(d.title && d.detail);
  }
});

test('describeMemoryError: message then hint, nothing else', () => {
  assert.equal(describeMemoryError({ code: 'X', message: 'Hata.', hint: 'Dene.' }), 'Hata. Dene.');
  assert.equal(describeMemoryError({ code: 'X', message: 'Hata.', hint: '' }), 'Hata.');
  assert.ok(describeMemoryError({ code: 'X', message: '', hint: '' }).length > 0);
});

test('wrappers fail honestly outside the desktop app', async () => {
  for (const reply of [await memoryAccessStatus(), await memoryAccessEnable(), await memoryAccessDisable()]) {
    assert.equal(reply.ok, false);
    assert.equal(reply.ok ? '' : reply.error.code, 'UNAVAILABLE');
  }
});
