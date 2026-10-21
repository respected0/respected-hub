// Pure, testable model for the review/evidence panel. No React, no Tauri imports.
// Turns a raw engine `inspect` payload and/or a `campaign-status` reply into plain Turkish evidence.
// The status/reason wordings intentionally mirror src/campaigns.ts so the panel stays jargon-free
// without importing the Tauri-bound client into a pure model.

export type Verdict = 'approved' | 'needs_changes' | 'failed' | 'human_approval' | 'unknown';
export interface EvidenceItem { label: string; text: string; }
export interface ReviewItem { stage: string; profile: string; decision: string; reason: string; }
export interface Evidence {
  headline: string;
  verdict: Verdict;
  evidence: EvidenceItem[];
  reviews: ReviewItem[];
  hasPatch: boolean;
  nextStepText: string;
}

const APPROVED = ['MASTER_APPROVED', 'approved', 'completed', 'COMPLETE', 'success'];
const HUMAN = ['HUMAN_APPROVAL_REQUIRED', 'needs_review', 'NEEDS_REVIEW', 'LANES_NEED_REVIEW', 'pending_review'];
const FAILED = ['FAILED', 'failed', 'BLOCKED', 'blocked', 'error', 'timed_out', 'cancelled', 'interrupted', 'invalid'];
const CHANGES = ['request_changes', 'changes_requested', 'needs_changes', 'rejected'];
const RUN_PLACEHOLDER = '<run_id>';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function text(value: unknown): string {
  return typeof value === 'string' ? value.trim() : '';
}

function firstText(...values: unknown[]): string {
  for (const value of values) { const found = text(value); if (found) return found; }
  return '';
}

function numberText(value: unknown): string {
  return typeof value === 'number' && Number.isFinite(value) ? String(value) : '';
}

function filesOf(record: Record<string, unknown>): string[] {
  return Array.isArray(record.changed_files)
    ? record.changed_files.filter((file): file is string => typeof file === 'string' && file !== '')
    : [];
}

function tokenTotal(...values: unknown[]): string {
  let sum = 0; let any = false;
  for (const value of values) {
    if (typeof value === 'number' && Number.isFinite(value)) { sum += value; any = true; continue; }
    if (isRecord(value)) {
      for (const entry of Object.values(value)) {
        if (typeof entry === 'number' && Number.isFinite(entry)) { sum += entry; any = true; }
      }
    }
  }
  return any ? String(sum) : '';
}

function statusPlain(status: string): string {
  switch (status) {
    case 'MASTER_APPROVED': return 'Onaylandı';
    case 'SKIPPED': return 'Atlandı';
    case 'FAILED': return 'Başarısız';
    case 'BLOCKED': return 'Engellendi';
    case 'HUMAN_APPROVAL_REQUIRED': return 'Elle onay gerekli';
    case '': return 'Bilinmiyor';
    default: return status;
  }
}

function reasonPlain(reason: string): string {
  if (reason.startsWith('dependency ')) return 'Bağımlı şerit onaylanmadı';
  switch (reason) {
    case 'lane_error': return 'Şerit hatası';
    case 'maf_no_output': return 'Şeritten yanıt alınamadı';
    default: return reason;
  }
}

function collectReviews(lanes: Record<string, unknown>[], root: Record<string, unknown>, result: Record<string, unknown>): ReviewItem[] {
  const nested = lanes.flatMap(lane => (Array.isArray(lane.reviews) ? lane.reviews : []));
  const sources = [...nested, ...(Array.isArray(root.reviews) ? root.reviews : []), ...(Array.isArray(result.reviews) ? result.reviews : [])];
  return sources.filter(isRecord).map(review => ({
    stage: text(review.stage) || '(aşama yok)',
    profile: text(review.profile) || '(profil yok)',
    decision: text(review.decision) || '(karar yok)',
    reason: text(review.reason) || text(review.detail),
  }));
}

function verdictOf(statuses: string[], reviews: ReviewItem[], errored: boolean): Verdict {
  if (errored || statuses.some(status => FAILED.includes(status))) return 'failed';
  if (statuses.some(status => HUMAN.includes(status))) return 'human_approval';
  if (reviews.some(review => CHANGES.includes(review.decision))) return 'needs_changes';
  if (statuses.some(status => APPROVED.includes(status))) return 'approved';
  return 'unknown';
}

function headlineOf(verdict: Verdict): string {
  switch (verdict) {
    case 'approved': return 'Bu çalışma onaylandı; kanıtlar aşağıda.';
    case 'human_approval': return 'Bu iş elle onay bekliyor; Hub onay vermez.';
    case 'needs_changes': return 'İnceleme değişiklik istedi.';
    case 'failed': return 'Bu çalışma başarısız oldu veya engellendi.';
    case 'unknown': return 'Bu çalışma için net bir sonuç yok; kanıtlara bakın.';
  }
}

function nextStepOf(verdict: Verdict, hasPatch: boolean, runId: string, statuses: string[]): string {
  const id = runId || RUN_PLACEHOLDER;
  if (verdict === 'human_approval') {
    return `Bu iş elle onay bekliyor. Onay interaktif bir terminal gerektirir; Hub onay veremez ve bu komutu çalıştırmaz. Kendi terminalinizde şunu çalıştırın:\npython -m respected_orchestrator maf-approve ${id}`;
  }
  if (statuses.includes('MASTER_APPROVED') || (verdict === 'approved' && hasPatch)) {
    return `Bu çalışma onaylandı. Hub güvenlik nedeniyle değişiklikleri kendisi uygulamaz ve bu komutu çalıştırmaz; kendi terminalinizde şunu çalıştırın:\npython -m respected_orchestrator apply ${id}`;
  }
  if (verdict === 'approved') return 'Çalışma onaylandı; uygulanacak bir değişiklik görünmüyor. Ek işlem gerekmez.';
  if (verdict === 'needs_changes') return 'İnceleme değişiklik istedi; düzeltmeden sonra çalışmayı yeniden inceleyin.';
  if (verdict === 'failed') return 'Çalışma başarısız olduğu için patch uygulanmaz. Ayrıntı için kanıtlara ve loglara bakın.';
  return 'Motor bu çalışma için onay veya patch bilgisi vermedi; kanıtlar sınırlı.';
}

interface EvidenceInput {
  run: Record<string, unknown>;
  request: Record<string, unknown>;
  lanes: Record<string, unknown>[];
  root: Record<string, unknown>;
  result: Record<string, unknown>;
  changedFiles: string[];
  patchPreview: string;
}

function evidenceItems(input: EvidenceInput): EvidenceItem[] {
  const { run, request, lanes, root, result, changedFiles, patchPreview } = input;
  const items: EvidenceItem[] = [];
  const push = (label: string, value: string) => { if (value) items.push({ label, text: value }); };

  push('Görev', text(request.task));
  push('İşçi', text(request.worker));
  push('Proje', text(request.project_root));
  push('Durum', firstText(run.status, root.status, result.status));
  push('Aşama', text(run.phase));
  push('Neden', reasonPlain(firstText(run.reason)));
  push('Hata', text(run.error));
  if (changedFiles.length) {
    const shown = changedFiles.slice(0, 20).join(', ');
    push('Değişen dosyalar', `${changedFiles.length} dosya: ${shown}${changedFiles.length > 20 ? ' …' : ''}`);
  }
  if (patchPreview) push('Patch önizleme', patchPreview);

  for (const lane of lanes) {
    const name = text(lane.name) || '(isimsiz şerit)';
    const parts = [statusPlain(firstText(lane.status))];
    const reason = reasonPlain(firstText(lane.reason));
    if (reason) parts.push(reason);
    const files = filesOf(lane);
    if (files.length) parts.push(`${files.length} dosya değişti`);
    const tokens = tokenTotal(lane.token_usage);
    if (tokens) parts.push(`${tokens} token`);
    push(`Şerit: ${name}`, parts.join(' · '));
  }

  push('Toplam token', tokenTotal(...lanes.map(lane => lane.token_usage)));

  const chain = isRecord(result.chain) ? result.chain : isRecord(root.chain) ? root.chain : null;
  if (chain) push('Devam zinciri', `${firstText(chain.parent) || '(ana kampanya)'} (sıra ${numberText(chain.index) || '—'}) · ${firstText(chain.outcome) || 'bilinmiyor'}`);

  push('Son inceleme notu', firstText(result.final_text, root.final_text));

  const logs = isRecord(run.logs) ? run.logs : {};
  for (const [name, log] of Object.entries(logs)) {
    if (typeof log === 'string' && log.trim()) push(`Log: ${name}`, log.length > 2000 ? `${log.slice(0, 2000)}…` : log);
  }

  return items;
}

/**
 * Builds the review/evidence summary from an engine `inspect` payload (a run) and/or a
 * `campaign-status` reply (the campaign lane summary). Tolerant of missing or malformed fields.
 */
export function evidenceFor(inspect: unknown, campaign: unknown, fallbackRunId = ''): Evidence {
  const run = isRecord(inspect) ? inspect : {};
  const root = isRecord(campaign) ? campaign : {};
  const result = isRecord(root.result) ? root.result : {};
  const lanes = (Array.isArray(result.lanes) ? result.lanes : []).filter(isRecord);
  const request = isRecord(run.request) ? run.request : {};

  const statuses = [
    text(run.status), text(run.phase),
    firstText(root.status, result.status),
    ...lanes.map(lane => text(lane.status)),
  ].filter(Boolean);

  const reviews = collectReviews(lanes, root, result);
  const changedFiles = [...filesOf(run), ...lanes.flatMap(filesOf)];
  const patchPreview = firstText(run.patch_preview, ...lanes.map(lane => text(lane.patch_preview)));
  const hasPatch = Boolean(patchPreview) || changedFiles.length > 0;
  const errored = Boolean(firstText(run.error) || lanes.map(lane => text(lane.error)).find(Boolean));
  const verdict = verdictOf(statuses, reviews, errored);
  const runId = firstText(run.run_id, ...lanes.map(lane => text(lane.run_id)), fallbackRunId);

  return {
    headline: headlineOf(verdict),
    verdict,
    evidence: evidenceItems({ run, request, lanes, root, result, changedFiles, patchPreview }),
    reviews,
    hasPatch,
    nextStepText: nextStepOf(verdict, hasPatch, runId, statuses),
  };
}