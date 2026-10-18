import { invoke, isTauri } from '@tauri-apps/api/core';

export interface Settings { python: string; orchestrator_root: string; brain_root: string; data_root: string }
export interface Run { run_id: string; status: string; phase?: string; reason?: string; error?: string;
  request?: { task: string; worker: string; project_root: string }; logs?: Record<string, string>; patch_preview?: string; }
export const native = isTauri();

export async function engine<T>(action: string, runId?: string, request?: unknown): Promise<T> {
  if (!native) throw new Error('Engine işlemleri Tauri masaüstü uygulamasında kullanılabilir.');
  return invoke<T>('engine_request', { action, runId: runId ?? null, request: request ?? null });
}

export const startupIntent = () => invoke<string | null>('startup_intent');
export const loadSettings = () => invoke<Settings>('get_settings');
export const saveSettings = (settings: Settings) => invoke<void>('save_settings', { settings });
export interface Products { development: boolean; brain_available: boolean; orchestrator_available: boolean;
  hub_version: string; engine: { version?: string; workers?: Record<string, string[] | null>; error?: string } }
export const products = () => invoke<Products>('product_status');
