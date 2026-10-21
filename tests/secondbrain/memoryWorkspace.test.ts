import { test } from 'node:test';
import assert from 'node:assert/strict';
import { KNOWN_NOTES, describeSearchMode, describeWorkspaceError } from '../../src/products/secondbrain/memoryWorkspace.ts';

test('known notes resolve to the engine-maintained paths, today substituted', () => {
  const paths = KNOWN_NOTES.map(note => note.path('2026-10-16'));
  assert.ok(paths.includes('daily/2026-10-16.md') && paths.includes('🔮 850-Companion/Last-Session.md'));
  assert.ok(paths.every(path => path.endsWith('.md') && !path.includes('..')));
});

test('search mode text is honest about semantic search being unavailable', () => {
  assert.equal(describeSearchMode({ mode: 'fts', semantic: 'off' }), 'Tam metin araması');
  assert.match(describeSearchMode({ mode: 'fts', semantic: 'unavailable' }), /kullanılamıyor/);
  assert.match(describeSearchMode({ mode: 'hybrid', semantic: 'ok' }), /anlamsal/);
});

test('errors show the engine message and hint only', () => {
  assert.equal(describeWorkspaceError({ code: 'X', message: 'Yok.', hint: 'Yenile.' }), 'Yok. Yenile.');
  assert.equal(describeWorkspaceError({ code: 'X', message: '', hint: '' }), 'İşlem başarısız.');
});
