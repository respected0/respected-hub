// Pure model for the global shell: top-bar health pill and breadcrumb text. No React/Tauri.
import type { ProductStatusKind } from '../shared/productsModel.ts';

export interface Health { tone: 'ok' | 'warn' | 'error' | 'unreachable' | 'idle'; label: string; }

export function globalHealth(native: boolean, loading: boolean, kinds: ProductStatusKind[]): Health {
  if (!native) return { tone: 'idle', label: 'Tarayıcı önizlemesi' };
  if (loading) return { tone: 'idle', label: 'Kontrol ediliyor…' };
  if (!kinds.length) return { tone: 'idle', label: 'Henüz ürün kurulu değil' };
  const ok = kinds.filter(kind => kind === 'ok').length;
  if (ok === kinds.length) return { tone: 'ok', label: `${ok}/${kinds.length} ürün hazır` };
  if (kinds.every(kind => kind === 'unreachable')) return { tone: 'unreachable', label: 'Ürün bağlı değil' };
  return { tone: kinds.some(kind => kind === 'error') ? 'error' : 'warn', label: `${ok}/${kinds.length} ürün hazır` };
}
