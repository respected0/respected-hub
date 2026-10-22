// Registry of ecosystem apps the shell knows about. Adding an app = adding an entry;
// `rpp` is set only for products that speak the Respected Product Protocol.
import type { RppProduct } from '../shared/productClient.ts';
import { ORCHESTRATOR_PAGES } from './workspaceModel.ts';

interface AppPage { id: string; label: string; description: string; }
export interface AppDef { id: string; name: string; tagline: string; rpp?: RppProduct; accent: 'blue' | 'violet'; pages: AppPage[]; }
export type HubRoute = 'home' | 'discover' | 'apps' | 'settings';
/** Settings holds the Hub's own settings and its activity history; product health lives in each product's Overview. */
export type SettingsTab = 'general' | 'activity';
export const HUB_NAV: { id: HubRoute; label: string }[] = [
  { id: 'home', label: 'Ana Sayfa' },
  { id: 'discover', label: 'Keşfet' },
  { id: 'apps', label: 'Kütüphane' },
  { id: 'settings', label: 'Ayarlar' },
];
export const SETTINGS_TABS: { id: SettingsTab; label: string }[] = [
  { id: 'general', label: 'Genel' },
  { id: 'activity', label: 'Etkinlik' },
];

import type { LifecycleOverview } from './lifecycle.ts';

/** Workspaces appear only for installed products (a browser preview has no lifecycle, so it shows all). */
export function visibleApps(lifecycle: LifecycleOverview | null, native: boolean): AppDef[] {
  return native && lifecycle ? APPS.filter(app => lifecycle?.products.some(item => item?.product === app.id && item.installed)) : APPS;
}

export const APPS: AppDef[] = [
  { id: 'brain', name: 'Respected Brain', rpp: 'brain', accent: 'violet',
    tagline: 'Ajanlar de\u011fi\u015fir; ortak haf\u0131za kasan\u0131zda kal\u0131r.',
    pages: [
      { id: 'overview', label: 'Genel bakış', description: 'Ürünün sağlığı ve nereden başlayacağınız.' },
      { id: 'memory', label: 'Hafıza', description: 'Notlarda arayın, okuyun, hızlı not yakalayın, sabah özetini görün.' },
      { id: 'setup', label: 'Kurulum', description: 'AI araç bağlantıları, MCP izinleri ve not kasası.' },
      { id: 'settings', label: 'Ayarlar', description: 'Brain ayarları, hafıza dizini ve anlamsal arama.' }] },
  { id: 'orchestrator', name: 'Respected Orchestrator', rpp: 'orchestrator', accent: 'blue',
    tagline: '\u00c7ok ajanl\u0131 g\u00f6rev motoru: izole \u00e7al\u0131\u015ft\u0131r, do\u011frula, incele.',
    pages: ORCHESTRATOR_PAGES },
];
