// Respected Product Protocol (RPP) 1 - shared TypeScript contract (docs/PRODUCT_CONTRACT.md).
// One JSON request in, one JSON response line out. RPP is not MCP and not a daemon.

export const RPP_VERSION = '1';
const ID_REQUIRED_OPS = ['settings.apply', 'integrations.apply', 'integrations.rollback'];
export const SECRET_REDACTED = '***set***';

interface RppRequest {
  rpp: string;
  op: string;
  operation_id: string | null;
  payload: Record<string, unknown>;
}

export interface RppError { code: string; message: string; hint?: string; }

interface RppOk { rpp: string; ok: true; operation_id: string | null; result: unknown; error: null; }
interface RppFail { rpp: string; ok: false; operation_id: string | null; result: null; error: RppError; }
export type RppResponse = RppOk | RppFail;

export interface Manifest {
  product_id: string;
  product_version: string;
  rpp: string;
  capabilities: string[];
  installed: boolean;
  install_state: string;
  roots: { app: string | null; data: string; vault: string | null };
}

export type SettingType = 'string' | 'integer' | 'boolean' | 'enum';
export interface SettingsField {
  key: string;
  type: SettingType;
  label: string;
  help?: string;
  default: unknown;
  secret: boolean;
  hidden?: boolean;
  enum?: string[];
  min?: number;
  max?: number;
}
export interface SettingsSchema { fields: SettingsField[]; }
export interface SettingsValidation { valid: boolean; errors: { key: string; message: string }[]; }

interface Negotiation { compatible: boolean; reason?: string; capabilities: string[]; }
interface PlainError { message: string; messageEn: string; hint?: string; }

export function buildRequest(op: string, payload: Record<string, unknown> = {}, operationId?: string): RppRequest {
  const operation_id = operationId ?? (ID_REQUIRED_OPS.includes(op) ? crypto.randomUUID() : null);
  return { rpp: RPP_VERSION, op, operation_id, payload };
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
  if (o.rpp !== RPP_VERSION) return bad(`Beklenmeyen protokol sürümü: ${String(o.rpp)}.`);
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

/** The product speaks this Hub's protocol: there is one version, "1". */
export function negotiate(manifest: Partial<Manifest> | null | undefined): Negotiation {
  const capabilities = Array.isArray(manifest?.capabilities) ? manifest.capabilities.filter((c): c is string => typeof c === 'string') : [];
  if (manifest?.rpp === RPP_VERSION) return { compatible: true, capabilities };
  return { compatible: false, reason: `Ürün RPP ${RPP_VERSION} konuşmuyor (${manifest?.rpp ?? 'sürüm bildirmedi'}).`, capabilities };
}

const PLAIN: Record<string, { tr: string; en: string }> = {
  UNKNOWN_OP: { tr: 'Bu işlem ürün tarafından tanınmıyor.', en: 'The product does not recognise this operation.' },
  UNSUPPORTED_VERSION: { tr: 'Ürün bu protokol sürümünü desteklemiyor.', en: 'The product does not support this protocol version.' },
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