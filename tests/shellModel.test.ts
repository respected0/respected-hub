import test from 'node:test';
import assert from 'node:assert/strict';
import { globalHealth } from '../src/shellModel.ts';

test('preview and loading never claim product state', () => {
  assert.equal(globalHealth(false, false, ['ok']).tone, 'idle');
  assert.equal(globalHealth(true, true, ['ok', 'ok']).label, 'Kontrol ediliyor…');
});
test('health counts only ok products', () => {
  assert.deepEqual(globalHealth(true, false, ['ok', 'ok']), { tone: 'ok', label: '2/2 ürün hazır' });
  assert.equal(globalHealth(true, false, ['ok', 'warn']).tone, 'warn');
  assert.equal(globalHealth(true, false, ['ok', 'error']).tone, 'error');
  assert.equal(globalHealth(true, false, ['unreachable', 'unreachable']).label, 'Ürün bağlı değil');
});
