import { native } from '../shared/api.ts';
import { rppCall } from '../shared/productClient.ts';
import { cached, peek } from '../shared/rppCache.ts';

// The person's name and the assistant's name are Brain settings; one read serves the greeting, Home and every revisit.
export interface Identity { user: string; assistant: string; }
const KEY = 'brain:identity';
const text = (value: unknown) => typeof value === 'string' ? value.trim() : '';

export const knownIdentity = (): Identity | undefined => peek<Identity>(KEY);

export function readIdentity(): Promise<Identity> {
  if (!native) return Promise.reject(new Error('desktop only'));
  return cached(KEY, async () => {
    const response = await rppCall('brain', 'settings.readback');
    if (!response.ok) throw new Error('settings.readback failed');
    const settings = (response.result as { settings?: Record<string, unknown> }).settings ?? {};
    return { user: text(settings.user_name), assistant: text(settings.assistant_name) };
  }, 60_000);
}
