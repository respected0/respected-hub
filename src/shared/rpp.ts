// Respected Product Protocol (RPP) v1.0/v1.1 - shared TypeScript contract.
// One JSON request in, one JSON response line out. RPP is not MCP and not a daemon.

export const RPP_VERSION = '1.0';
export const RPP_VERSION_MAX = '1.1';
export const V11_OPS = ['lifecycle.plan', 'lifecycle.apply', 'lifecycle.status', 'lifecycle.rollback', 'operation.receipt'] as const;
const ID_REQUIRED_OPS = ['settings.apply', 'lifecycle.apply', 'lifecycle.rollback'];
const SUPPORTED_VERSIONS = [RPP_VERSION, RPP_VERSION_MAX];

export function versionForOp(op: string): string {
  return (V11_OPS as readonly string[]).includes(op) ? RPP_VERSION_MAX : RPP_VERSION;
}
export const SECRET_REDACTED = '***set***';

export interface RppProtocolRange { min: string; max: string; }

export interface RppRequest {
  rpp: string;
  op: string;
  operation_id: string | null;
  payload: Record<string, unknown>;
}

export interface RppError { code: string; message: string; hint?: string; }

export interface RppOk { rpp: string; ok: true; operation_id: string | null; result: unknown; error: null; }
export interface RppFail { rpp: string; ok: false; operation_id: string | null; result: null; error: RppError; }
export type RppResponse = RppOk | RppFail;

export interface Manifest {
  product_id: string;
  product_version: string;
  protocol: RppProtocolRange;
  capabilities: string[];
  settings_supported: boolean;
}

export interface HealthCheck { name: string; ok: boolean; detail: string; }
export interface Health { status: 'healthy' | 'degraded' | 'unhealthy'; version: string; checks: HealthCheck[]; }

export type SettingType = 'string' | 'integer' | 'boolean' | 'enum';
export interface SettingsField {
  key: string;
  type: SettingType;
  label: string;
  default: unknown;
  secret: boolean;
  hidden?: boolean;
  enum?: string[];
  min?: number;
  max?: number;
}
export interface SettingsSchema { fields: SettingsField[]; }
export interface SettingsValidation { valid: boolean; errors: { key: string; message: string }[]; }
export interface SettingsReadback { settings: Record<string, unknown>; source: 'file' | 'defaults'; }

export interface Negotiation { compatible: boolean; reason?: string; capabilities: string[]; }
export interface PlainError { message: string; messageEn: string; hint?: string; }

export function buildRequest(op: string, payload: Record<string, unknown> = {}, operationId?: string): RppRequest {
  const operation_id = operationId ?? (ID_REQUIRED_OPS.includes(op) ? crypto.randomUUID() : null);
  return { rpp: versionForOp(op), op, operation_id, payload };
}

export function parseResponse(text: string): RppResponse {
  const bad = (message: string): RppFail => ({
    rpp: RPP_VERSION, ok: false, operation_id: null, result: null,
    error: { code: 'BAD_RESPONSE', message, hint: 'Ürünün protokol sürümünü ve yanıt biçimini kontrol edin.' },
  });

  let data: unknown;
  try { data = JSON.parse(text); } catch { return bad('Yanıt geçerli JSON değil.'); }
  if (typeof data !== 'object' || data === null || Array.isArray(data)) return bad('Yanıt bir JSON nesnesi değil.');

  const o = data as Record<string, unknown>;
  if (typeof o.rpp !== 'string' || !SUPPORTED_VERSIONS.includes(o.rpp)) return bad(`Beklenmeyen protokol sürümü: ${String(o.rpp)}.`);
  if (typeof o.ok !== 'boolean') return bad('Yanıtta "ok" alanı boolean değil.');
  if (!('operation_id' in o) || (o.operation_id !== null && typeof o.operation_id !== 'string')) {
    return bad('Yanıtta "operation_id" alanı yok veya metin/null değil.');
  }
  const operation_id = o.operation_id as string | null;
  const rpp = o.rpp;

  if (o.ok) {
    if (o.error != null) return bad('Başarılı yanıt hata alanı içermemeli.');
    if (!('result' in o)) return bad('Başarılı yanıtta "result" alanı yok.');
    return { rpp, ok: true, operation_id, result: o.result, error: null };
  }

  if (o.result !== null) return bad('Hatalı yanıtta "result" null olmalı.');

  const e = o.error;
  if (typeof e !== 'object' || e === null || Array.isArray(e)) return bad('Hatalı yanıtta "error" nesnesi yok.');
  const err = e as Record<string, unknown>;
  if (typeof err.code !== 'string' || !/^[A-Z][A-Z0-9_]*$/.test(err.code)) return bad('Hata kodu UPPER_SNAKE biçiminde değil.');
  if (typeof err.message !== 'string' || !err.message) return bad('Hata mesajı boş veya metin değil.');
  if (err.hint != null && typeof err.hint !== 'string') return bad('Hata ipucu metin değil.');

  return {
    rpp, ok: false, operation_id, result: null,
    error: typeof err.hint === 'string'
      ? { code: err.code, message: err.message, hint: err.hint }
      : { code: err.code, message: err.message },
  };
}

function versionWeight(version: string): number {
  const m = /^(\d+)\.(\d+)$/.exec(version);
  return m ? Number(m[1]) * 1000 + Number(m[2]) : NaN;
}

export function negotiate(
  manifest: Partial<Manifest> | null | undefined,
  hubSupported: RppProtocolRange = { min: RPP_VERSION, max: RPP_VERSION_MAX },
): Negotiation {
  const capabilities = Array.isArray(manifest?.capabilities)
    ? manifest.capabilities.filter((c): c is string => typeof c === 'string')
    : [];

  const protocol = manifest?.protocol;
  if (!protocol || typeof protocol.min !== 'string' || typeof protocol.max !== 'string') {
    return { compatible: false, reason: 'Ürün protokol aralığını bildirmedi.', capabilities };
  }

  const hubMin = versionWeight(hubSupported.min), hubMax = versionWeight(hubSupported.max);
  const peerMin = versionWeight(protocol.min), peerMax = versionWeight(protocol.max);
  if ([hubMin, hubMax, peerMin, peerMax].some(Number.isNaN)) {
    return { compatible: false, reason: 'Protokol sürümü major.minor biçiminde değil.', capabilities };
  }
  if (Math.max(hubMin, peerMin) > Math.min(hubMax, peerMax)) {
    return {
      compatible: false,
      reason: `Örtüşen protokol sürümü yok (ürün ${protocol.min}-${protocol.max}, hub ${hubSupported.min}-${hubSupported.max}).`,
      capabilities,
    };
  }
  return { compatible: true, capabilities };
}

const PLAIN: Record<string, { tr: string; en: string }> = {
  UNKNOWN_OP: { tr: 'Bu işlem ürün tarafından tanınmıyor.', en: 'The product does not recognise this operation.' },
  PROTOCOL_UNSUPPORTED: { tr: 'Ürün bu protokol sürümünü desteklemiyor.', en: 'The product does not support this protocol version.' },
  BAD_REQUEST: { tr: 'İstek biçimi geçersiz.', en: 'The request format is invalid.' },
  BAD_RESPONSE: { tr: 'Üründen gelen yanıt beklenen biçimde değil.', en: 'The response from the product has an unexpected format.' },
  INVALID_SETTINGS: { tr: 'Ayarlar geçersiz; hiçbir değişiklik kaydedilmedi.', en: 'The settings are invalid; nothing was saved.' },
  UNSUPPORTED_OP: { tr: 'Ürün bu işlemi bu protokol sürümünde sunmuyor. Ürünü güncelleyin.', en: 'The product does not offer this operation in this protocol version.' },
  LIFECYCLE_CONFLICT: { tr: 'Bu işlem kimliği farklı bir kurulum eylemi için zaten kullanılmış.', en: 'This operation id was already used for a different lifecycle action or target.' },
  LIFECYCLE_PARTIAL: { tr: 'İşlem kısmen uygulandı; sonraki adım başarısız oldu. Geri alma veya onarım gerekli.', en: 'The operation was partly applied; a later step failed.' },
  VERIFY_FAILED: { tr: 'İşlem sonrası doğrulama başarısız oldu; sonuç onaylanmadı.', en: 'Post-apply verification failed; the result is not accepted.' },
  ROLLBACK_UNAVAILABLE: { tr: 'Geri alınamaz bir adım uygulandığı için otomatik geri alma yapılamıyor.', en: 'Rollback is unavailable because an irreversible step was applied.' },
  PERMISSION_DENIED: { tr: 'Bu adım için gereken izin verilmedi.', en: 'A step needs a permission that was not granted.' },
  RECEIPT_NOT_FOUND: { tr: 'Bu işlem kimliği için kayıtlı bir makbuz bulunamadı.', en: 'No receipt exists for this operation id.' },
  OPERATION_ID_CONFLICT: { tr: 'Bu işlem kimliği farklı ayarlarla daha önce kullanılmış.', en: 'This operation id was already used with different settings.' },
};

export function describeError(error: RppError | null | undefined): PlainError {
  const known = error ? PLAIN[error.code] : undefined;
  const base = known ?? {
    tr: error?.message ? `İşlem başarısız: ${error.message}` : 'İşlem başarısız ve nedeni bilinmiyor.',
    en: error?.message ?? 'The operation failed for an unknown reason.',
  };
  return error?.hint ? { message: base.tr, messageEn: base.en, hint: error.hint } : { message: base.tr, messageEn: base.en };
}