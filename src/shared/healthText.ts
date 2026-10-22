// Product health checks come with machine names ("data_root_writable"); the person sees plain sentences.
// A check this table does not know keeps its own name and detail, so nothing a product reports is hidden.
interface Plain { label: string; ok: (detail: string) => string; bad: string; }

const after = (detail: string) => detail.includes(': ') ? detail.slice(detail.indexOf(': ') + 2) : detail;

const PLAIN: Record<string, Plain> = {
  'Protokol uyumluluğu': { label: 'Hub ile iletişim', ok: () => 'Hub ürünle sorunsuz konuşuyor.', bad: 'Hub ürünle konuşamıyor; ürünü ya da Hub\'ı güncelleyin.' },
  data_root_writable: { label: 'Veri klasörü', ok: () => 'Ayarlarınız ve çalışma verileriniz yazılabiliyor.', bad: 'Veri klasörüne yazılamıyor.' },
  config_loads: { label: 'Ayarlar', ok: () => 'Ayarlar okunuyor.', bad: 'Ayarlar okunamadı.' },
  vault_registered: { label: 'Not kasası', ok: detail => `Kasa bağlı: ${after(detail)}`, bad: 'Kayıtlı bir not kasası yok.' },
  git_available: { label: 'Git', ok: () => 'Git kurulu; görevler izole kopyada çalışabilir.', bad: 'Git bulunamadı; görevler izole kopya için git ister.' },
  worker_cli: { label: 'Çalışan araçlar', ok: detail => `Kullanılabilir: ${detail}`, bad: 'Görevleri yapacak komut satırı aracı (ör. claude, codex) bulunamadı.' },
  engine_key: { label: 'Motor anahtarı', ok: () => 'Hazır; ilk görevde kendiliğinden oluşur.', bad: 'Motor anahtarı okunamıyor.' },
};

export function plainCheck(name: string, ok: boolean, detail?: string): { label: string; text: string } {
  const known = PLAIN[name];
  if (!known) return { label: name, text: detail ?? '' };
  return { label: known.label, text: ok ? known.ok(detail ?? '') : `${known.bad}${detail ? ` (${detail})` : ''}` };
}
