import { useEffect, useState, type ReactNode } from 'react';
import { knownIdentity, readIdentity } from '../identity.ts';
import type { ProductStatusKind } from '../../shared/productsModel.ts';
import { Emblem } from '../../shared/ui.tsx';

// The top of a product's Overview: a greeting and the product's state in one plain sentence.
// The person's name is Brain's `user_name` setting (optional; no name, no name in the greeting).
export default function Hello({ app, product, kind, loading, version, children }: { app: string; product: string; kind: ProductStatusKind; loading: boolean; version?: string; children?: ReactNode }) {
  const [name, setName] = useState(() => knownIdentity()?.user ?? '');
  useEffect(() => {
    let active = true;
    readIdentity().then(identity => { if (active) setName(identity.user); }).catch(() => { /* optional personalization */ });
    return () => { active = false; };
  }, []);
  const sentence = loading ? `${product} kontrol ediliyor…` : kind === 'ok' ? `${product} çalışıyor.` : kind === 'unreachable' ? `${product}'e ulaşılamıyor.` : `${product}'de dikkat gerektiren bir şey var.`;
  return <section className="hello" aria-label="Özet"><Emblem app={app} />
    <div><h2>{name ? `Merhaba ${name}` : 'Merhaba'}</h2>
      <p><span className={`dot ${loading ? '' : kind}`} aria-hidden="true" /> {sentence}{version ? <span className="ver"> · v{version}</span> : null}</p>
      {children}</div></section>;
}
