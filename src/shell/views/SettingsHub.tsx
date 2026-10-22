import { SETTINGS_TABS, type SettingsTab } from '../apps.ts';
import type { Ecosystem } from '../useEcosystem.ts';
import Activity from './Activity.tsx';
import Connections from './Connections.tsx';
import type { Go } from './Home.tsx';
import Settings from './Settings.tsx';

const INTRO: Record<SettingsTab, string> = {
  general: 'Hub\'ın güncelleme kanalı ve kaynak yolları; Hub ve ürünlerin bilgisayarınıza neler kurduğu.',
  status: 'Hub kurulu her ürünle kendi protokolü (RPP) üzerinden konuşur. Bir şey çalışmıyorsa nedenini buradan okuyun; ürün kurmak ya da kaldırmak için Kütüphane\'yi kullanın.',
  activity: 'Kurulum işlemleri ve Orchestrator motorunun gerçek görev kayıtları.',
};

// Hub settings, product status and the activity history in one place: they are diagnostics and configuration, not destinations.
export default function SettingsHub({ tab, onTab, eco, go, onCatalogChanged }: { tab: SettingsTab; onTab: (tab: SettingsTab) => void; eco: Ecosystem; go: Go; onCatalogChanged: () => void }) {
  return <div className="page">
    <div className="page-head"><div><span className="eyebrow">Ayarlar</span><h1>{SETTINGS_TABS.find(item => item.id === tab)?.label}</h1><p>{INTRO[tab]}</p></div></div>
    <div className="tabs" role="tablist" aria-label="Ayar bölümleri">{SETTINGS_TABS.map(item =>
      <button key={item.id} role="tab" className="tab" aria-selected={tab === item.id} onClick={() => onTab(item.id)}>{item.label}</button>)}</div>
    <div className="tab-panel" role="tabpanel">
      {tab === 'general' && <Settings onCatalogChanged={onCatalogChanged} />}
      {tab === 'status' && <Connections />}
      {tab === 'activity' && <Activity eco={eco} go={go} />}
    </div>
  </div>;
}
