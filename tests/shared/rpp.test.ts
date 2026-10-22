import test from 'node:test';
import assert from 'node:assert/strict';
import {
  RPP_VERSION,
  buildRequest,
  parseResponse,
  negotiate,
  describeError,
} from '../../src/shared/rpp.ts';

const okEnvelope = JSON.stringify({
  rpp: '1', ok: true, operation_id: 'op-123', result: { status: 'healthy' }, error: null,
});
const errEnvelope = JSON.stringify({
  rpp: '1', ok: false, operation_id: null, result: null,
  error: { code: 'UNKNOWN_OP', message: 'Bilinmeyen işlem.', hint: 'manifest çağırın.' },
});

test('buildRequest echoes op/payload and marks non-required ops without an id', () => {
  const req = buildRequest('health', { deep: true });
  assert.equal(req.rpp, RPP_VERSION);
  assert.equal(req.op, 'health');
  assert.equal(req.operation_id, null);
  assert.deepEqual(req.payload, { deep: true });
});

test('buildRequest generates a valid operation_id for settings.apply and keeps explicit ones', () => {
  const id = buildRequest('settings.apply', { settings: {} }).operation_id;
  assert.equal(typeof id, 'string');
  assert.match(id as string, /^[A-Za-z0-9_-]{8,64}$/);
  assert.equal(buildRequest('settings.apply', {}, 'chosen_id').operation_id, 'chosen_id');
});

test('parseResponse accepts a well-formed success envelope', () => {
  const res = parseResponse(okEnvelope);
  assert.equal(res.ok, true);
  if (res.ok) {
    assert.equal(res.operation_id, 'op-123');
    assert.deepEqual(res.result, { status: 'healthy' });
    assert.equal(res.error, null);
  }
});

test('parseResponse accepts a well-formed error envelope', () => {
  const res = parseResponse(errEnvelope);
  assert.equal(res.ok, false);
  if (!res.ok) {
    assert.equal(res.error.code, 'UNKNOWN_OP');
    assert.equal(res.error.hint, 'manifest çağırın.');
    assert.equal(res.result, null);
  }
});

test('parseResponse maps malformed input to BAD_RESPONSE and never throws', () => {
  const inputs = [
    'not json',
    '[]',
    'null',
    JSON.stringify({ rpp: '9.9', ok: true, operation_id: null, result: {}, error: null }),
    JSON.stringify({ rpp: '1', ok: 'yes', operation_id: null, result: {}, error: null }),
    JSON.stringify({ rpp: '1', ok: false, operation_id: null, result: null, error: null }),
    JSON.stringify({ rpp: '1', ok: false, operation_id: null, result: null, error: { code: 'lower', message: 'x' } }),
    JSON.stringify({ rpp: '1', ok: false, operation_id: null, result: null, error: { code: 'X', message: '' } }),
    JSON.stringify({ rpp: '1', ok: true, operation_id: null, result: {}, error: { code: 'X', message: 'y' } }),
    JSON.stringify({ rpp: '1', ok: true, result: {}, error: null }),
    JSON.stringify({ rpp: '1', ok: true, operation_id: 42, result: {}, error: null }),
    JSON.stringify({ rpp: '1', ok: true, operation_id: null, error: null }),
    JSON.stringify({ rpp: '1', ok: false, operation_id: null, result: {}, error: { code: 'UNKNOWN_OP', message: 'x' } }),
  ];
  for (const input of inputs) {
    const res = parseResponse(input);
    assert.equal(res.ok, false, input);
    if (!res.ok) assert.equal(res.error.code, 'BAD_RESPONSE', input);
  }
});


test('describeError covers every contract code in Turkish with an English fallback', () => {
  for (const code of ['UNKNOWN_OP', 'UNSUPPORTED_VERSION', 'BAD_REQUEST', 'BAD_RESPONSE', 'INVALID_SETTINGS', 'OPERATION_ID_CONFLICT']) {
    const plain = describeError({ code, message: 'raw', hint: 'do this' });
    assert.ok(plain.message.length > 0, code);
    assert.ok(plain.messageEn.length > 0, code);
    assert.equal(plain.message, describeError({ code, message: 'raw' }).message, code);
    assert.equal(plain.hint, 'do this');
  }
  const unknown = describeError({ code: 'SOMETHING_NEW', message: 'Detay.' });
  assert.ok(unknown.message.includes('Detay.'));
  assert.equal(unknown.hint, undefined);
  assert.equal(describeError(null).message.length > 0, true);
});



test('every typed error has a Turkish message', () => {
  for (const code of ['LIFECYCLE_CONFLICT', 'LIFECYCLE_PARTIAL', 'VERIFY_FAILED', 'PERMISSION_DENIED']) {
    assert.ok(!describeError({ code, message: 'x' }).message.startsWith('İşlem başarısız:'), code);
  }
});

test('one protocol version: requests carry rpp 1, replies with any other version are refused, manifests are checked', () => {
  assert.equal(buildRequest('health').rpp, '1');
  assert.ok(buildRequest('settings.apply').operation_id);
  assert.equal(buildRequest('integrations.plan').operation_id, null);
  for (const version of ['1.0', '1.1', '2']) {
    const reply = parseResponse(JSON.stringify({ rpp: version, ok: true, operation_id: null, result: {}, error: null }));
    assert.ok(!reply.ok && reply.error.code === 'BAD_RESPONSE', version);
  }
  const speaking = negotiate({ rpp: '1', capabilities: ['health', 'settings.apply'] });
  assert.equal(speaking.compatible, true);
  assert.deepEqual(speaking.capabilities, ['health', 'settings.apply']);
  for (const manifest of [{ rpp: '1.1', capabilities: [] }, { capabilities: ['health'] }, null]) {
    const answer = negotiate(manifest);
    assert.equal(answer.compatible, false);
    assert.ok(answer.reason);
  }
});
