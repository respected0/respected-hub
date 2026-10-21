import test from 'node:test';
import assert from 'node:assert/strict';
import { rppCallVia } from '../src/shared/productClient.ts';

const okEnvelope = JSON.stringify({
  rpp: '1.0', ok: true, operation_id: 'op-1', result: { status: 'healthy' }, error: null,
});

test('rppCallVia forwards product/op/payload/operationId to the transport', async () => {
  const seen: unknown[] = [];
  const transport = async (invocation: unknown) => { seen.push(invocation); return okEnvelope; };
  const response = await rppCallVia(transport, 'orchestrator', 'health', { deep: true }, 'op-1');
  assert.deepEqual(seen, [{ product: 'orchestrator', op: 'health', payload: { deep: true }, operationId: 'op-1' }]);
  assert.equal(response.ok, true);
  if (response.ok) assert.deepEqual(response.result, { status: 'healthy' });
});

test('rppCallVia parses object replies and generates an apply operation id', async () => {
  let operationId: string | null = null;
  const transport = async (invocation: { operationId: string | null }) => {
    operationId = invocation.operationId;
    return { rpp: '1.0', ok: false, operation_id: invocation.operationId, result: null,
      error: { code: 'INVALID_SETTINGS', message: 'bad', hint: 'fix' } };
  };
  const response = await rppCallVia(transport, 'secondbrain', 'settings.apply', { settings: {} });
  assert.match(operationId as string, /^[A-Za-z0-9_-]{8,64}$/);
  assert.equal(response.ok, false);
  if (!response.ok) { assert.equal(response.error.code, 'INVALID_SETTINGS'); assert.equal(response.error.hint, 'fix'); }
});

test('rppCallVia maps malformed replies to BAD_RESPONSE without throwing', async () => {
  const replies = [
    'not json',
    '[]',
    JSON.stringify({ rpp: '9.9', ok: true, operation_id: null, result: {}, error: null }),
    JSON.stringify({ rpp: '1.0', ok: true, result: {}, error: null }),
  ];
  for (const reply of replies) {
    const response = await rppCallVia(async () => reply, 'orchestrator', 'manifest');
    assert.equal(response.ok, false, reply);
    if (!response.ok) assert.equal(response.error.code, 'BAD_RESPONSE', reply);
  }
});