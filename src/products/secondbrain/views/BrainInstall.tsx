import { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { describeLifecycleError, lifecycleApply, lifecyclePlan } from '../../../shell/lifecycle.ts';
import type { LifecyclePlan } from '../../../shell/lifecycle.ts';
import { EMPTY_SELECTION, summarize, toSettings } from '../brainClients.ts';
import type { Selection } from '../brainClients.ts';
import ClientPicker from './ClientPicker.tsx';

const STEPS = ['Kimlik', 'Hafıza', 'Kaynaklar', 'İzinler', 'Onay'];
const PROVIDERS: [string, string][] = [['auto', 'Otomatik (kurulu olanı seç)'], ['claude', 'Claude'], ['codex', 'Codex'], ['gemini', 'Gemini'], ['antigravity', 'Antigravity'], ['cursor', 'Cursor']];

// First installation of SecondBrain through its own installer: Identity -> Memory -> Sources -> Permissions -> Finish.
// Every answer is one of SecondBrain's own settings, handed to its installer exactly as a script would (`/VAULT=`, `/SET=key=value`).
export default function BrainInstall({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const [name, setName] = useState('');
  const [assistant, setAssistant] = useState('');
  const [bio, setBio] = useState('');
  const [parent, setParent] = useState('');
  const [vaultName, setVaultName] = useState('RespectedOS');
  const [captureMode, setCaptureMode] = useState<'auto' | 'manual'>('auto');
  const [provider, setProvider] = useState('auto');
  const [selection, setSelection] = useState<Selection>(EMPTY_SELECTION);
  const [plan, setPlan] = useState<LifecyclePlan | null>(null);
  const [planError, setPlanError] = useState('');
  const [planning, setPlanning] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  const separator = parent.includes('\\') ? '\\' : '/';
  const vaultPath = parent && vaultName.trim() ? `${parent.replace(/[\\/]+$/, '')}${separator}${vaultName.trim()}` : '';
  const settings = { user_name: name.trim(), assistant_name: assistant.trim(), user_bio: bio.trim(), capture_mode: captureMode, summary_provider: provider, ...toSettings(selection) };
  const params = { vault_path: vaultPath, settings };

  // The Hub's plan for running the installer (it downloads and verifies the installer first), at the last step.
  useEffect(() => {
    if (step < 4 || !vaultPath) return;
    let active = true;
    setPlanning(true); setPlanError('');
    void lifecyclePlan('secondbrain', 'install', params).then(reply => {
      if (!active) return;
      setPlanning(false);
      if (reply.ok) setPlan(reply.result); else { setPlan(null); setPlanError(describeLifecycleError(reply.error)); }
    });
    return () => { active = false; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step >= 4, vaultPath, JSON.stringify(settings)]);

  async function choose() {
    try { const picked = await open({ directory: true, multiple: false, title: 'Kasanın oluşturulacağı klasörü seç' }); if (typeof picked === 'string') setParent(picked); }
    catch (reason) { setError(String(reason)); }
  }

  async function install() {
    setBusy(true); setError('');
    const reply = await lifecycleApply('secondbrain', 'install', params);
    setBusy(false);
    if (!reply.ok) { setError(describeLifecycleError(reply.error)); return; }
    onDone();
  }

  const next = () => setStep(step + 1);
  const choice = (label: string, help: string, checked: boolean, change: (on: boolean) => void) => <label className="choice"><span><strong>{label}</strong><small>{help}</small></span>
    <input type="checkbox" role="switch" checked={checked} onChange={event => change(event.target.checked)} /></label>;

  return <div className="wizard panel-glass">
    <ol className="steps" aria-label="Kurulum adımları">{STEPS.map((label, index) => <li key={label} className={index === step ? 'current' : index < step ? 'past' : ''} aria-current={index === step ? 'step' : undefined}><span>{index + 1}</span>{label}</li>)}</ol>
    {step === 0 && <div className="step-body"><h2>SecondBrain'i kuralım</h2>
      <p>Hub, SecondBrain'in kendi imzalı kurucusunu çalıştırır ve buradaki cevapları ona iletir: program, not kasan ve istediğin AI araç bağlantıları kurulur. Aynı ayarları sonradan Hub'dan, kurucudan ya da "respectedbrain config" ile değiştirebilirsin.</p>
      <label>Adın <small>İsteğe bağlı. Kasa şablonundaki kişiselleştirme için kullanılır.</small><input value={name} maxLength={40} placeholder="ör. Ayşe" onChange={event => setName(event.target.value)} /></label>
      <label>Asistanının adı <small>İsteğe bağlı. Kasadaki asistan kimliği ve AI araçlarına verilen hafıza araçları bu adı kullanır (ör. Jarvis).</small><input value={assistant} maxLength={40} placeholder="ör. Jarvis" onChange={event => setAssistant(event.target.value)} /></label>
      <label>Kısa tanıtım <small>İsteğe bağlı. Kasa şablonunda seni tanıtan satıra yazılır.</small><input value={bio} maxLength={200} placeholder="ör. Yazılım geliştirici" onChange={event => setBio(event.target.value)} /></label></div>}
    {step === 1 && <div className="step-body vault-create"><h2>Hafıza</h2><p className="muted">Boş veya yeni bir klasör seç; var olan notların üzerine yazılmaz.</p>
      <label>Klasör<input value={parent} placeholder="ör. C:\Kullanıcı\Belgeler" onChange={event => setParent(event.target.value)} /></label>
      <div className="actions"><button type="button" onClick={() => void choose()}>Gözat…</button></div>
      <label>Kasa klasörünün adı<input value={vaultName} maxLength={60} onChange={event => setVaultName(event.target.value)} /></label>
      {vaultPath && <p className="muted">Oluşturulacak: <span className="mono">{vaultPath}</span></p>}
      <fieldset className="capture-mode"><legend>Hafıza yakalama</legend>
        <label className="choice"><span><strong>Otomatik</strong><small>Oturumlar bittiğinde AI araçlarının konuşmaları kasaya özetlenir (varsayılan).</small></span>
          <input type="radio" name="capture" checked={captureMode === 'auto'} onChange={() => setCaptureMode('auto')} /></label>
        <label className="choice"><span><strong>Elle (önce sen onayla)</strong><small>Hiçbir oturum kendiliğinden özetlenmez; Hub'dan “Şimdi yakala” dediğinde işlenir.</small></span>
          <input type="radio" name="capture" checked={captureMode === 'manual'} onChange={() => setCaptureMode('manual')} /></label></fieldset>
      <label>Özetleyici AI <small>Oturumları kasa notlarına özetleyen yerel AI aracı.</small><select value={provider} onChange={event => setProvider(event.target.value)}>{PROVIDERS.map(([id, label]) => <option key={id} value={id}>{label}</option>)}</select></label></div>}
    {step === 2 && <div className="step-body"><h2>Kaynaklar</h2>
      <p className="muted">Hangi AI araçlarının oturumları hafızaya yakalansın? Seçtiğin araçlara hafıza kuralları ve becerileri eklenir. Hepsi varsayılan olarak kapalıdır; başka araçların ve senin ayarların korunur.</p>
      <ClientPicker kind="hooks" selected={selection.sources} detected={null} onChange={sources => setSelection({ ...selection, sources })} />
      <p className="muted">Yalnızca bu bilgisayarda bulunan araçlara yazılır; kurulumdan sonra SecondBrain › Ayarlar'da tam olarak neyin değişeceğini görerek düzenleyebilirsin.</p></div>}
    {step === 3 && <div className="step-body"><h2>İzinler</h2>
      <p className="muted">Hangi araçlar hafızayı MCP sunucusu olarak kullanabilsin? Her izin istediğin zaman Hub'dan geri alınabilir.</p>
      <ClientPicker kind="mcp" selected={selection.mcp} detected={null} onChange={mcp => setSelection({ ...selection, mcp })} />
      {choice('Sabah özeti zamanlanmış görevi', 'Windows Görev Zamanlayıcı’ya günlük bir görev ekler.', selection.schedule, schedule => setSelection({ ...selection, schedule }))}
      {choice('Masaüstü kısayolu', 'Masaüstüne not kasası klasörünü açan bir kısayol koyar.', selection.shortcut, shortcut => setSelection({ ...selection, shortcut }))}
      <p className="muted">Orchestrator'ın hafızaya erişimi (okuma, isteğe bağlı yazma) burada değil, kurulumdan sonra Orchestrator › Ayarlar'da ayrıca ve varsayılan kapalı verilir.</p></div>}
    {step === 4 && <div className="step-body"><h2>Onay</h2>
      <ul className="review"><li><span>Kimlik</span><strong>{name.trim() || '—'}{assistant.trim() ? ` · asistan: ${assistant.trim()}` : ''}</strong></li>
        <li><span>Not kasası</span><strong className="mono">{vaultPath}</strong></li>
        <li><span>Hafıza yakalama</span><strong>{captureMode === 'auto' ? 'Otomatik' : 'Elle'} · özetleyici: {PROVIDERS.find(([id]) => id === provider)?.[1]}</strong></li>
        <li><span>Kaynaklar</span><strong>{summarize(selection.sources)}</strong></li>
        <li><span>MCP izni</span><strong>{summarize(selection.mcp)}</strong></li>
        <li><span>Diğer</span><strong>{[selection.schedule && 'Zamanlanmış görev', selection.shortcut && 'Kısayol'].filter(Boolean).join(', ') || 'Hiçbiri'}</strong></li>
        <li><span>Sürüm</span><strong>{plan?.to_version ? `v${plan.to_version}` : '—'}</strong></li></ul>
      {planning && <p className="muted" role="status">Kurucu indiriliyor ve doğrulanıyor…</p>}
      {planError ? <p className="error" role="alert">{planError}</p> : <ol className="plan-steps">{plan?.plan.steps.map(item => <li key={item.id}>{item.summary}{!item.reversible && <em className="pill"> {item.irreversible_reason}</em>}</li>)}</ol>}
      <p className="muted">Kurulum SecondBrain'in kendi kurucusuyla yapılır ve 1-2 dakika sürebilir. Hata olursa ürün kendi değişikliklerini geri alır.</p></div>}
    {error && <p className="error" role="alert">{error}</p>}
    <div className="wizard-actions">
      <button type="button" disabled={step === 0 || busy} onClick={() => setStep(step - 1)}>Geri</button>
      {step < 4 ? <button type="button" className="cta" disabled={step === 1 && !vaultPath} onClick={next}>Devam</button>
        : <button type="button" className="cta" disabled={busy || !plan || !!planError} onClick={() => void install()}>{busy ? 'Kuruluyor…' : 'Onayla ve kur'}</button>}
    </div>
  </div>;
}
