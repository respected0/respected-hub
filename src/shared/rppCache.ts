// Every product call starts a program (240 to 570 ms measured), so the Hub never asks the same thing twice at once and never makes a page wait for
// something it already knows: `peek` gives the last answer at once, `cached` refreshes it in the background (and shares one running call).
const store = new Map<string, { at: number; value: unknown }>();
const running = new Map<string, Promise<unknown>>();

export function peek<T>(key: string): T | undefined {
  return store.get(key)?.value as T | undefined;
}

/** Runs `fetcher` unless the same key is already running or was answered less than `maxAgeMs` ago; failed calls are not stored. */
export function cached<T>(key: string, fetcher: () => Promise<T>, maxAgeMs = 0): Promise<T> {
  const hit = store.get(key);
  if (hit && Date.now() - hit.at < maxAgeMs) return Promise.resolve(hit.value as T);
  const live = running.get(key);
  if (live) return live as Promise<T>;
  const call = fetcher().then(value => { store.set(key, { at: Date.now(), value }); return value; }).finally(() => running.delete(key));
  running.set(key, call);
  return call;
}

/** `cached` for replies shaped `{ ok }`: a failed reply is handed back to every waiting caller but never stored. */
export async function cachedReply<T extends { ok: boolean }>(key: string, fetcher: () => Promise<T>, maxAgeMs = 0): Promise<T> {
  try { return await cached(key, async () => { const reply = await fetcher(); if (!reply.ok) throw reply; return reply; }, maxAgeMs); }
  catch (failure) { if (failure && typeof failure === 'object' && 'ok' in failure) return failure as T; throw failure; }
}

export function forget(prefix: string): void {
  for (const key of [...store.keys()]) if (key.startsWith(prefix)) store.delete(key);
}
