import test from 'node:test';
import assert from 'node:assert/strict';
import { cached, cachedReply, forget, peek } from '../../src/shared/rppCache.ts';
import { coolSettings } from '../../src/shared/settingsData.ts';

test('the same key shares one running call and the answer is kept', async () => {
  let calls = 0;
  const slow = async () => { calls += 1; await new Promise(resolve => setTimeout(resolve, 10)); return 'answer'; };
  const [a, b] = await Promise.all([cached('t:one', slow), cached('t:one', slow)]);
  assert.deepEqual([a, b, calls, peek('t:one')], ['answer', 'answer', 1, 'answer']);
});

test('a fresh answer is reused, an old one is asked again', async () => {
  let calls = 0;
  const count = async () => ++calls;
  await cached('t:age', count, 60_000);
  assert.equal(await cached('t:age', count, 60_000), 1);
  assert.equal(await cached('t:age', count, 0), 2);
});

test('a failed call is not stored and forget clears by prefix', async () => {
  await assert.rejects(cached('t:fail', async () => { throw new Error('nope'); }));
  assert.equal(peek('t:fail'), undefined);
  await cached('t:keep', async () => 1);
  forget('t:');
  assert.equal(peek('t:keep'), undefined);
});

test('a failed reply reaches every waiting caller but is never stored', async () => {
  let calls = 0;
  const failing = async () => { calls += 1; await new Promise(resolve => setTimeout(resolve, 10)); return { ok: false as const, error: 'down' }; };
  const [a, b] = await Promise.all([cachedReply('t:reply', failing), cachedReply('t:reply', failing)]);
  assert.deepEqual([a.ok, b.ok, calls, peek('t:reply')], [false, false, 1, undefined]);
  await cachedReply('t:reply', async () => ({ ok: true as const, result: 1 }));
  assert.deepEqual(peek('t:reply'), { ok: true, result: 1 });
});

test('a product that changed under us loses its kept answers, and only its own', async () => {
  await cached('brain:manifest', async () => 'old');
  await cached('orchestrator:workers', async () => 'other');
  coolSettings('brain');
  assert.deepEqual([peek('brain:manifest'), peek('orchestrator:workers')], [undefined, 'other']);
  coolSettings('orchestrator');
  assert.equal(peek('orchestrator:workers'), undefined);
});
