import { invoke, isTauri } from '@tauri-apps/api/core';
import type { CatalogEntry, IntegrationReply, InventoryResult, Project } from './integrationsTypes.ts';

export interface Settings { python: string; orchestrator_root: string; brain_root: string; data_root: string }
export interface RunUsage {
  raw_input_tokens?: number; cached_input_tokens?: number; output_tokens?: number;
  reasoning_tokens?: number | null; estimated_weighted_tokens?: number;
}
export interface RunRequest {
  task: string; worker: string; project_root: string; master?: string; kind?: string;
  ownership?: string[]; forbidden?: string[]; acceptance?: string[][]; worker_profile?: string;
  created_at?: string; timeout_seconds?: number; workflow?: Record<string, unknown>;
}
export interface RunCheck { command: string[]; returncode: number }
export interface Run {
  run_id: string; status: string; phase?: string; reason?: string; error?: string;
  request?: RunRequest; logs?: Record<string, string>; patch_preview?: string;
  usage?: RunUsage | null; model?: string | null; usage_status?: string; token_budget?: number;
  changed_files?: string[]; generated_artifacts?: string[]; generated_artifact_count?: number; artifact_images?: string[];
  checks?: RunCheck[]; duration_seconds?: number; created_at?: string; updated_at?: string;
  worker_incomplete?: string;
}
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

/** Read-only third-party inventory. Never writes to a client configuration. */
export async function thirdPartyInventory(): Promise<InventoryResult> {
  if (!native) throw new Error('Entegrasyon envanteri yalnızca masaüstü uygulamasında okunur.');
  const reply = await invoke<IntegrationReply>('integrations_request', { request: { action: 'inventory' } });
  if (!reply.ok || !reply.result || !('inventory' in reply.result)) throw new Error(reply.error?.message ?? 'Envanter okunamadı.');
  return reply.result;
}

/** Compiled-in third-party catalog. Read-only and local: it works with the network offline. */
export async function thirdPartyCatalog(): Promise<CatalogEntry[]> {
  if (!native) throw new Error('Katalog yalnızca masaüstü uygulamasında okunur.');
  const reply = await invoke<IntegrationReply>('integrations_request', { request: { action: 'catalog' } });
  if (!reply.ok || !reply.result || !('entries' in reply.result)) throw new Error(reply.error?.message ?? 'Katalog okunamadı.');
  return reply.result.entries;
}

/** User-triggered signed catalog refresh. Offline or invalid downloads leave the last verified catalog intact. */
export async function refreshThirdPartyCatalog(): Promise<CatalogEntry[]> {
  if (!native) throw new Error('Katalog yalnızca masaüstü uygulamasında yenilenir.');
  const reply = await invoke<IntegrationReply>('integrations_request', { request: { action: 'catalog_refresh' } });
  if (!reply.ok || !reply.result || !('entries' in reply.result)) throw new Error(reply.error?.message ?? 'Katalog yenilenemedi.');
  return reply.result.entries;
}

/** Hub-owned project registry; selecting a root never writes an AI client config. */
export const listProjects = () => invoke<Project[]>('projects_list');
export const addProject = (root: string) => invoke<Project[]>('projects_add', { root });
export const removeProject = (id: string) => invoke<Project[]>('projects_remove', { id });
