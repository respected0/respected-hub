import { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { describeLifecycleError, lifecycleApply, lifecyclePlan } from '../../../shell/lifecycle.ts';
import type { LifecyclePlan } from '../../../shell/lifecycle.ts';
import { rppCall } from '../../../shared/productClient.ts';
import { EMPTY_SELECTION, summarize, toDesired } from '../brainClients.ts';
import type { Selection } from '../brainClients.ts';
import ClientPicker from './ClientPicker.tsx';
import IntegrationPreview from './IntegrationPreview.tsx';

const STEPS = ['Kimlik', 'Hafıza', 'Kaynaklar', 'İzinler', 'Onay'];
const PROVIDERS: [string, string][] = [['auto', 'Otomatik (kurulu olanı seç)'], ['claude', 'Claude'], ['codex', 'Codex'], ['gemini', 'Gemini'], ['antigravity', 'Antigravity'], ['cursor', 'Cursor']];

// First installation of SecondBrain through its own installer: Identity -> Memory -> Sources -> Permissions -> Finish.
// Only settings the engine really has are offered: names go into the vault template, capture mode and summariser are engine preferences.
export default function BrainInstall({ onDone }: { onDone: () => void }) {
  const [step, setStep] = useState(0);
  const [name, setName] = useState('');
  const [assistant, setAssistant] = useState('');
  const [parent, setParent] = useState('');
  const [vaultName, setVaultName] = useState('SecondBrain');
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
  const profile: Record<string, string> = {};
  if (name.trim()) profile.USER_NAME = name.trim();
  if (assistant.trim()) profile.COMPANION = assistant.trim();
  const desired = toDesired(selection);
  const params = { vault_path: vaultPath, profile, desired };
  const detected = plan?.integrations?.clients ? Object.fromEntries(plan.integrations.clients.map(item => [item.id, item.detected])) : null;

  // The product itself computes the preview (Discover / Inspect / Plan); refreshed whenever the choices change.
  useEffect(() => {
    if (step < 2 || !vaultPath) return;
    let active = true;
    setPlanning(true); setPlanError('');
    void lifecyclePlan('secondbrain', 'install', params).then(reply => {
      if (!active) return;
      setPlanning(false);
      if (reply.ok) setPlan(reply.result); else { setPlan(null); setPlanError(describeLifecycleError(reply.error)); }
    });
    return () => { active = false; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step >= 2, vaultPath, JSON.stringify(desired), name, assistant]);

  async function choose() {
    try { const picked = await open({ directory: true, multiple: false, title: 'Kasanın oluşturulacağı klasörü seç' }); if (typeof picked === 'string') setParent(picked); }
    catch (reason) { setError(String(reason)); }
  }

  async function install() {
    setBusy(true); setError('');
    const reply = await lifecycleApply('secondbrain', 'install', params);
    if (!reply.ok) { setBusy(false); setError(describeLifecycleError(reply.error)); return; }
    // Engine preferences exist once the product is installed; written through the product's own settings and read back.
    const settings: Record<string, unknown> = {};
    if (captureMode !== 'auto') settings.capture_mode = captureMode;
    if (provider !== 'auto') settings.summary_provider = provider;
    if (Object.keys(settings).length) {
      const applied = await rppCall('secondbrain', 'settings.apply', { settings }, crypto.randomUUID());
      if (!applied.ok) { setBusy(false); setError('SecondBrain kuruldu ama tercihler kaydedilemedi; Ayarlar bölümünden yeniden deneyin.'); return; }
    }
    setBusy(false);
    onDone();
  }

  const next = () => setStep(step + 1);
  const choice = (label: string, help: string, checked: boolean, change: (on: boolean) => void) => <label className="choice"><span><strong>{label}</strong><small>{help}</small></span>
    <input type="checkbox" role="switch" checked={checked} onChange={event => change(event.target.checked)} /></label>;

  return <div className="wizard panel-glass">
    <ol className="steps" aria-label="Kurulum adımları">{STEPS.map((label, index) => <li key={label} className={index === step ? 'current' : index < step ? 'past' : ''} aria-current={index === step ? 'step' : undefined}><span>{index + 1}</span>{label}</li>)}</ol>
    {step === 0 && <div className="step-body"><h2>SecondBrain'i kuralım</h2>
      <p>Hub, SecondBrain'in kendi kurucusunu doğrulanmış pakette çalıştırır: program, not kasan ve istediğin AI araç bağlantıları kurulur. Hiçbir adım senin onayın olmadan uygulanmaz.</p>
      <label>Adın <small>İsteğe bağlı. Kasa şablonundaki kişiselleştirme için kullanılır.</small><input value={name} maxLength={40} placeholder="ör. Ayşe" onChange={event => setName(event.target.value)} /></label>
      <label>Asistanının adı <small>İsteğe bağlı. Kasadaki asistan kimliği notlarına yazılır (ör. Jarvis); sonradan o notlardan değiştirilebilir.</small><input value={assistant} maxLength={40} placeholder="ör. Jarvis" onChange={event => setAssistant(event.target.value)} /></label></div>}
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
      <ClientPicker kind="hooks" selected={selection.sources} detected={detected} onChange={sources => setSelection({ ...selection, sources })} />
      {planning && <p className="muted" role="status">Plan hazırlanıyor…</p>}
      {planError && <p className="error" role="alert">{planError}</p>}</div>}
    {step === 3 && <div className="step-body"><h2>İzinler</h2>
      <p className="muted">Hangi araçlar hafızayı MCP sunucusu olarak kullanabilsin? Her izin istediğin zaman Hub'dan geri alınabilir.</p>
      <ClientPicker kind="mcp" selected={selection.mcp} detected={detected} onChange={mcp => setSelection({ ...selection, mcp })} />
      {choice('Sabah özeti zamanlanmış görevi', 'Windows Görev Zamanlayıcı’ya günlük bir görev ekler.', selection.schedule, schedule => setSelection({ ...selection, schedule }))}
      {choice('Masaüstü kısayolu', 'Masaüstüne not kasası klasörünü açan bir kısayol koyar.', selection.shortcut, shortcut => setSelection({ ...selection, shortcut }))}
      <p className="muted">Orchestrator'ın hafızaya erişimi (okuma, isteğe bağlı yazma) burada değil, kurulumdan sonra Orchestrator → Ayarlar'da ayrıca ve varsayılan kapalı verilir.</p>
      {planning && <p className="muted" role="status">Plan hazırlanıyor…</p>}
      {planError && <p className="error" role="alert">{planError}</p>}
      {plan?.integrations && !planning && <IntegrationPreview plan={plan.integrations} />}</div>}
    {step === 4 && <div className="step-body"><h2>Onay</h2>
      <ul className="review"><li><span>Kimlik</span><strong>{name.trim() || '—'}{assistant.trim() ? ` · asistan: ${assistant.trim()}` : ''}</strong></li>
        <li><span>Not kasası</span><strong className="mono">{vaultPath}</strong></li>
        <li><span>Hafıza yakalama</span><strong>{captureMode === 'auto' ? 'Otomatik' : 'Elle'} · özetleyici: {PROVIDERS.find(([id]) => id === provider)?.[1]}</strong></li>
        <li><span>Kaynaklar</span><strong>{summarize(selection.sources)}</strong></li>
        <li><span>MCP izni</span><strong>{summarize(selection.mcp)}</strong></li>
        <li><span>Diğer</span><strong>{[selection.schedule && 'Zamanlanmış görev', selection.shortcut && 'Kısayol'].filter(Boolean).join(', ') || 'Hiçbiri'}</strong></li>
        <li><span>Paket</span><strong>{plan?.to_version ? `v${plan.to_version} · imzasız yerel` : '—'}</strong></li></ul>
      {planError ? <p className="error" role="alert">{planError}</p> : <ol className="plan-steps">{plan?.plan.steps.map(item => <li key={item.id}>{item.summary}{!item.reversible && <em className="pill"> {item.irreversible_reason}</em>}</li>)}</ol>}
      <p className="muted">Kurulum SecondBrain'in kendi kurucusuyla yapılır ve 1-2 dakika sürebilir. Hata olursa ürün kendi değişikliklerini geri alır.</p></div>}
    {error && <p className="error" role="alert">{error}</p>}
    <div className="wizard-actions">
      <button type="button" disabled={step === 0 || busy} onClick={() => setStep(step - 1)}>Geri</button>
      {step < 4 ? <button type="button" className="cta" disabled={(step === 1 && !vaultPath) || (step >= 2 && (planning || !!planError || !plan))} onClick={next}>Devam</button>
        : <button type="button" className="cta" disabled={busy || !plan || !!planError} onClick={() => void install()}>{busy ? 'Kuruluyor…' : 'Onayla ve kur'}</button>}
    </div>
  </div>;
}
