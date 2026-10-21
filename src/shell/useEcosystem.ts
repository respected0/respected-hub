// Live ecosystem state for the shell: per-app RPP summary + config source, and engine runs.
import { useCallback, useEffect, useRef, useState } from 'react';
import { engine, native, type Run } from '../shared/api.ts';
import { APPS } from './apps.ts';
import { rppTry } from '../shared/productClient.ts';
import { activityList, lifecycleStatus, type JournalEntry, type LifecycleOverview } from './lifecycle.ts';
import { summarizeProduct, type ProductSummary } from '../shared/productsModel.ts';

export interface AppRuntime { loading: boolean; summary: ProductSummary; configSource: 'file' | 'defaults' | null; }
export interface Ecosystem { apps: Record<string, AppRuntime>; runs: Run[]; lifecycle: LifecycleOverview | null; journal: JournalEntry[]; refresh: () => void; }

const pending: AppRuntime = { loading: true, summary: summarizeProduct(null, null), configSource: null };

export function useEcosystem(): Ecosystem {
  const [apps, setApps] = useState<Record<string, AppRuntime>>(() => Object.fromEntries(APPS.map(app => [app.id, pending])));
  const [runs, setRuns] = useState<Run[]>([]);
  const [lifecycle, setLifecycle] = useState<LifecycleOverview | null>(null);
  const [journal, setJournal] = useState<JournalEntry[]>([]);

  // one refresh at a time (each spawns several engine processes); a request made meanwhile reruns once afterwards
  const inFlight = useRef(false);
  const rerun = useRef(false);
  const load = useCallback(async () => {
    if (!native) { setApps(Object.fromEntries(APPS.map(app => [app.id, { ...pending, loading: false }]))); return; }
    const [overview, history] = await Promise.all([lifecycleStatus(), activityList(50)]);
    const absent = new Set((overview.ok ? overview.result.products : []).filter(item => item?.install_state === 'absent').map(item => item!.product));
    const entries = await Promise.all(APPS.map(async app => {
      if (!app.rpp) return [app.id, { ...pending, loading: false }] as const;
      // not installed: nothing to poll, so no error banner for a product that was never there
      if (absent.has(app.id)) return [app.id, { loading: false, summary: { ...summarizeProduct(null, null), statusText: 'Kurulu değil', errorText: undefined }, configSource: null }] as const;
      const [manifest, health, readback] = await Promise.all([rppTry(app.rpp, 'manifest'), rppTry(app.rpp, 'health'), rppTry(app.rpp, 'settings.readback')]);
      const source = readback?.ok && typeof readback.result === 'object' && readback.result ? (readback.result as { source?: string }).source : null;
      return [app.id, { loading: false, summary: summarizeProduct(manifest, health), configSource: source === 'file' || source === 'defaults' ? source : null }] as const;
    }));
    setApps(Object.fromEntries(entries));
    try { setRuns((await engine<{ runs: Run[] }>('list')).runs); } catch { setRuns([]); }
    setLifecycle(overview.ok ? overview.result : null); setJournal(history.ok ? history.result : []);
  }, []);

  const refresh = useCallback(async (force = true) => {
    if (inFlight.current) { if (force) rerun.current = true; return; }
    inFlight.current = true;
    try { await load(); } finally { inFlight.current = false; }
    if (rerun.current) { rerun.current = false; await refresh(false); }
  }, [load]);

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => void refresh(false), 8000);
    return () => window.clearInterval(timer);
  }, [refresh]);

  return { apps, runs, lifecycle, journal, refresh: () => void refresh() };
}
