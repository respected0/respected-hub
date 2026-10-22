import { invoke, isTauri } from '@tauri-apps/api/core';

export interface Settings { python: string; orchestrator_root: string; brain_root: string; data_root: string }
export interface Run { run_id: string; status: string; phase?: string; reason?: string; error?: string;
  request?: { task: string; worker: string; project_root: string }; logs?: Record<string, string>; patch_preview?: string; }
export const native = isTauri();

export async function engine<T>(action: string, runId?: string, request?: unknown): Promise<T> {
  if (!native) throw new Error('Engine işlemleri Tauri masaüstü uygulamasında kullanılabilir.');
  return invoke<T>('engine_request', { action, runId: runId ?? null, request: request ?? null });
}

/** Why the Hub was opened: a product installer (`install`) or Windows Installed apps (`uninstall` / `manage`). */
export interface StartupIntent { action: 'install' | 'uninstall' | 'manage'; product: string }
export const startupIntent = () => invoke<StartupIntent | null>('startup_intent');
export const loadSettings = () => invoke<Settings>('get_settings');
export const saveSettings = (settings: Settings) => invoke<void>('save_settings', { settings });
export interface Products { development: boolean; brain_available: boolean; orchestrator_available: boolean; orchestrator_cli: string | null;
  hub_version: string; engine: { version?: string; workers?: Record<string, string[] | null>; error?: string } }
export const products = () => invoke<Products>('product_status');
export const developmentBuild = () => invoke<boolean>('development_build');
