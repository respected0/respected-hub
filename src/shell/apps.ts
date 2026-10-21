// Registry of ecosystem apps the shell knows about. Adding an app = adding an entry;
// `rpp` is set only for products that speak the Respected Product Protocol.
import type { RppProduct } from '../shared/productClient.ts';
import { ORCHESTRATOR_PAGES } from '../products/orchestrator/views/workspaceModel.ts';

export interface AppPage { id: string; label: string; }
export interface AppDef { id: string; name: string; tagline: string; rpp?: RppProduct; accent: 'blue' | 'violet'; pages: AppPage[]; }
export type HubRoute = 'home' | 'apps' | 'discover' | 'connections' | 'activity' | 'settings';
export const HUB_NAV: { id: HubRoute; label: string }[] = [
  { id: 'home', label: 'Ana Sayfa' },
  { id: 'apps', label: 'Uygulamalarım' },
  { id: 'discover', label: 'Keşfet' },
  { id: 'connections', label: 'Ürün durumu' },
  { id: 'activity', label: 'Etkinlik' },
  { id: 'settings', label: 'Ayarlar' },
];

import type { LifecycleOverview } from './lifecycle.ts';

/** Workspaces appear only for installed products (a browser preview has no lifecycle, so it shows all). */
export function visibleApps(lifecycle: LifecycleOverview | null, native: boolean): AppDef[] {
  return native && lifecycle ? APPS.filter(app => lifecycle?.products.some(item => item?.product === app.id && item.installed)) : APPS;
}

export const APPS: AppDef[] = [
  { id: 'secondbrain', name: 'Respected SecondBrain', rpp: 'secondbrain', accent: 'violet',
    tagline: 'Ajanlar de\u011fi\u015fir; ortak haf\u0131za kasan\u0131zda kal\u0131r.',
    pages: [{ id: 'overview', label: 'Genel bak\u0131\u015f' }, { id: 'memory', label: 'Hafıza' }, { id: 'setup', label: 'Kurulum' }, { id: 'settings', label: 'Ayarlar' }] },
  { id: 'orchestrator', name: 'Respected Orchestrator', rpp: 'orchestrator', accent: 'blue',
    tagline: '\u00c7ok ajanl\u0131 g\u00f6rev motoru: izole \u00e7al\u0131\u015ft\u0131r, do\u011frula, incele.',
    pages: ORCHESTRATOR_PAGES },
];
