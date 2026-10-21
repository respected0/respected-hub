import test from 'node:test';
import assert from 'node:assert/strict';
import { describeSemantic } from '../../src/products/secondbrain/memorySemantic.ts';

test('every state explains that normal search keeps working or the data stays local', () => {
  assert.equal(describeSemantic({ supported: true, state: 'off' }).tone, 'unreachable');
  assert.match(describeSemantic({ supported: true, state: 'provider_missing' }).detail, /Ollama/);
  assert.match(describeSemantic({ supported: true, state: 'model_missing', model: 'm' }).detail, /460 MB/);
  const ready = describeSemantic({ supported: true, state: 'ready', vectors: 3, notes: 3, model: 'm' });
  assert.equal(ready.tone, 'ok');
  assert.match(ready.detail, /çıkmaz/);
  assert.equal(describeSemantic({ supported: true, state: 'building', vectors: 1, notes: 3 }).tone, 'warn');
});
