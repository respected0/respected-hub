import { useEffect, useState } from 'react';
import { knownIdentity, readIdentity } from '../identity.ts';
import type { ProductStatusKind } from '../../shared/productsModel.ts';
import { Emblem } from '../../shared/ui.tsx';

// The top of a product's Overview: a greeting, and a sentence about the product only when it needs attention (all well needs no words).
// The person's name is Brain's `user_name` setting (optional; no name, no name in the greeting).
export default function Hello({ app, product, kind, loading }: { app: string; product: string; kind: ProductStatusKind; loading: boolean }) {
  const [name, setName] = useState(() => knownIdentity()?.user ?? '');
  useEffect(() => {
    let active = true;
    readIdentity().then(identity => { if (active) setName(identity.user); }).catch(() => { /* optional personalization */ });
    return () => { active = false; };
  }, []);
  const sentence = loading || kind === 'ok' ? '' : kind === 'unreachable' ? `${product}'e ulaşılamıyor.` : `${product}'de dikkat gerektiren bir şey var.`;
  return <section className="hello" aria-label="Özet"><Emblem app={app} />
    <div><h2>{name ? `Merhaba ${name}` : 'Merhaba'}</h2>
      {sentence && <p><span className={`dot ${kind}`} aria-hidden="true" /> {sentence}</p>}</div></section>;
}
