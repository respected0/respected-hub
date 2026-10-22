import test from 'node:test';
import assert from 'node:assert/strict';
import { plainCheck } from '../../src/shared/healthText.ts';
import { excerpt } from '../../src/products/brain/noteExcerpt.ts';

test('known checks read as plain sentences, bad ones keep the product detail', () => {
  assert.equal(plainCheck('vault_registered', true, 'Vault registered: C:\Notes').text, 'Kasa bağlı: C:\Notes');
  assert.match(plainCheck('git_available', false, 'git not found on PATH').text, /Git bulunamadı.*git not found/);
  assert.equal(plainCheck('worker_cli', true, 'claude, codex').text, 'Kullanılabilir: claude, codex');
});

test('an unknown check is shown as the product reported it', () => {
  assert.deepEqual(plainCheck('something_new', true, 'fine'), { label: 'something_new', text: 'fine' });
});

test('excerpt drops front matter, marks and blank lines and shortens long lines', () => {
  const lines = excerpt('---\ntitle: x\n---\n# Son Oturum\n\n- **Tarih:** 2026-10-22\n' + 'a'.repeat(300), 3, 20);
  assert.deepEqual(lines, ['Son Oturum', 'Tarih: 2026-10-22', `${'a'.repeat(19)}…`]);
});
