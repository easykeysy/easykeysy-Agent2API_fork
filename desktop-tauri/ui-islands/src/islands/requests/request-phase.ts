import { t } from '../../i18n'
import type { RequestEntry } from './request-cells'

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, ch => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
  }[ch] ?? ch))
}

/* ─── （ ui/request-phase.js， window）──────────
 *
 * 1 need in **** and must 1 ： （file） and 
 * （islands/request-detail.tsx StatusCell， window.wbRequestPhase badgeHtml /
 * elapsedLineHtml and ）。 / / when 1 in this ， old ——
 * not can and after ： that will that （「 in 」）。
 *
 * after in `phase` / `phaseStartedAt`（ core::upstream::usage::LogPhase）：
 * phase empty = not in （、 old ）， use 「 in 」。
 * （） in when t()： head i18n ， to 。
 */
const PHASES: Record<string, { label: string; cls: string }> = {
  connecting: { label: t('连接中'), cls: 'phase-connecting' },
  waiting: { label: t('等待响应'), cls: 'phase-waiting' },
  streaming: { label: t('响应中'), cls: 'phase-streaming' },
  retrying: { label: t('重试中'), cls: 'phase-retrying' },
  queued: { label: t('排队中'), cls: 'phase-queued' },
}
const PHASE_FALLBACK_LABEL = t('进行中')
const PHASE_FALLBACK_CLS = 'running'
/** ： has ， need 「 this 1 in 」 */
const PHASE_TITLES: Record<string, string> = {
  connecting: t('请求已受理，正在选路、取凭证、建立上游连接'),
  waiting: t('上游请求已发出，正在等第一个字节到达（模型的思考时间也在这段）'),
  streaming: t('首帧已到，上游内容正在下发'),
  retrying: t('本轮尝试失败，正在退避等待或切换到下一个账号'),
  queued: t('上游模型繁忙，请求已排进上游队列；网关正按上游建议的时长等待后重发（不是登录态或额度问题）'),
  '': t('请求正在转发中，用时列显示的是已用时'),
}

/** for that ， 1 by 「 no has 」（ not ） */
export function phaseOf(entry: unknown): string {
  const raw = String((entry as RequestEntry | null)?.phase ?? '').trim().toLowerCase()
  return Object.prototype.hasOwnProperty.call(PHASES, raw) ? raw : ''
}

/** （empty → 「 in 」） */
export function phaseLabelOf(entry: unknown): string {
  const phase = phaseOf(entry)
  return phase ? PHASES[phase].label : PHASE_FALLBACK_LABEL
}

/**
 * when → （Nseconds / NMseconds， and 「 use when 」）。
 *
 * mustexcludingempty：`Number(null)` ** is 0**（ not is NaN）， Number.isFinite will 
 * 「 no has this 」 null 「0 ms」， is max(1, …) 1 empty 「1seconds」。
 * not to returnempty， use 。
 */
export function phaseElapsedText(ms: unknown): string {
  if (ms === null || ms === undefined || ms === '') return ''
  const value = Number(ms)
  if (!Number.isFinite(value) || value < 0) return ''
  const seconds = Math.max(1, Math.floor(value / 1000))
  if (seconds < 60) return t('{n}秒', { n: seconds })
  return t('{m}分{s}秒', { m: Math.floor(seconds / 60), s: seconds % 60 })
}

/**
 * current when （ms）， not to to null。 use server `phaseElapsedMs`（ and 
 * 1 when ， when not will ）；response no has this when use phaseStartedAt 1 。
 */
function phaseElapsedMsOf(entry: unknown): number | null {
  const raw: unknown = (entry as RequestEntry | null)?.phaseElapsedMs
  if (raw !== null && raw !== undefined && raw !== '') {
    const server = Number(raw)
    if (Number.isFinite(server) && server >= 0) return server
  }
  const started = Number((entry as RequestEntry | null)?.phaseStartedAt)
  if (!Number.isFinite(started) || started <= 0) return null
  const local = Date.now() - started
  return Number.isFinite(local) ? Math.max(0, local) : null
}

/** （ + ）。return， not —— and not */
export function phaseBadgeHtml(entry: unknown): string {
  const phase = phaseOf(entry)
  const label = phase ? PHASES[phase].label : PHASE_FALLBACK_LABEL
  const cls = phase ? PHASES[phase].cls : PHASE_FALLBACK_CLS
  const title = PHASE_TITLES[phase] || PHASE_TITLES['']
  return `<span class="badge tag ${cls}" title="${escapeHtml(title)}">`
    + `<span class="req-live-dot" aria-hidden="true"></span>${escapeHtml(label)}</span>`
}

/** when ；** has **（ not is ： is 「 no to 」） */
export function phaseElapsedLineHtml(entry: unknown): string {
  const text = phaseElapsedText(phaseElapsedMsOf(entry))
  if (!text) return ''
  const label = phaseLabelOf(entry)
  const tip = t('进入「{phase}」阶段已持续 {time}', { phase: label, time: text })
  return `<span class="req-phase-elapsed" title="${escapeHtml(tip)}">`
    + `${escapeHtml(text)}</span>`
}
