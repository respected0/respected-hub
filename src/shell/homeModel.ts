// Pure decision: which Home experience matches the real ecosystem state. No React/Tauri.
import type { ProductStatusKind } from '../shared/productsModel.ts';

export interface AppSnapshot { loading: boolean; kind: ProductStatusKind; }
type HomeMode = 'preview' | 'loading' | 'first-launch' | 'attention' | 'active';

export function homeMode(native: boolean, apps: AppSnapshot[]): HomeMode {
  if (!native) return 'preview';
  if (!apps.length) return 'first-launch';   // nothing installed yet (the shell lists only installed products)
  if (apps.some(app => app.loading)) return 'loading';
  if (apps.every(app => app.kind === 'unreachable')) return 'first-launch';
  if (apps.some(app => app.kind !== 'ok')) return 'attention';
  return 'active';
}
