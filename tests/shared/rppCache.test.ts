import test from 'node:test';
import assert from 'node:assert/strict';
import { cached, forget, peek } from '../../src/shared/rppCache.ts';

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
