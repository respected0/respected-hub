import { useEffect, useState } from 'react';
import { native } from '../../../shared/api.ts';
import { cached, peek } from '../../../shared/rppCache.ts';
import { readNote, type NoteContent } from '../memoryWorkspace.ts';
import { excerpt } from '../noteExcerpt.ts';

const today = () => new Date().toLocaleDateString('sv-SE');   // YYYY-MM-DD in local time, as the daily notes are named
const CARDS: { key: string; title: string; path: string; empty: string }[] = [
  { key: 'session', title: 'Son oturum', path: '🔮 850-Companion/Last-Session.md', empty: 'Henüz bir oturum özeti yok.' },
  { key: 'daily', title: 'Bugünün günlüğü', path: `daily/${today()}.md`, empty: 'Bugün için günlük kaydı yok.' },
  { key: 'briefing', title: 'Bugünün brifingi', path: `🎯 100-Command-Center/Briefings/${today()}.md`, empty: 'Bugün için brifing yok. Sabah özeti zamanlandıysa sabah oluşur.' },
];

// What Brain remembered lately, read from the person's own notes (the same notes the Memory page opens).
export default function BrainOverview({ openMemory, flat = false }: { openMemory: () => void; flat?: boolean }) {
  // the last answer shows at once; reading again happens behind it
  const [notes, setNotes] = useState<Record<string, NoteContent | null> | null>(() => {
    const known = CARDS.map(card => peek<NoteContent | null>(`note:${card.path}`));
    return known.every(note => note !== undefined) ? Object.fromEntries(CARDS.map((card, index) => [card.key, known[index] ?? null])) : null;
  });
  useEffect(() => {
    if (!native) return;
    let active = true;
    void Promise.all(CARDS.map(async card => [card.key, await cached(`note:${card.path}`, async () => { const reply = await readNote(card.path); return reply.ok ? reply.result : null; }, 30_000)] as const))
      .then(entries => { if (active) setNotes(Object.fromEntries(entries)); });
    return () => { active = false; };
  }, []);
  if (!native) return null;
  return <section className={`note-cards ${flat ? 'flat' : ''}`} aria-label="Hafızadan son kayıtlar">{CARDS.map(card => { const note = notes?.[card.key];
    return <article key={card.key} className="note-card"><h3>{card.title}</h3>
      {!notes ? <p className="muted">Okunuyor…</p>
        : note ? <><ul>{excerpt(note.content).map(line => <li key={line}>{line}</li>)}</ul><small className="muted">Güncellendi: {new Date(note.modified).toLocaleString('tr-TR')}</small></>
        : <p className="muted">{card.empty}</p>}
      <button className="link-btn" onClick={openMemory}>Hafızada aç</button></article>; })}</section>;
}
