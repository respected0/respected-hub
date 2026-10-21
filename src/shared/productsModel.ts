// Pure, testable model for the Products view. No React, no Tauri imports.
import { describeError, negotiate, SECRET_REDACTED } from './rpp.ts';
import type { Manifest, RppError, RppResponse, SettingType, SettingsField, SettingsSchema } from './rpp.ts';

export type ProductStatusKind = 'ok' | 'warn' | 'error' | 'unreachable';

export interface ProductCheck { name: string; ok: boolean; detail: string; }

export interface ProductSummary {
  title: string;
  version: string;
  protocolOk: boolean;
  statusKind: ProductStatusKind;
  statusText: string;
  checks: ProductCheck[];
  capabilities: string[];
  errorText?: string;
}

const UNREACHABLE_HINT = 'Ürün yanıt vermedi. Kurulu ve erişilebilir olduğundan emin olun, sonra "Yenile" ile tekrar deneyin.';

function plainText(error: RppError | null | undefined): string {
  const plain = describeError(error);
  return plain.hint ? `${plain.message} ${plain.hint}` : plain.message;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function summarizeProduct(
  manifestResponse: RppResponse | null | undefined,
  healthResponse: RppResponse | null | undefined,
): ProductSummary {
  if (!manifestResponse) {
    return {
      title: '', version: '', protocolOk: false, statusKind: 'unreachable',
      statusText: 'Ürüne ulaşılamadı.', checks: [], capabilities: [],
      errorText: `Ürüne ulaşılamadı: üründen yanıt alınamadı. ${UNREACHABLE_HINT}`,
    };
  }
  if (!manifestResponse.ok) {
    return {
      title: '', version: '', protocolOk: false, statusKind: 'error',
      statusText: 'Ürün bilgisi alınamadı.', checks: [], capabilities: [],
      errorText: plainText(manifestResponse.error),
    };
  }

  const manifest = isRecord(manifestResponse.result) ? (manifestResponse.result as unknown as Partial<Manifest>) : null;
  const negotiation = negotiate(manifest);
  const capabilities = negotiation.capabilities;
  const title = manifest && typeof manifest.product_id === 'string' ? manifest.product_id : '';
  const version = manifest && typeof manifest.product_version === 'string' ? manifest.product_version : '';
  const range = manifest && manifest.protocol ? `${manifest.protocol.min}-${manifest.protocol.max}` : '';
  const protocolCheck: ProductCheck = {
    name: 'Protokol uyumluluğu',
    ok: negotiation.compatible,
    detail: negotiation.compatible ? `RPP ${range} destekleniyor.` : (negotiation.reason ?? 'Protokol bilgisi yok.'),
  };

  if (!negotiation.compatible) {
    return {
      title, version, protocolOk: false, statusKind: 'error', statusText: 'Protokol uyumsuz.',
      checks: [protocolCheck], capabilities,
      errorText: `${negotiation.reason ?? 'Ürün protokol aralığını bildirmedi.'} Hub bu protokol sürümüyle konuşamıyor; ürünü veya hub sürümünü güncelleyin.`,
    };
  }

  if (!healthResponse) {
    return {
      title, version, protocolOk: true, statusKind: 'warn', statusText: 'Sağlık bilgisi alınamadı.',
      checks: [protocolCheck], capabilities,
      errorText: `Ürün yanıt veriyor ama sağlık bilgisi alınamadı. ${UNREACHABLE_HINT}`,
    };
  }
  if (!healthResponse.ok) {
    return {
      title, version, protocolOk: true, statusKind: 'warn', statusText: 'Sağlık bilgisi alınamadı.',
      checks: [protocolCheck], capabilities,
      errorText: plainText(healthResponse.error),
    };
  }

  const health = isRecord(healthResponse.result) ? healthResponse.result : {};
  const rawChecks: unknown[] = Array.isArray(health.checks) ? health.checks : [];
  const checks: ProductCheck[] = [protocolCheck, ...rawChecks.map((entry) => {
    const item = isRecord(entry) ? entry : {};
    return {
      name: typeof item.name === 'string' && item.name ? item.name : 'Kontrol',
      ok: item.ok === true,
      detail: typeof item.detail === 'string' ? item.detail : '',
    };
  })];
  const healthVersion = typeof health.version === 'string' ? health.version : '';
  const status = health.status;

  if (status === 'healthy') {
    return { title, version: version || healthVersion, protocolOk: true, statusKind: 'ok', statusText: 'Çalışıyor. Tüm kontroller başarılı.', checks, capabilities };
  }
  if (status === 'unhealthy') {
    return {
      title, version: version || healthVersion, protocolOk: true, statusKind: 'error', statusText: 'Çalışmıyor. Sağlık kontrolleri başarısız.',
      checks, capabilities, errorText: 'Ürün çalışmıyor: sağlık kontrolleri başarısız. Ayrıntı için kontrollere bakın.',
    };
  }
  return { title, version: version || healthVersion, protocolOk: true, statusKind: 'warn', statusText: 'Kısmen çalışıyor. Bazı kontroller başarısız.', checks, capabilities };
}

export interface FormField {
  key: string;
  label: string;
  type: SettingType;
  value: string | boolean;
  secret: boolean;
  options: string[];
  min?: number;
  max?: number;
}

const SETTING_TYPES: SettingType[] = ['string', 'integer', 'boolean', 'enum'];

function normalizeType(type: unknown): SettingType {
  return SETTING_TYPES.includes(type as SettingType) ? (type as SettingType) : 'string';
}

function asBoolean(value: unknown): boolean {
  return value === true || value === 'true';
}

export function formFromSchema(
  schema: SettingsSchema | null | undefined,
  readbackSettings: Record<string, unknown> | null | undefined,
): FormField[] {
  const fields = Array.isArray(schema?.fields) ? (schema?.fields as SettingsField[]) : [];
  const settings = isRecord(readbackSettings) ? readbackSettings : {};
  return fields.filter((field) => !!field && typeof field.key === 'string' && field.hidden !== true).map((field) => {
    const secret = field.secret === true;
    const original = settings[field.key];
    const source = original === undefined ? field.default : original;
    const type = normalizeType(field.type);
    let value: string | boolean;
    if (type === 'boolean') value = asBoolean(source);
    else if (secret) value = '';
    else value = source === undefined || source === null ? '' : String(source);
    const result: FormField = {
      key: field.key,
      label: typeof field.label === 'string' && field.label ? field.label : field.key,
      type, value, secret,
      options: Array.isArray(field.enum) ? field.enum.filter((option) => typeof option === 'string') : [],
    };
    if (typeof field.min === 'number') result.min = field.min;
    if (typeof field.max === 'number') result.max = field.max;
    return result;
  });
}

export function parseFieldValue(field: FormField, raw: string): unknown {
  if (field.type === 'boolean') return raw === 'true';
  if (field.type === 'integer') return raw.trim() === '' ? NaN : Number(raw);
  return raw;
}

function sameValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  const numeric = (value: unknown) => typeof value === 'number' || typeof value === 'string';
  return numeric(a) && numeric(b) && String(a) === String(b);
}

export function changedSettings(
  original: Record<string, unknown> | null | undefined,
  edited: Record<string, unknown> | null | undefined,
): Record<string, unknown> {
  const base = isRecord(original) ? original : {};
  const changes: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(isRecord(edited) ? edited : {})) {
    if (value === undefined) continue;
    if (!(key in base) || !sameValue(base[key], value)) changes[key] = value;
  }
  return changes;
}

export interface ReadbackMismatch { key: string; expected: string; actual: string; }

function show(value: unknown): string {
  return value === undefined || value === null ? '' : String(value);
}

export function verifyReadback(
  intended: Record<string, unknown> | null | undefined,
  readback: Record<string, unknown> | null | undefined,
): { verified: boolean; mismatches: ReadbackMismatch[] } {
  const settings = isRecord(readback) ? readback : {};
  const mismatches: ReadbackMismatch[] = [];
  for (const [key, expected] of Object.entries(isRecord(intended) ? intended : {})) {
    if (!(key in settings)) {
      mismatches.push({ key, expected: show(expected), actual: '(okunamadı)' });
      continue;
    }
    const actual = settings[key];
    // Readback never returns secret values; a redaction marker means the value is set.
    if (actual === SECRET_REDACTED) {
      if (show(expected) === '') mismatches.push({ key, expected: '', actual: '(değişmedi)' });
      continue;
    }
    if (!sameValue(expected, actual)) mismatches.push({ key, expected: show(expected), actual: show(actual) });
  }
  return { verified: mismatches.length === 0, mismatches };
}
