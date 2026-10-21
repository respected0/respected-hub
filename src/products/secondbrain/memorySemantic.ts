// Optional local semantic search of SecondBrain. Off by default; FTS search works in every state.
import { invoke } from '@tauri-apps/api/core';
import { native } from '../../shared/api.ts';
import type { LifecycleError } from '../../shell/lifecycle.ts';

export type SemanticState = 'unsupported' | 'no-vault' | 'unknown' | 'off' | 'provider_missing' | 'model_missing' | 'building' | 'ready';
export interface SemanticStatus { supported: boolean; state: SemanticState; model?: string; provider?: string; download_mb?: number | null; notes?: number; vectors?: number; problem?: string; }
export type SemanticReply = { ok: true; result: SemanticStatus; error: null } | { ok: false; result: null; error: LifecycleError };

async function call(command: string): Promise<SemanticReply> {
  if (!native) return { ok: false, result: null, error: { code: 'UNAVAILABLE', message: 'Bu işlem yalnızca masaüstü uygulamasında kullanılabilir.', hint: 'Tauri masaüstü sürümünü çalıştırın.' } };
  try { return await invoke<SemanticReply>(command); }
  catch (reason) { return { ok: false, result: null, error: { code: 'INTERNAL', message: String(reason), hint: 'Uygulamayı yeniden başlatın.' } }; }
}

export const semanticStatus = () => call('memory_semantic_status');
export const semanticEnable = () => call('memory_semantic_enable');
export const semanticDisable = () => call('memory_semantic_disable');

export function describeSemantic(s: SemanticStatus): { title: string; detail: string; tone: 'ok' | 'warn' | 'unreachable' } {
  switch (s.state) {
    case 'unsupported': return { title: 'Desteklenmiyor', detail: 'Kurulu SecondBrain anlamsal aramayı desteklemiyor. SecondBrain’i güncelleyin.', tone: 'unreachable' };
    case 'no-vault': return { title: 'Kasa yok', detail: 'Önce SecondBrain kasasını kurun.', tone: 'unreachable' };
    case 'off': return { title: 'Kapalı', detail: 'Normal (tam metin) arama çalışıyor. Anlamsal arama, anlamca yakın notları da bulur; isteğe bağlıdır.', tone: 'unreachable' };
    case 'provider_missing': return { title: 'Yerel servis yok', detail: 'Seçili Ollama servisine ulaşılamıyor. Hub’ın kendi çalışma zamanına geçmek için kapatıp yeniden etkinleştirin. Normal arama sürer.', tone: 'warn' };
    case 'model_missing': return { title: 'Model indirilmemiş', detail: `Yerel model ve çalışma zamanı (yaklaşık ${s.download_mb ?? 460} MB) henüz indirilmedi. Normal arama sürer.`, tone: 'warn' };
    case 'building': return { title: 'Hazırlanıyor', detail: `${s.vectors ?? 0}/${s.notes ?? 0} not işlendi. Dizin yenilendikçe tamamlanır.`, tone: 'warn' };
    case 'ready': return { title: 'Hazır', detail: `Anlamsal arama etkin (${s.vectors ?? 0} not, model ${s.model ?? ''}). Notlarınız bilgisayarınızdan çıkmaz.`, tone: 'ok' };
    default: return { title: 'Belirlenemedi', detail: s.problem ?? 'Durum okunamadı.', tone: 'unreachable' };
  }
}
