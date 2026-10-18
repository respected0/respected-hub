// Pure decision: which Home experience matches the real ecosystem state. No React/Tauri.
import type { ProductStatusKind } from './views/productsModel.ts';

export interface AppSnapshot { loading: boolean; kind: ProductStatusKind; configSource: 'file' | 'defaults' | null; }
export type HomeMode = 'preview' | 'loading' | 'first-launch' | 'attention' | 'setup-incomplete' | 'active';

export function homeMode(native: boolean, apps: AppSnapshot[]): HomeMode {
  if (!native) return 'preview';
  if (!apps.length) return 'first-launch';   // nothing installed yet (the shell lists only installed products)
  if (apps.some(app => app.loading)) return 'loading';
  if (apps.every(app => app.kind === 'unreachable')) return 'first-launch';
  if (apps.some(app => app.kind !== 'ok')) return 'attention';
  if (apps.some(app => app.configSource === 'defaults')) return 'setup-incomplete';
  return 'active';
}
