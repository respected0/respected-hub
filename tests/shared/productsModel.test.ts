import { test } from 'node:test';
import assert from 'node:assert/strict';
import { changedSettings, formFromSchema, parseFieldValue, summarizeProduct, verifyReadback } from '../../src/shared/productsModel.ts';
import type { FormField } from '../../src/shared/productsModel.ts';
import type { RppResponse, SettingsSchema } from '../../src/shared/rpp.ts';

function ok(result: unknown): RppResponse {
  return { rpp: '1', ok: true, operation_id: null, result, error: null };
}

function fail(code: string, message: string, hint?: string): RppResponse {
  return { rpp: '1', ok: false, operation_id: null, result: null, error: hint ? { code, message, hint } : { code, message } };
}

const manifest = {
  product_id: 'orchestrator',
  product_version: '2.3.0',
  rpp: '1',
  capabilities: ['tasks', 'settings'],
};

test('summarizeProduct: no response is unreachable', () => {
  const summary = summarizeProduct(null, null);
  assert.equal(summary.statusKind, 'unreachable');
  assert.equal(summary.protocolOk, false);
  assert.ok(summary.errorText && summary.errorText.includes('Ürüne ulaşılamadı'));
});

test('summarizeProduct: manifest error envelope is explained in Turkish', () => {
  const summary = summarizeProduct(fail('UNKNOWN_OP', 'unknown op'), undefined);
  assert.equal(summary.statusKind, 'error');
  assert.equal(summary.errorText, 'Bu işlem ürün tarafından tanınmıyor.');
});

test('summarizeProduct: healthy product is ok with capabilities and checks', () => {
  const summary = summarizeProduct(ok(manifest), ok({ status: 'healthy', version: '2.3.0', checks: [{ name: 'store', ok: true, detail: 'erişilebilir' }] }));
  assert.equal(summary.statusKind, 'ok');
  assert.equal(summary.protocolOk, true);
  assert.equal(summary.title, 'orchestrator');
  assert.equal(summary.version, '2.3.0');
  assert.deepEqual(summary.capabilities, ['tasks', 'settings']);
  assert.equal(summary.checks.length, 2);
  assert.equal(summary.checks[0].name, 'Protokol uyumluluğu');
  assert.equal(summary.checks[1].name, 'store');
});

test('summarizeProduct: incompatible protocol fails with a clear reason', () => {
  const summary = summarizeProduct(ok({ ...manifest, rpp: '2' }), ok({ status: 'healthy', version: '2.0.0', checks: [] }));
  assert.equal(summary.statusKind, 'error');
  assert.equal(summary.protocolOk, false);
  assert.ok(summary.errorText && summary.errorText.includes('RPP 1 konuşmuyor'));
});

test('summarizeProduct: malformed manifest result fails negotiation', () => {
  const summary = summarizeProduct(ok(null), ok({ status: 'healthy', version: '1.0', checks: [] }));
  assert.equal(summary.statusKind, 'error');
  assert.equal(summary.protocolOk, false);
  assert.equal(summary.statusText, 'Protokol uyumsuz.');
});

test('summarizeProduct: unhealthy and degraded map to error and warn', () => {
  const unhealthy = summarizeProduct(ok(manifest), ok({ status: 'unhealthy', version: '2.3.0', checks: [{ name: 'db', ok: false, detail: 'kapalı' }] }));
  assert.equal(unhealthy.statusKind, 'error');
  const degraded = summarizeProduct(ok(manifest), ok({ status: 'degraded', version: '2.3.0', checks: [] }));
  assert.equal(degraded.statusKind, 'warn');
});

test('summarizeProduct: health failure and missing health are warnings', () => {
  const failed = summarizeProduct(ok(manifest), fail('BAD_REQUEST', 'bad'));
  assert.equal(failed.statusKind, 'warn');
  assert.equal(failed.errorText, 'İstek biçimi geçersiz.');
  const missing = summarizeProduct(ok(manifest), undefined);
  assert.equal(missing.statusKind, 'warn');
  assert.ok(missing.errorText && missing.errorText.includes('sağlık bilgisi alınamadı'));
});

test('summarizeProduct: malformed health checks do not throw', () => {
  const summary = summarizeProduct(ok(manifest), ok({ status: 'healthy', version: '2.3.0', checks: 'nope' }));
  assert.equal(summary.checks.length, 1);
  assert.equal(summary.statusKind, 'ok');
});

const schema: SettingsSchema = {
  fields: [
    { key: 'root', type: 'string', label: 'Kaynak dizini', default: '/tmp/default', secret: false },
    { key: 'limit', type: 'integer', label: 'Sınır', default: 10, secret: false, min: 1, max: 100 },
    { key: 'enabled', type: 'boolean', label: 'Etkin', default: false, secret: false },
    { key: 'mode', type: 'enum', label: 'Mod', default: 'fast', secret: false, enum: ['fast', 'slow'] },
    { key: 'token', type: 'string', label: 'Token', default: '', secret: true },
  ],
};

test('formFromSchema: never prefills secrets and applies readback and defaults', () => {
  const fields = formFromSchema(schema, { root: '/data', limit: 25, enabled: true, mode: 'slow', token: 'supersecret' });
  const byKey = Object.fromEntries(fields.map((field) => [field.key, field]));
  assert.equal(byKey.token.value, '');
  assert.equal(byKey.token.secret, true);
  assert.equal(byKey.root.value, '/data');
  assert.equal(byKey.limit.value, '25');
  assert.equal(byKey.enabled.value, true);
  assert.equal(byKey.mode.value, 'slow');
  assert.deepEqual(byKey.mode.options, ['fast', 'slow']);
  assert.equal(byKey.limit.min, 1);
  assert.equal(byKey.limit.max, 100);
  assert.ok(!JSON.stringify(fields).includes('supersecret'));
});

test('formFromSchema: falls back to defaults and to the key as label', () => {
  const fields = formFromSchema({ fields: [{ key: 'limit', type: 'integer', label: '', default: 7, secret: false }] }, {});
  assert.equal(fields[0].value, '7');
  assert.equal(fields[0].label, 'limit');
});

test('formFromSchema: hidden fields are never shown', () => {
  const hiddenSchema: SettingsSchema = { fields: [
    { key: 'limit', type: 'integer', label: 'Limit', default: 7, secret: false },
    { key: 'memory_enabled', type: 'boolean', label: 'm', default: false, secret: false, hidden: true },
    { key: 'memory_token', type: 'string', label: 't', default: '', secret: true, hidden: true },
  ] };
  assert.deepEqual(formFromSchema(hiddenSchema, { memory_token: '***set***' }).map(f => f.key), ['limit']);
});

test('formFromSchema: malformed schema yields no fields', () => {
  assert.deepEqual(formFromSchema(null, {}), []);
  assert.deepEqual(formFromSchema({} as SettingsSchema, {}), []);
  assert.deepEqual(formFromSchema({ fields: 'nope' } as unknown as SettingsSchema, null), []);
});

test('parseFieldValue converts each field type', () => {
  const base: FormField = { key: 'k', label: 'k', type: 'string', value: '', secret: false, options: [] };
  assert.equal(parseFieldValue({ ...base, type: 'boolean' }, 'true'), true);
  assert.equal(parseFieldValue({ ...base, type: 'boolean' }, 'false'), false);
  assert.equal(parseFieldValue({ ...base, type: 'integer' }, '42'), 42);
  assert.ok(Number.isNaN(parseFieldValue({ ...base, type: 'integer' }, '') as number));
  assert.equal(parseFieldValue({ ...base, type: 'string' }, 'x'), 'x');
  assert.equal(parseFieldValue({ ...base, type: 'enum' }, 'fast'), 'fast');
});

test('changedSettings: returns only changed keys and tolerates number/string equality', () => {
  const original = { root: '/data', limit: 25, mode: 'fast' };
  const changes = changedSettings(original, { root: '/data', limit: '25', mode: 'slow' });
  assert.deepEqual(changes, { mode: 'slow' });
});

test('changedSettings: untouched secret is omitted, typed secret is sent', () => {
  const original = { root: '/tmp/default', limit: 10, enabled: false, mode: 'fast', token: 'supersecret' };
  const fields = formFromSchema(schema, original);
  const untouched: Record<string, unknown> = {};
  for (const field of fields) {
    const raw = typeof field.value === 'boolean' ? String(field.value) : field.value;
    if (field.secret && raw === '') continue;
    untouched[field.key] = parseFieldValue(field, raw);
  }
  assert.deepEqual(changedSettings(original, untouched), {});
  const typed = { ...untouched, token: 'yeni-gizli' };
  assert.deepEqual(changedSettings(original, typed), { token: 'yeni-gizli' });
});

test('verifyReadback: matches, mismatches and missing keys', () => {
  assert.deepEqual(verifyReadback({ a: 1, b: true }, { a: '1', b: true }), { verified: true, mismatches: [] });
  const mismatch = verifyReadback({ a: 1 }, { a: 2 });
  assert.equal(mismatch.verified, false);
  assert.deepEqual(mismatch.mismatches, [{ key: 'a', expected: '1', actual: '2' }]);
  const missing = verifyReadback({ a: 1 }, {});
  assert.deepEqual(missing.mismatches, [{ key: 'a', expected: '1', actual: '(okunamadı)' }]);
});

test('verifyReadback: redacted secrets count as verified, empty secrets do not', () => {
  assert.equal(verifyReadback({ token: 'yeni' }, { token: '***set***' }).verified, true);
  const empty = verifyReadback({ token: '' }, { token: '***set***' });
  assert.equal(empty.verified, false);
  assert.equal(empty.mismatches[0].key, 'token');
});

test('verifyReadback: malformed readback and no changes', () => {
  assert.equal(verifyReadback({ a: 1 }, null).verified, false);
  assert.equal(verifyReadback({}, null).verified, true);
  assert.equal(verifyReadback(null, null).verified, true);
});
