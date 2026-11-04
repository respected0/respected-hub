import type { Run, RunUsage } from '../../shared/api.ts';

export interface TaskForm {
  projectRoot: string; task: string; worker: string; kind: string; ownership: string;
  forbidden: string; acceptance: string; customCommand: string; timeout: number;
}

const number = (value?: number | null) => typeof value === 'number' ? value.toLocaleString('tr-TR') : '—';

/** One readable line: weighted total first, then where it came from. Never fake a zero. */
export function usageText(usage?: RunUsage | null): string {
  if (!usage) return 'Token kullanımı bildirilmedi';
  const reasoning = typeof usage.reasoning_tokens === 'number' ? ` · düşünme ${number(usage.reasoning_tokens)}` : '';
  return `${number(usage.estimated_weighted_tokens)} ağırlıklı token · giriş ${number(usage.raw_input_tokens)}`
    + ` (önbellek ${number(usage.cached_input_tokens)}) · çıkış ${number(usage.output_tokens)}${reasoning}`;
}

export function taskTitle(run: Run): string {
  const task = run.request?.task?.trim();
  if (!task) return run.run_id;
  return task.split('\n')[0].slice(0, 100);
}

function argv(value: unknown): string[] {
  if (!Array.isArray(value) || !value.length || value.some(v => typeof v !== 'string' || !v || v.includes('\0'))) {
    throw new Error('Komut boş olmayan bir JSON string listesi olmalı.');
  }
  return value;
}

export function buildRequest(form: TaskForm) {
  const ownership = form.ownership.split('\n').map(s => s.trim()).filter(Boolean);
  const forbidden = form.forbidden.split('\n').map(s => s.trim()).filter(Boolean);
  if (!form.projectRoot.trim() || !form.task.trim() || !ownership.length) throw new Error('Proje, görev ve explicit dosya kapsamı gerekli.');
  if (!Number.isInteger(form.timeout) || form.timeout < 1 || form.timeout > 3600) throw new Error('Süre 1–3600 saniye olmalı.');
  const raw: unknown = JSON.parse(form.acceptance);
  if (!Array.isArray(raw)) throw new Error('Doğrulama komutları JSON listesi olmalı.');
  const acceptance = raw.map(argv);
  if (form.kind === 'write' && !acceptance.length) throw new Error('Yazma görevi için doğrulama komutu gerekli.');
  const command = form.worker === 'custom' ? argv(JSON.parse(form.customCommand)) : [];
  return { project_root: form.projectRoot.trim(), task: form.task.trim(), worker: form.worker, kind: form.kind,
    ownership, forbidden, acceptance, command, timeout_seconds: form.timeout };
}

export function terminal(status: string) {
  return ['completed', 'failed', 'timed_out', 'cancelled', 'interrupted', 'invalid'].includes(status);
}
