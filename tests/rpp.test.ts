import test from 'node:test';
import assert from 'node:assert/strict';
import {
  RPP_VERSION,
  buildRequest,
  parseResponse,
  negotiate,
  describeError,
} from '../src/rpp.ts';

const okEnvelope = JSON.stringify({
  rpp: '1.0', ok: true, operation_id: 'op-123', result: { status: 'healthy' }, error: null,
});
const errEnvelope = JSON.stringify({
  rpp: '1.0', ok: false, operation_id: null, result: null,
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
    JSON.stringify({ rpp: '1.0', ok: 'yes', operation_id: null, result: {}, error: null }),
    JSON.stringify({ rpp: '1.0', ok: false, operation_id: null, result: null, error: null }),
    JSON.stringify({ rpp: '1.0', ok: false, operation_id: null, result: null, error: { code: 'lower', message: 'x' } }),
    JSON.stringify({ rpp: '1.0', ok: false, operation_id: null, result: null, error: { code: 'X', message: '' } }),
    JSON.stringify({ rpp: '1.0', ok: true, operation_id: null, result: {}, error: { code: 'X', message: 'y' } }),
    JSON.stringify({ rpp: '1.0', ok: true, result: {}, error: null }),
    JSON.stringify({ rpp: '1.0', ok: true, operation_id: 42, result: {}, error: null }),
    JSON.stringify({ rpp: '1.0', ok: true, operation_id: null, error: null }),
    JSON.stringify({ rpp: '1.0', ok: false, operation_id: null, result: {}, error: { code: 'UNKNOWN_OP', message: 'x' } }),
  ];
  for (const input of inputs) {
    const res = parseResponse(input);
    assert.equal(res.ok, false, input);
    if (!res.ok) assert.equal(res.error.code, 'BAD_RESPONSE', input);
  }
});

test('negotiate compares protocol ranges numerically and keeps advertised capabilities', () => {
  const manifest = { protocol: { min: '1.0', max: '1.0' }, capabilities: ['settings.apply', 'health'] };
  const same = negotiate(manifest);
  assert.equal(same.compatible, true);
  assert.deepEqual(same.capabilities, ['settings.apply', 'health']);

  const hubAhead = negotiate({ protocol: { min: '0.9', max: '1.0' }, capabilities: [] });
  assert.equal(hubAhead.compatible, true);

  const mismatch = negotiate({ protocol: { min: '2.0', max: '2.1' }, capabilities: [] });
  assert.equal(mismatch.compatible, false);
  assert.ok(mismatch.reason);

  const missing = negotiate({ capabilities: ['health'] });
  assert.equal(missing.compatible, false);
  assert.deepEqual(missing.capabilities, ['health']);

  const malformed = negotiate({ protocol: { min: 'v1', max: '1.0' }, capabilities: [] });
  assert.equal(malformed.compatible, false);
});

test('describeError covers every contract code in Turkish with an English fallback', () => {
  for (const code of ['UNKNOWN_OP', 'PROTOCOL_UNSUPPORTED', 'BAD_REQUEST', 'BAD_RESPONSE', 'INVALID_SETTINGS', 'OPERATION_ID_CONFLICT']) {
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
test('v1.1 ops are sent under rpp 1.1 with a required operation id; v1.0 ops stay on 1.0', () => {
  const plan = buildRequest('lifecycle.plan', { action: 'install' });
  assert.equal(plan.rpp, '1.1');
  assert.equal(plan.operation_id, null);
  assert.ok(buildRequest('lifecycle.apply').operation_id);
  assert.ok(buildRequest('lifecycle.rollback').operation_id);
  assert.equal(buildRequest('health').rpp, '1.0');
});

test('parseResponse accepts 1.1, keeps the version and rejects unknown versions', () => {
  const ok = parseResponse(JSON.stringify({ rpp: '1.1', ok: true, operation_id: null, result: {}, error: null }));
  assert.ok(ok.ok && ok.rpp === '1.1');
  const bad = parseResponse(JSON.stringify({ rpp: '2.0', ok: true, operation_id: null, result: {}, error: null }));
  assert.ok(!bad.ok && bad.error.code === 'BAD_RESPONSE');
});

test('negotiate accepts a v1.0-only product and a 1.0..1.1 product, rejects 2.x', () => {
  assert.equal(negotiate({ protocol: { min: '1.0', max: '1.0' }, capabilities: ['a'] }).compatible, true);
  assert.equal(negotiate({ protocol: { min: '1.0', max: '1.1' }, capabilities: [] }).compatible, true);
  assert.equal(negotiate({ protocol: { min: '2.0', max: '2.1' }, capabilities: [] }).compatible, false);
});

test('every v1.1 typed error has a Turkish message', () => {
  for (const code of ['UNSUPPORTED_OP', 'LIFECYCLE_CONFLICT', 'LIFECYCLE_PARTIAL', 'VERIFY_FAILED', 'ROLLBACK_UNAVAILABLE', 'PERMISSION_DENIED', 'RECEIPT_NOT_FOUND']) {
    assert.ok(!describeError({ code, message: 'x' }).message.startsWith('İşlem başarısız:'), code);
  }
});
