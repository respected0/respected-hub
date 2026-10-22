import { useState } from 'react';
import { KNOWN_NOTES, captureNote, describeSearchMode, describeWorkspaceError, readNote, runBriefing, searchNotes } from '../memoryWorkspace.ts';
import type { CaptureResult, NoteContent, SearchResult } from '../memoryWorkspace.ts';

// Local date as YYYY-MM-DD (toISOString would give the UTC date).
function localToday(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

const stripMarks = (s: string) => s.replace(/>>>|<<</g, '');

// Brain memory page: search, read, quick capture and on-demand briefing.
export default function MemoryWorkspace() {
  const [query, setQuery] = useState('');
  const [searching, setSearching] = useState(false);
  const [found, setFound] = useState<SearchResult | null>(null);
  const [searchError, setSearchError] = useState('');

  const [note, setNote] = useState<NoteContent | null>(null);
  const [reading, setReading] = useState(false);
  const [readError, setReadError] = useState('');

  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState<CaptureResult | null>(null);
  const [saveError, setSaveError] = useState('');

  const [briefing, setBriefing] = useState(false);
  const [briefMsg, setBriefMsg] = useState('');
  const [briefError, setBriefError] = useState('');

  async function search() {
    const q = query.trim();
    if (!q) return;
    setSearching(true); setSearchError('');
    const reply = await searchNotes(q);
    if (reply.ok) setFound(reply.result); else { setFound(null); setSearchError(describeWorkspaceError(reply.error)); }
    setSearching(false);
  }

  async function open(path: string) {
    setReading(true); setReadError('');
    const reply = await readNote(path);
    if (reply.ok) setNote(reply.result); else { setNote(null); setReadError(describeWorkspaceError(reply.error)); }
    setReading(false);
  }

  async function save() {
    setSaving(true); setSaveError(''); setSaved(null);
    const reply = await captureNote(content, title);
    if (reply.ok) { setSaved(reply.result); setTitle(''); setContent(''); } else setSaveError(describeWorkspaceError(reply.error));
    setSaving(false);
  }

  async function brief() {
    setBriefing(true); setBriefMsg(''); setBriefError('');
    const reply = await runBriefing();
    if (reply.ok) setBriefMsg(reply.result.status === 0 ? 'Brifing tamamlandı.' : `Brifing bitti ancak çıkış kodu ${reply.result.status}.`); else setBriefError(describeWorkspaceError(reply.error));
    setBriefing(false);
  }

  const today = localToday();
  return <>
    <section className="panel-glass" aria-label="Not ara">
      <div className="section-head"><h2>Not ara</h2></div>
      <form className="actions" onSubmit={e => { e.preventDefault(); void search(); }}>
        <input type="text" value={query} onChange={e => setQuery(e.target.value)} placeholder="Notlarda ara…" aria-label="Arama sorgusu" />
        <button type="submit" className="cta" disabled={searching || !query.trim()}>{searching ? 'Aranıyor…' : 'Ara'}</button>
      </form>
      {searchError && <p className="error" role="alert">{searchError}</p>}
      {found && <>
        <p className="muted" role="status">{describeSearchMode(found.search)}</p>
        {found.results.length === 0 ? <p role="status">Eşleşen not yok</p> : <ul>
          {found.results.map(hit => <li key={hit.path}>
            <strong>{hit.title}</strong> <span className="mono">{hit.path}</span>
            <p className="muted">{stripMarks(hit.snippet)}</p>
            <button type="button" disabled={reading} onClick={() => void open(hit.path)}>Oku</button>
          </li>)}
        </ul>}
      </>}
    </section>

    <section className="panel-glass" aria-label="Not oku">
      <div className="section-head"><h2>Not oku</h2></div>
      <div className="actions">
        {KNOWN_NOTES.map(k => <button key={k.label} type="button" disabled={reading} onClick={() => void open(k.path(today))}>{k.label}</button>)}
      </div>
      {readError && <p className="error" role="alert">{readError}</p>}
      {note && <div>
        <h3>{note.title}</h3>
        <p className="muted"><span className="mono">{note.path}</span> · Değiştirilme: {new Date(note.modified).toLocaleString()}</p>
        {note.truncated && <p role="status">Not büyük olduğu için içeriğin yalnızca ilk kısmı gösteriliyor.</p>}
        <pre style={{ maxHeight: 360, overflow: 'auto' }}>{note.content}</pre>
        <div className="actions"><button type="button" onClick={() => setNote(null)}>Kapat</button></div>
      </div>}
    </section>

    <section className="panel-glass" aria-label="Hızlı not yakala">
      <div className="section-head"><h2>Hızlı not yakala</h2></div>
      <input type="text" value={title} maxLength={120} onChange={e => setTitle(e.target.value)} placeholder="Başlık (isteğe bağlı)" aria-label="Not başlığı" />
      <textarea value={content} maxLength={20000} rows={6} onChange={e => setContent(e.target.value)} placeholder="Notunuz…" aria-label="Not içeriği" />
      {saved && <p role="status">Kaydedildi: <span className="mono">{saved.path}</span> — {saved.indexed ? 'aramada bulunabilir.' : 'henüz aramada bulunamaz (dizin yenilenince görünür).'}</p>}
      {saveError && <p className="error" role="alert">{saveError}</p>}
      <div className="actions"><button type="button" className="cta" disabled={saving || !content.trim()} onClick={() => void save()}>{saving ? 'Kaydediliyor…' : 'Kaydet'}</button></div>
    </section>

    <section className="panel-glass" aria-label="Brifing">
      <div className="section-head"><h2>Brifing</h2></div>
      <p className="muted">Bir model günün notlarını özetler; zamanlanmış görev bunu her gün kendiliğinden yapar. Dakikalar sürebilir.</p>
      {briefMsg && <p role="status">{briefMsg}</p>}
      {briefError && <p className="error" role="alert">{briefError}</p>}
      <div className="actions"><button type="button" className="cta" disabled={briefing} onClick={() => void brief()}>{briefing ? 'Çalışıyor… (dakikalar sürebilir)' : 'Brifingi şimdi çalıştır'}</button></div>
    </section>
  </>;
}
