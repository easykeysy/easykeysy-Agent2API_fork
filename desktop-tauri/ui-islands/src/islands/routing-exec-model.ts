import { t } from '../i18n'
import { formatDuration } from './routing-ui'
import {
  attemptState, candidateReason, errorText, isTraceCancelled, isTraceRunning, isTraceSuccess,
  traceAttempts, traceCandidates, tracePhase, traceProvider, type TraceAttempt, type TraceCandidate,
} from './routing-trace-model'
import type { ReplayRow } from './routing-types'

export const execCandidateKey = (candidate: TraceCandidate) => candidate.candidateId || `${candidate.memberId || ''}:${candidate.id}`
export const execAttemptName = (attempt: TraceAttempt) => attempt.account || attempt.accountName || attempt.accountId || t('Account not recorded')

// 同一账号可参加不同成员模型；只有旧记录缺少成员标识时才退回账号匹配。
export function execAttemptMatches(candidate: TraceCandidate, attempt: TraceAttempt): boolean {
  if (candidate.candidateId && attempt.candidateId) return candidate.candidateId === attempt.candidateId
  if (candidate.memberId && attempt.memberId && candidate.memberId !== attempt.memberId) return false
  if (candidate.provider && attempt.provider && candidate.provider !== attempt.provider) return false
  if (candidate.model && attempt.model && candidate.model !== attempt.model) return false
  return attempt.accountId ? candidate.id === attempt.accountId
    : Boolean(candidate.name && candidate.name === (attempt.account || attempt.accountName))
}

export function execLastAttempt(row: ReplayRow, attempts = traceAttempts(row)): TraceAttempt | undefined {
  const last = attempts[attempts.length - 1]
  return last && (last.inferred || last.index + 1 >= (row.attempts || 1)) ? last : undefined
}

export function execCarrier(row: ReplayRow) {
  const attempts = traceAttempts(row), candidates = traceCandidates(row)
  const last = execLastAttempt(row, attempts)
  const pickedId = row.accountId || row.route?.picked
  const candidate = candidates.find(item => row.route?.pickedCandidate ? execCandidateKey(item) === row.route.pickedCandidate
    : last ? execAttemptMatches(item, last) : Boolean(pickedId && item.id === pickedId))
  const running = isTraceRunning(row), cancelled = isTraceCancelled(row), success = isTraceSuccess(row)
  const name = row.accountName || last?.account || last?.accountName || candidate?.name || pickedId || last?.accountId
  const provider = traceProvider(row.provider || last?.provider || candidate?.provider, row.providerLabel || last?.providerLabel || candidate?.providerLabel)
  const carrying = Boolean(name && (success || running && (last ? attemptState(row, last) !== 'failed' : row.phase !== 'retrying')))
  let reason = errorText(last?.reason || candidate?.reason)
    || (last?.stickyHit || row.route?.stickyHit ? t('Session binding hit') : t('Routing reason not recorded'))
  const previous = last && attempts.find(attempt => attempt.index === last.index - 1)
  if (previous && attemptState(row, previous) === 'failed') {
    reason = `${t('Switched to this account after {account} failed', { account: execAttemptName(previous) })}；${reason}`
  }
  if (cancelled) return { label: t('Request outcome'), name: t('Request cancelled'), reason: errorText(row.error) || t('The client cancelled the request'), carrying: false, candidate, last }
  if (!running && !success) return { label: t('Request outcome'), name: t('Request did not complete successfully'),
    reason: errorText(row.error || last?.error || last?.notice) || t('Failure reason not recorded'), carrying: false, candidate, last }
  if (!carrying && success) return { label: t('Final handling account'), name: t('Account not recorded'), reason: t('The request succeeded, but the serving account was not recorded'), carrying: false, candidate, last }
  if (!carrying) return { label: t('Current status'), name: tracePhase(row.phase),
    reason: last && attemptState(row, last) === 'failed' ? errorText(last.error || last.notice) || t('This attempt failed; backing off or switching to the next account')
      : t('No serving account recorded yet'), carrying: false, candidate, last }
  return { label: running ? t('Current handling account') : t('Final handling account'), name: `${provider} · ${name}`, reason, carrying, candidate, last }
}

export type ExecAccount = {
  key: string
  name: string
  provider: string
  model?: string | null
  current: boolean
  kind: 'tried' | 'skipped' | 'idle'
  label: string
  tone: 'accent' | 'fail' | 'muted'
  reason: string
}

export function execAccounts(row: ReplayRow): ExecAccount[] {
  const attempts = traceAttempts(row), candidates = traceCandidates(row)
  const carrier = execCarrier(row)
  const fromAttempt = (attempt: TraceAttempt, candidate?: TraceCandidate): ExecAccount => {
    const current = carrier.carrying && (carrier.last ? carrier.last.index === attempt.index
      : Boolean(candidate && carrier.candidate && execCandidateKey(carrier.candidate) === execCandidateKey(candidate)))
    const cancelled = isTraceCancelled(row) && carrier.last?.index === attempt.index
    const state = attemptState(row, attempt)
    return {
      key: candidate ? execCandidateKey(candidate) : `attempt:${attempt.index}`,
      name: candidate?.name || execAttemptName(attempt),
      provider: traceProvider(candidate?.provider || attempt.provider, candidate?.providerLabel || attempt.providerLabel),
      model: candidate?.model || attempt.model,
      current,
      kind: 'tried',
      label: cancelled ? t('Canceled') : current ? isTraceRunning(row) ? t('Current handling account') : t('Final handling account')
        : state === 'failed' ? t('Attempt failed') : state === 'success' ? t('Upstream responded') : state === 'running' ? tracePhase(row.phase) : t('Result not recorded'),
      tone: cancelled ? 'muted' : current ? 'accent' : state === 'failed' ? 'fail' : 'muted',
      reason: errorText(cancelled ? row.error : state === 'failed' ? attempt.error || attempt.notice : attempt.reason || candidate?.reason || attempt.notice)
        || (attempt.status != null ? `HTTP ${attempt.status}` : t('No call details saved')),
    }
  }
  const entries = candidates.map((candidate): ExecAccount => {
    const attempt = [...attempts].reverse().find(item => execAttemptMatches(candidate, item))
    if (attempt) return fromAttempt(attempt, candidate)
    const current = carrier.carrying && Boolean(carrier.candidate && execCandidateKey(carrier.candidate) === execCandidateKey(candidate))
    return {
      key: execCandidateKey(candidate), name: candidate.name || candidate.id || t('Account not recorded'),
      provider: traceProvider(candidate.provider, candidate.providerLabel), model: candidate.model, current,
      kind: current || candidate.state === 'tried' ? 'tried' : candidate.kind === 'skipped' ? 'skipped' : 'idle',
      label: current ? isTraceRunning(row) ? t('Current handling account') : t('Final handling account') : candidate.state === 'tried' ? t('Already attempted')
        : candidate.kind === 'skipped' ? t('Skipped') : t('Not called'),
      tone: current ? 'accent' : 'muted',
      reason: current ? carrier.reason : candidate.state === 'tried' ? errorText(candidate.reason) || t('No call details saved')
        : candidate.kind === 'skipped' ? errorText(candidateReason(row, candidate))
          : [errorText(candidate.reason), candidateReason(row, candidate)].filter(Boolean).join('; '),
    }
  })
  const hasCurrent = entries.some(entry => entry.current)
  const currentMatches = (attempt: TraceAttempt) => (row.accountId || row.route?.picked)
    ? attempt.accountId === (row.accountId || row.route?.picked)
    : Boolean(row.accountName && (attempt.account || attempt.accountName) === row.accountName)
  const current = !hasCurrent && (carrier.carrying || !carrier.last && (row.accountId || row.accountName))
    && !attempts.some(currentMatches) ? traceAttempts({ ...row, attemptDetails: [] }) : []
  if (current.length) {
    const attempt = current[0]
    entries.push({ ...fromAttempt(attempt), key: 'current-account', current: carrier.carrying,
      label: carrier.carrying ? isTraceRunning(row) ? t('Current handling account') : t('Final handling account') : t('Result not recorded'),
      reason: carrier.carrying ? carrier.reason : t('No call details saved') })
  }
  const missing = attempts.filter(attempt => !candidates.some(candidate => execAttemptMatches(candidate, attempt)))
  return [...entries, ...missing.map(attempt => fromAttempt(attempt))]
}

const reading = (value?: number | null) => value != null && Number.isFinite(value) && value >= 0 ? value : null
const relative = (row: ReplayRow, value?: number | null) => value != null && Number.isFinite(value) && value >= row.ts ? value - row.ts : null

export type ExecSegment = { start: number; end: number; tone: 'wait' | 'frame' | 'stream' | 'fail'; label: string }
export type ExecTiming = {
  attempt: TraceAttempt
  duration: number | null
  response: number | null
  segments: ExecSegment[]
  label: string
  tone: 'accent' | 'fail' | 'muted'
}

export function execTiming(row: ReplayRow, now: number) {
  const attempts = traceAttempts(row), last = execLastAttempt(row, attempts)
  const running = isTraceRunning(row)
  const total = running ? Math.max(0, now - row.ts) : reading(row.durationMs)
  const firstFrame = reading(row.firstResponseMs)
  const response = last ? relative(row, last.responseAt) : null
  const timings: ExecTiming[] = attempts.map((attempt, index) => {
    const started = relative(row, attempt.startedAt)
    const responded = relative(row, attempt.responseAt)
    const state = attemptState(row, attempt)
    const cancelled = isTraceCancelled(row) && attempt.index === last?.index
    const finished = relative(row, attempt.finishedAt) ?? (state === 'running' && attempt.index === last?.index ? total : null)
    const nextStart = relative(row, attempts[index + 1]?.startedAt)
    const end = finished ?? responded
    const frame = firstFrame != null && started != null && firstFrame >= started
      && (finished == null || firstFrame <= finished) && (nextStart == null || firstFrame < nextStart) ? firstFrame : null
    const segments: ExecSegment[] = []
    const add = (start: number, stop: number, tone: ExecSegment['tone'], label: string) => {
      if (stop > start) segments.push({ start, end: stop, tone, label: `${label} ${formatDuration(stop - start)}` })
    }
    // 缺少起止时间的旧尝试不补画；响应头到达与上游首帧是两个独立读数。
    if (started != null && end != null && end >= started) {
      const boundary = Math.min(end, frame ?? end, responded != null && responded >= started ? responded : end)
      add(started, boundary, state === 'failed' && frame == null && !cancelled ? 'fail' : 'wait', t('Awaiting response'))
      if (frame != null && frame <= end) {
        add(boundary, frame, 'frame', t('Waiting for first byte'))
        add(frame, end, state === 'failed' && !cancelled ? 'fail' : 'stream', t('Output'))
      } else {
        add(boundary, end, state === 'failed' && !cancelled ? 'fail' : 'frame', t('After response arrival'))
      }
    }
    return {
      attempt, duration: started != null && finished != null && finished >= started ? finished - started : null,
      response: responded, segments,
      label: cancelled ? t('Canceled') : state === 'running' ? tracePhase(row.phase) : state === 'failed'
        ? [attempt.status != null ? String(attempt.status) : '', t('Failed')].filter(Boolean).join(' · ')
        : state === 'success' ? t('Success') : t('Result not recorded'),
      tone: cancelled ? 'muted' : state === 'failed' ? 'fail' : state === 'running' ? 'accent' : 'muted',
    }
  })
  const scale = Math.max(total ?? 0, ...timings.flatMap(timing => timing.segments.map(segment => segment.end)), 1)
  return { total, firstFrame, response, scale, timings }
}

export function execScopeLabel(row: ReplayRow): string {
  const snapshot = row.route?.configSnapshot
  if (snapshot?.groupId) return `${t('Custom route groups')} · ${snapshot.groupName || snapshot.scope}`
  const scope = snapshot?.scope ?? row.route?.scope
  return scope == null ? t('Unrecorded') : scope ? `${t('Independent model configuration')} · ${scope}` : t('Inherits global configuration')
}
