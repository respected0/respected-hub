import { test } from 'node:test';
import assert from 'node:assert/strict';
import { describeIndexError, describeIndexStatus, describeRefreshResult, memoryIndexRefresh, memoryIndexStatus } from '../src/products/secondbrain/memoryIndex.ts';
import type { IndexState, IndexStatus } from '../src/products/secondbrain/memoryIndex.ts';

const base: IndexStatus = { supported: true, state: 'fresh', indexed_files: 7, changed_files: 0, vault_id: 'v', last_refresh: null, problems: [] };

test('describeIndexStatus: every state', () => {
  const states: IndexState[] = ['unsupported', 'no-vault', 'missing', 'invalid', 'fresh', 'stale', 'unknown'];
  for (const state of states) {
    const d = describeIndexStatus({ ...base, state });
    assert.equal(d.state, state);
    assert.ok(d.title && d.detail);
    if (state !== 'fresh') assert.ok(!/^güncel$/i.test(d.title));
  }
});

test('describeIndexStatus: unsupported wins; unknown is honest', () => {
  assert.equal(describeIndexStatus({ ...base, supported: false }).state, 'unsupported');
  const u = describeIndexStatus({ ...base, state: 'unknown' });
  assert.match(u.detail, /belirlenemedi/);
  assert.equal(describeIndexStatus({ ...base, state: 'stale', changed_files: 3 }).title, 'Güncel değil');
});

test('describeRefreshResult: counts and duration', () => {
  const s = describeRefreshResult({ indexed: 5, skipped: 2, deleted: 1, total: 7, duration_seconds: 3, state: 'fresh' });
  for (const part of ['5', '2', '1', '7', '3 sn']) assert.ok(s.includes(part));
});

test('describeIndexError: message then hint, nothing else', () => {
  assert.equal(describeIndexError({ code: 'X', message: 'Hata.', hint: 'Dene.' }), 'Hata. Dene.');
  assert.equal(describeIndexError({ code: 'X', message: 'Hata.', hint: '' }), 'Hata.');
  assert.ok(describeIndexError({ code: 'X', message: '', hint: '' }).length > 0);
});

test('wrappers fail honestly outside the desktop app', async () => {
  for (const reply of [await memoryIndexStatus(), await memoryIndexRefresh(), await memoryIndexRefresh(true)]) {
    assert.equal(reply.ok, false);
    assert.equal(reply.ok ? '' : reply.error.code, 'UNAVAILABLE');
  }
});
