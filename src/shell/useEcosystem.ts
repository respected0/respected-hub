// Live ecosystem state for the shell: per-app RPP summary + config source, and engine runs.
import { useCallback, useEffect, useRef, useState } from 'react';
import { engine, native, type Run } from '../shared/api.ts';
import { APPS } from './apps.ts';
import { rppTry } from '../shared/productClient.ts';
import { activityList, lifecycleStatus, type JournalEntry, type LifecycleOverview } from './lifecycle.ts';
import { summarizeProduct, type ProductSummary } from '../shared/productsModel.ts';
import { onReturn } from '../shared/focus.ts';
import { coolSettings, warmBrainSettings, warmOrchestratorSettings } from '../shared/settingsData.ts';
import { terminal } from '../products/orchestrator/tasks.ts';

interface AppRuntime { loading: boolean; summary: ProductSummary; }
export interface Ecosystem { apps: Record<string, AppRuntime>; runs: Run[]; lifecycle: LifecycleOverview | null; journal: JournalEntry[]; refresh: () => void;
  // the product whose install/update/repair/removal is running: its processes are stopped on purpose, so it is not reported as a problem meanwhile
  busy: string | null; setBusy: (product: string | null) => void; }

const pending: AppRuntime = { loading: true, summary: summarizeProduct(null, null) };

export function useEcosystem(): Ecosystem {
  const [apps, setApps] = useState<Record<string, AppRuntime>>(() => Object.fromEntries(APPS.map(app => [app.id, pending])));
  const [runs, setRuns] = useState<Run[]>([]);
  const [lifecycle, setLifecycle] = useState<LifecycleOverview | null>(null);
  const [journal, setJournal] = useState<JournalEntry[]>([]);
  const [busy, setBusy] = useState<string | null>(null);

  // one refresh at a time (each spawns several engine processes); a request made meanwhile reruns once afterwards
  const inFlight = useRef(false);
  const rerun = useRef(false);
  // The manifest only changes with the installed version, so it is asked again only then (Orchestrator's takes ~570 ms).
  const manifests = useRef<Record<string, { version: string | null; response: Awaited<ReturnType<typeof rppTry>> }>>({});
  const load = useCallback(async () => {
    if (!native) { setApps(Object.fromEntries(APPS.map(app => [app.id, { ...pending, loading: false }]))); return; }
    // every part is shown as soon as it is read: the slow product calls never hold back the install state, and one product never waits for the other
    const runsDone = engine<{ runs: Run[] }>('list').then(reply => setRuns(reply.runs)).catch(() => setRuns([]));
    const history = activityList(50).then(reply => setJournal(reply.ok ? reply.result : []));
    const overview = await lifecycleStatus();
    setLifecycle(overview.ok ? overview.result : null);
    const products = overview.ok ? overview.result.products : [];
    const absent = new Set(products.filter(item => item?.install_state === 'absent').map(item => item!.product));
    await Promise.all(APPS.map(async app => {
      if (!app.rpp) { setApps(before => ({ ...before, [app.id]: { ...pending, loading: false } })); return; }
      // not installed: nothing to poll, so no error banner for a product that was never there
      if (absent.has(app.id)) { setApps(before => ({ ...before, [app.id]: { loading: false, summary: { ...summarizeProduct(null, null), statusText: 'Kurulu değil', errorText: undefined } } })); return; }
      const version = products.find(item => item?.product === app.id)?.version ?? null;
      const known = manifests.current[app.id];
      if (known && known.version !== version) coolSettings(app.rpp);  // an update or repair: the kept answers are the old program's
      const [manifest, health] = await Promise.all([
        known && known.version === version && known.response?.ok ? known.response : rppTry(app.rpp, 'manifest'),
        rppTry(app.rpp, 'health')]);
      manifests.current[app.id] = { version, response: manifest };
      setApps(before => ({ ...before, [app.id]: { loading: false, summary: summarizeProduct(manifest, health) } }));
    }));
    if (!absent.has('brain')) warmBrainSettings();
    if (!absent.has('orchestrator')) warmOrchestratorSettings();  // so its Settings page opens without waiting for the programs
    await Promise.all([runsDone, history]);
  }, []);

  // a refresh that nobody asked for (window focus, a timer) is skipped when the last one finished moments ago
  const lastDone = useRef(0);
  const refresh = useCallback(async (force = true) => {
    if (inFlight.current) { if (force) rerun.current = true; return; }
    if (!force && Date.now() - lastDone.current < 2500) return;
    do {
      rerun.current = false; inFlight.current = true;
      try { await load(); } finally { inFlight.current = false; lastDone.current = Date.now(); }
    } while (rerun.current);
  }, [load]);

  // Read on open and whenever the person returns to the window (installers and command lines change things outside the Hub);
  // a timer runs only while a task is in flight, because that state moves on its own.
  useEffect(() => { void refresh(); return onReturn(() => void refresh(false)); }, [refresh]);
  const working = runs.some(run => !terminal(run.status));
  useEffect(() => {
    if (!working) return;
    const timer = window.setInterval(() => void refresh(false), 3000);
    return () => window.clearInterval(timer);
  }, [working, refresh]);

  return { apps, runs, lifecycle, journal, refresh: () => void refresh(), busy, setBusy };
}
