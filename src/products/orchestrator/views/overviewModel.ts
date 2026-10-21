// Pure, testable model for the Orchestrator workspace overview panel.
// Real engine data only: worker CLIs come from engine `status.workers`, run
// counts come from engine `list`. No React, no Tauri imports. Nothing is faked:
// missing engine data degrades to honest Turkish warnings.

import { terminal } from '../tasks.ts';
import type { Run } from '../../../shared/api.ts';

export type RunGroup = 'active' | 'succeeded' | 'failed' | 'other';

export interface RunCounts { active: number; succeeded: number; failed: number; other: number; }

export interface OverviewModel {
  runCounts: RunCounts;
  latestRuns: Run[];
  workersAvailable: string[];
  workersMissing: string[];
  warnings: string[];
}

export const WORKERS_WARNING = 'Worker CLI durumu motor tarafından bildirilmedi.';
export const RUNS_WARNING = 'Çalıştırma listesi alınamadı.';
export const DROPPED_WARNING = 'Bazı çalıştırma kayıtları okunamadı, listeye alınmadı.';
export const LATEST_LIMIT = 5;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function classifyRun(status: unknown): RunGroup {
  const value = typeof status === 'string' ? status : '';
  if (!value) return 'other';
  if (!terminal(value)) return 'active';
  if (value === 'completed') return 'succeeded';
  if (value === 'failed' || value === 'timed_out' || value === 'interrupted' || value === 'invalid') return 'failed';
  return 'other';
}

// The engine `list` reply carries no timestamps, so "newest" is approximated by
// run_id, which is timestamp-prefixed (e.g. run-20261011122324-...). This holds
// whether the engine returns runs oldest- or newest-first.
export function latestRuns(runs: Run[], limit = LATEST_LIMIT): Run[] {
  return [...runs].sort((a, b) => (a.run_id < b.run_id ? 1 : a.run_id > b.run_id ? -1 : 0)).slice(0, limit);
}

function workerAvailable(value: unknown): boolean {
  if (typeof value === 'string') return value.length > 0;
  if (Array.isArray(value)) return value.length > 0;
  return false;
}

function parseRuns(runs: unknown): { list: Run[]; dropped: number } {
  if (!Array.isArray(runs)) return { list: [], dropped: 0 };
  const list: Run[] = [];
  let dropped = 0;
  for (const entry of runs) {
    if (isRecord(entry) && typeof entry.run_id === 'string' && entry.run_id) list.push(entry as unknown as Run);
    else dropped += 1;
  }
  return { list, dropped };
}

export function deriveOverview(status: unknown, runs: unknown): OverviewModel {
  const workersRaw = isRecord(status) && 'workers' in status ? status.workers : status;

  const workersAvailable: string[] = [];
  const workersMissing: string[] = [];
  if (isRecord(workersRaw)) {
    for (const [name, value] of Object.entries(workersRaw)) {
      (workerAvailable(value) ? workersAvailable : workersMissing).push(name);
    }
  }

  const warnings: string[] = [];
  if (!isRecord(workersRaw)) warnings.push(WORKERS_WARNING);

  const runsMissing = !Array.isArray(runs);
  const { list, dropped } = parseRuns(runs);
  if (runsMissing) warnings.push(RUNS_WARNING);
  else if (dropped > 0) warnings.push(DROPPED_WARNING);

  const runCounts: RunCounts = { active: 0, succeeded: 0, failed: 0, other: 0 };
  for (const run of list) runCounts[classifyRun(run.status)] += 1;

  return { runCounts, latestRuns: latestRuns(list), workersAvailable, workersMissing, warnings };
}