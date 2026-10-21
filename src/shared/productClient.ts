// Thin typed wrapper over the private RPP entry point ('rpp_request' command).
// Outside Tauri the call rejects with a clear message; rppCallVia stays injectable for unit tests.
import { invoke } from '@tauri-apps/api/core';
import { native } from './api.ts';
import { buildRequest, parseResponse } from './rpp.ts';
import type { RppResponse } from './rpp.ts';

export type RppProduct = 'secondbrain' | 'orchestrator';

export type RppInvocation = {
  product: RppProduct;
  op: string;
  payload: Record<string, unknown> | null;
  operationId: string | null;
};

export type RppTransport = (invocation: RppInvocation) => Promise<unknown>;

export async function rppCallVia(
  transport: RppTransport,
  product: RppProduct,
  op: string,
  payload?: Record<string, unknown>,
  operationId?: string,
): Promise<RppResponse> {
  const request = buildRequest(op, payload ?? {}, operationId);
  const reply = await transport({ product, op, payload: payload ?? null, operationId: request.operation_id });
  return parseResponse(typeof reply === 'string' ? reply : JSON.stringify(reply));
}

export async function rppCall(
  product: RppProduct,
  op: string,
  payload?: Record<string, unknown>,
  operationId?: string,
): Promise<RppResponse> {
  if (!native) throw new Error('RPP çağrıları Tauri masaüstü uygulamasında kullanılabilir.');
  return rppCallVia((invocation) => invoke<unknown>('rpp_request', invocation), product, op, payload, operationId);
}

// Never throws: null means the product could not be reached at all.
export async function rppTry(product: RppProduct, op: string, payload?: Record<string, unknown>, operationId?: string): Promise<RppResponse | null> {
  try { return await rppCall(product, op, payload, operationId); } catch { return null; }
}
