// Typed client for Hub campaign tracking plus a pure, testable status summary.
// Real data only: outside Tauri the calls reject / the view shows an honest notice.
import { invoke } from '@tauri-apps/api/core';
import { engine, native } from '../../shared/api.ts';

export interface CampaignStatusResponse {
  campaign_id?: string;
  status?: string;
  result?: unknown;
  events?: unknown;
  supervisor_alive?: boolean;
}

export interface CampaignStart { campaign_id: string; status: string; pid: number }

export type CampaignPhase = 'running' | 'finished' | 'needs_review' | 'interrupted' | 'blocked';
export interface CampaignLane { name: string; statusText: string; detail: string; tokens: number | null }
export interface CampaignTimelineItem { at: string; text: string }
export interface CampaignChain { index: number; parent: string; outcome: string }
export interface CampaignSummary {
  headline: string;
  phase: CampaignPhase;
  lanes: CampaignLane[];
  timeline: CampaignTimelineItem[];
  chain?: CampaignChain;
  finalText?: string;
}

/** Newest-first campaign directory names from the read-only native command. */
export async function listCampaigns(): Promise<string[]> {
  if (!native) throw new Error('Kampanya listesi Tauri masaüstü uygulamasında kullanılabilir.');
  return invoke<string[]>('campaign_list');
}

export function campaignStatus(id: string): Promise<CampaignStatusResponse> {
  return engine<CampaignStatusResponse>('campaign-status', id);
}

/** Parses the spec locally, then asks the engine to start exactly one campaign. */
export async function startCampaign(specJson: string): Promise<CampaignStart> {
  let spec: unknown;
  try { spec = JSON.parse(specJson); }
  catch { throw new Error('Kampanya spec geçerli bir JSON olmalı.'); }
  if (!isRecord(spec)) throw new Error('Kampanya spec bir JSON nesnesi olmalı.');
  return engine<CampaignStart>('campaign-start', undefined, spec);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function phaseOf(status: string, alive: boolean): CampaignPhase {
  switch (status) {
    case 'running': case 'started': case 'queued': return 'running';
    case 'LANES_APPROVED': case 'COMPLETE': case 'completed': return 'finished';
    case 'LANES_NEED_REVIEW': return 'needs_review';
    case 'BLOCKED': case 'FAILED': case 'failed': return 'blocked';
    case 'interrupted': case 'cancelled': case 'timed_out': return 'interrupted';
    default: return alive ? 'running' : 'interrupted';
  }
}

function headlineOf(phase: CampaignPhase, alive: boolean, masterFinished: boolean): string {
  switch (phase) {
    case 'running': return 'Kampanya çalışıyor; şeritler sürüyor.';
    case 'finished':
      if (masterFinished) return 'Tüm şeritler onaylandı; son inceleme tamamlandı.';
      return alive ? 'Tüm şeritler onaylandı; son inceleme sürüyor.' : 'Tüm şeritler onaylandı; son inceleme tamamlanmadı.';
    case 'needs_review': return 'Şeritler bitti ama bazıları elle inceleme bekliyor.';
    case 'blocked': return 'Kampanya engellendi veya başarısız oldu.';
    case 'interrupted': return 'Kampanya yarıda kesildi; süpervizör çalışmıyor.';
  }
}

function laneStatusText(status: unknown): string {
  const text = typeof status === 'string' ? status : '';
  switch (text) {
    case 'MASTER_APPROVED': return 'Onaylandı';
    case 'SKIPPED': return 'Atlandı';
    case 'FAILED': return 'Başarısız';
    case 'BLOCKED': return 'Engellendi';
    case 'HUMAN_APPROVAL_REQUIRED': return 'Elle onay gerekli';
    case '': return 'Bilinmiyor';
    default: return text;
  }
}

function reasonText(reason: string): string {
  if (reason.startsWith('dependency ')) return 'Bağımlı şerit onaylanmadı';
  switch (reason) {
    case 'lane_error': return 'Şerit hatası';
    case 'maf_no_output': return 'Şeritten yanıt alınamadı';
    default: return reason;
  }
}

function laneDetail(lane: Record<string, unknown>): string {
  const parts: string[] = [];
  if (typeof lane.reason === 'string' && lane.reason) parts.push(reasonText(lane.reason));
  const changed = Array.isArray(lane.changed_files) ? lane.changed_files.length : 0;
  if (changed > 0) parts.push(`${changed} dosya değişti`);
  else if (typeof lane.attempts === 'number' && lane.attempts > 1) parts.push(`${lane.attempts} deneme`);
  return parts.join(' · ');
}

function tokensOf(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (isRecord(value)) {
    let sum = 0; let any = false;
    for (const entry of Object.values(value)) if (typeof entry === 'number' && Number.isFinite(entry)) { sum += entry; any = true; }
    return any ? sum : null;
  }
  return null;
}

function campaignOutcomeText(status: unknown): string {
  const text = typeof status === 'string' ? status : '';
  switch (text) {
    case 'LANES_APPROVED': return 'onaylandı';
    case 'LANES_NEED_REVIEW': return 'inceleme bekliyor';
    default: return text || 'bilinmiyor';
  }
}

function masterOutcomeText(status: unknown): string {
  const text = typeof status === 'string' ? status : '';
  switch (text) {
    case 'COMPLETE': return 'tamamlandı';
    case 'FAILED': return 'başarısız';
    case 'BLOCKED': return 'engellendi';
    default: return text || 'bilinmiyor';
  }
}

function eventText(event: Record<string, unknown>): string {
  const kind = typeof event.event === 'string' ? event.event : '';
  const lane = typeof event.lane === 'string' && event.lane ? event.lane : 'Şerit';
  switch (kind) {
    case 'lane.started': return `${lane} başladı.`;
    case 'lane.completed': return `${lane} tamamlandı: ${laneStatusText(event.status)}.`;
    case 'campaign.completed': return `Tüm şeritler bitti: ${campaignOutcomeText(event.status)}.`;
    case 'master.finished': return `Son inceleme bitti: ${masterOutcomeText(event.status)}.`;
    default: return kind || 'Olay';
  }
}

/** Turns a raw `campaign-status` reply into plain Turkish. Tolerant of missing fields. */
export function summarizeCampaign(status: unknown): CampaignSummary {
  const root = isRecord(status) ? status : {};
  const result = isRecord(root.result) ? root.result : {};
  const events = (Array.isArray(root.events) ? root.events : []).filter(isRecord);
  const rawStatus = typeof root.status === 'string' ? root.status : typeof result.status === 'string' ? result.status : '';
  const alive = root.supervisor_alive === true;
  const phase = phaseOf(rawStatus, alive);
  const masterFinished = events.some(event => event.event === 'master.finished');

  const lanes: CampaignLane[] = (Array.isArray(result.lanes) ? result.lanes : []).filter(isRecord).map(lane => ({
    name: typeof lane.name === 'string' && lane.name ? lane.name : '(isimsiz şerit)',
    statusText: laneStatusText(lane.status),
    detail: laneDetail(lane),
    tokens: tokensOf(lane.token_usage),
  }));

  const timeline: CampaignTimelineItem[] = events.map(event => ({
    at: typeof event.at === 'string' ? event.at : '',
    text: eventText(event),
  }));

  const summary: CampaignSummary = { headline: headlineOf(phase, alive, masterFinished), phase, lanes, timeline };

  const chainRaw = isRecord(result.chain) ? result.chain : isRecord(root.chain) ? root.chain : null;
  if (chainRaw && typeof chainRaw.index === 'number' && typeof chainRaw.parent === 'string' && typeof chainRaw.outcome === 'string') {
    summary.chain = { index: chainRaw.index, parent: chainRaw.parent, outcome: chainRaw.outcome };
  }

  const finalText = typeof result.final_text === 'string' ? result.final_text : typeof root.final_text === 'string' ? root.final_text : '';
  if (finalText) summary.finalText = finalText;

  return summary;
}