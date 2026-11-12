import * as React from 'react'
import { Badge, Button, HtmlContent, StatusDot } from '@ui'
import { t } from '../../i18n'
import { NameEyeButton } from '../accounts-panels'
import { maskName } from '../accounts-store'
import { numberText } from '../accounts-shared'
import { apiKeyRequestLabel, clientRequestLabel } from './source-filters'

export type RequestEntry = {
  id?: string; ts?: number; status?: number; error?: string | null
  provider?: string; providerLabel?: string; accountId?: string; accountName?: string
  model?: string; clientModel?: string; upstreamModel?: string
  clientReasoning?: string; upstreamReasoning?: string
  durationMs?: number; firstResponseMs?: number | null
  promptTokens?: number; completionTokens?: number; totalTokens?: number; cacheReadTokens?: number
  /** up by request up （ after `credit`）；null / = this 1 no up ，
   * and 0（） is —— before 「—」 */
  credit?: number | null
  attempts?: number; attemptDetails?: unknown[]; sensitiveHits?: unknown[]
  /** request（ after `is_test`； before no has this ） */
  isTest?: boolean
  phase?: string; phaseElapsedMs?: number | null; phaseStartedAt?: number | null
  /** client（ after `client`： by down User-Agent ；
   * empty / = no has —— old 、、request UA ） */
  client?: string
  clientLabel?: string
  clientEvidence?: import('../../../../ui-shared/client-identification').ClientEvidence
  apiKeyId?: string; apiKeyName?: string; apiKeySource?: string
}

export type VisibleColumn = { key: string; label: string; sel: string; track: string; align: 'left' | 'center' | 'right' }
function shared() {
  return window as unknown as {
    wbProviders?: { labelOf?: (id: string) => string }
    wbUnits?: { formatTokens?: (value: unknown) => string }
    wbRequestHover?: { hasProcessFacts?: (entry: unknown) => boolean }
    wbRequestDetail?: { open?: (id: string, row: RequestEntry | null) => unknown }
  }
}

/* ─── ───────────────────────────── */

/** use when ：seconds to ms，minutes up to seconds（ and OmniProxy 1 ） */
function formatDuration(ms: unknown): string {
  const rounded = Math.round(Number(ms) || 0)
  if (rounded < 1000) return `${rounded}ms`
  const seconds = Math.floor(rounded / 1000)
  if (seconds >= 60) return t('{m}分{s}秒', { m: Math.floor(seconds / 60), s: seconds % 60 })
  const millis = rounded % 1000
  return millis > 0 ? t('{s}秒{ms}ms', { s: seconds, ms: millis }) : t('{n}秒', { n: seconds })
}

/**
 * （tok/s） ： and then in this —— can can 
 * by 、 can can by total use when （）， not will for not up 。
 */
const SPEED_TIP = t('生成速度：输出 token ÷ 流式段用时（总用时 − 首响）；流式段不足总用时一半时按总用时计')

/** down （ms）： by total use when ， tokensPerSecond */
const STREAM_MS_FLOOR = 500

/**
 * （tok/s）： token ÷ use when （total use when − ）。
 *
 * ** not **「 etc 」 when —— that if is （ before up 1 
 * not ）， will 。， is 1 **** when 
 * not can by ： up not ，token in after 1 
 * 1 to （ up 、path、 is this ），486 token ÷ 0.6 seconds
 * can 800+ tok/s —— that is 「 down 」， not is 「」。
 * ， not by total use when （ can to ， not to ）：
 * · ≥ STREAM_MS_FLOOR（response duration ≈ ，）；
 * · ≥ total use when 1 （ need request， not can ）。
 * / total use when as 0（failure token 、 old ）→ empty， use 。
 * in no has completionTokens（），empty —— has has 。
 */
function tokensPerSecond(entry: RequestEntry): string {
  const tokens = Number(entry.completionTokens) || 0
  const duration = Number(entry.durationMs) || 0
  if (tokens <= 0 || duration <= 0) return ''
  const first = Number(entry.firstResponseMs)
  const streamMs = Number.isFinite(first) && first > 0 && first < duration ? duration - first : 0
  const streaming = streamMs >= STREAM_MS_FLOOR && streamMs * 2 >= duration
  const rate = tokens / ((streaming ? streamMs : duration) / 1000)
  return `${rate >= 100 ? Math.round(rate) : rate.toFixed(1)} tok/s`
}

/**
 * ： up to for request when （OmniProxy ttfb ）。null（ old no has this /
 * request in before failure）「-」—— formatDuration will null 0ms，empty。
 */
function formatFirstResponse(ms: unknown): string {
  const value = Number(ms)
  if (!Number.isFinite(value) || value <= 0) return '-'
  return formatDuration(value)
}

/** usage： and to units.js formatTokens（ and / 1 ，
 * in 「 / 」 and 「k / M」 in that 1 ）； not in when */
function formatTokens(value: unknown): string {
  return shared().wbUnits?.formatTokens?.(value) ?? (Number(value) || 0).toLocaleString('zh-CN')
}

/**
 * in 「 already use when 」：now - ts seconds（ in not 1 seconds 1 seconds —— 「0seconds」 no ）。
 * 1 seconds， this 1 seconds 1 。
 */
function formatElapsed(ts: unknown): string {
  const elapsed = Math.max(0, Date.now() - (Number(ts) || 0))
  return t('{n}秒', { n: Math.max(1, Math.floor(elapsed / 1000)) })
}

/** in ： in / ， as 0 when none， to 「-」 */
function cacheRate(entry: RequestEntry): string {
  const prompt = Number(entry.promptTokens) || 0
  if (prompt <= 0) return '-'
  return `${((Number(entry.cacheReadTokens) || 0) / prompt * 100).toFixed(2)}%`
}

/**
 * success and after `RequestEntry::is_success` for ：2xx ** and ** no has error need 。request 
 * HTTP 200 in response， after up / error can error —— by 
 * will this failure， and after （status=ok/error）、success for not up 。
 */
export function isOk(entry: RequestEntry): boolean {
  const status = Number(entry.status) || 0
  return status >= 200 && status < 300 && !entry.error
}

/**
 * is 「 in in 」： after is status=0 and no has error need —— status=0 need is old 
 * 「 no requestfailure」 ， by failure（ that failure already ， not this in ）。
 */
export function isRunning(entry: RequestEntry): boolean {
  return (Number(entry?.status) || 0) === 0 && !entry?.error
}

/**
 * 1 request in DOM （ use ）： id（ down and 、 1 ），
 * no has id old ts； to empty —— will （ to 「」 ）， is 
 * not to ， not not as 。
 */
const rowKey = (entry: RequestEntry): string => String(entry?.id || entry?.ts || '')

/** this requestall total（ no has when as 0） */
function countRetries(entry: RequestEntry): number {
  const details = Array.isArray(entry?.attemptDetails) ? entry.attemptDetails : []
  return details.reduce<number>((sum, item) => {
    const retries = (item as { retries?: unknown[] } | null)?.retries
    return sum + (Array.isArray(retries) ? retries.length : 0)
  }, 0)
}

/** ：providers.js directory（）→ → id（ that 1 use after to providerLabel） */
export function providerLabelOf(id: string): string {
  return t(shared().wbProviders?.labelOf?.(id) || id)
}

/** key ：ts in 1 ms can can （ and request）， 1 after ， React key */
export function uniqueRowKeys(rows: RequestEntry[]): string[] {
  const seen = new Set<string>()
  return rows.map((entry, index) => {
    const base = rowKey(entry) || `i${index}`
    let key = base
    let n = 1
    while (seen.has(key)) key = `${base}#${n++}`
    seen.add(key)
    return key
  })
}

/* ─── ───────────────────────────── */

/**
 * in 「」：** has **（empty = no to ，）。 and 
 * this 1 ： that etc 「 no has to 」= failure， use 「-」； in is 「 no to 」。
 */
function runningFirstLine(entry: RequestEntry): React.ReactNode {
  const value = Number(entry.firstResponseMs)
  if (!Number.isFinite(value) || value <= 0) return null
  return (
    <span className='req-dur-line sub' title={t('首响：上游首帧到达的耗时（请求仍在转发中）')}>
      {t('首响 {time}', { time: formatDuration(value) })}
    </span>
  )
}

/**
 * 1 （， .req-xxx and ta-* for ）。
 *
 * by key not is in ：「 by 、」，「 1 」 has 
 * 1 —— set when not will 1 。 and old 1 （CSS and by 
 * this ），empty old （empty when has .req-none， no has .req-retry）。
 */
export function RequestCell({ entry, column, namesHidden }: { entry: RequestEntry; column: VisibleColumn; namesHidden: boolean }): React.ReactNode {
  const { align, key } = column
  const base = column.sel.slice(1) + (key === 'dur' || key === 'speed' ? ' req-num' : '')
  const className = `${base} ta-${align}`

  switch (key) {
    case 'time': {
      if (!entry.ts) return <span key={key} className={className}>—</span>
      const d = new Date(entry.ts)
      const pad = (n: number) => String(n).padStart(2, '0')
      // （ 30 ）， and when ， not can 
      return (
        <span key={key} className={className}>
          <span>{`${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`}</span>
          <span>{`${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`}</span>
        </span>
      )
    }

    case 'target': {
      const id = String(entry.provider ?? '').trim()
      const fromBackend = String(entry.providerLabel ?? '').trim()
      // after to is （「WorkBuddy 」 this ）， and directory 1 need 1 
      const provider = fromBackend ? t(fromBackend) : (id ? providerLabelOf(id) : '')
      // as empty = request in before failure（ after ：none when to empty）， to not is empty
      return (
        <span key={key} className={className}>
          {provider ? (
            <span className='req-provider'
              title={id && id !== provider ? t('{name}（{id}）', { name: provider, id }) : provider}>
              {provider}
            </span>
          ) : (
            <span className='req-provider is-empty'
              title={t('这条明细没有记录提供商（旧数据或请求未走到转发）')}>—</span>
          )}
          {entry.accountName ? (
            <span className='req-acct' title={namesHidden ? undefined : entry.accountId || entry.accountName}>
              {namesHidden ? maskName(entry.accountName) : entry.accountName}
            </span>
          ) : (
            <span className='req-acct is-empty'
              title={t('请求在任何账号接手之前就失败了')}>—</span>
          )}
        </span>
      )
    }

    case 'retry': {
      const attempts = Number(entry.attempts) || 1
      const identity = rowKey(entry)
      const tags: React.ReactNode[] = []
      // 「」： this 1 is that 「」 （ after `is_test`）。 and 
      // not —— ** not is ** ， is 1 （brand ， and 「」），
      // not use render button。 not is 1 ： is 「 this 1 is 」，
      // and 「 / 」 1 ， this is set can ， as 1 1 not 
      if (entry.isTest) {
        tags.push(
          <Badge key='test' variant='brand' shape='tag'
            title={t('面板内发起的模型调用，消耗额度并记录 Key，不计入生产报表')}>{clientRequestLabel(entry)}</Badge>,
        )
      }
      // to （ need ）； attempts > 1： no when 
      // request can ， not is empty
      const showChain = shared().wbRequestHover?.hasProcessFacts?.(entry) ?? attempts > 1
      if (showChain) {
        const count = attempts > 1 ? attempts : countRetries(entry)
        tags.push(
          // Badge render ** button**： by `[data-req-hover]` 、
          // can focusin ，span to not 。 and data-* by Base UI and to this 
          // button up （ not is ）， then to （ `warn` warning）
          <Badge key='chain' variant='warning' shape='tag' render={<Button size='default' variant='ghost' type='button' />}
            className='req-hover-tag' data-req-hover='chain' data-req-id={identity}>
            {count > 1 ? t('重试 {n}', { n: count }) : t('重试')}
          </Badge>,
        )
      }
      // in ： is 「 in empty」。 1 「」（ this 1 has 52px， will 
      // 「 N」）； to ，aria-label up 。
      // （sensitive） is this to ： not is 「failure」 not is 「」，
      // is ； is 1 ， not will has 
      if (Array.isArray(entry.sensitiveHits) && entry.sensitiveHits.length) {
        tags.push(
          <Badge key='sensitive' variant='sensitive' shape='tag' render={<Button size='default' variant='ghost' type='button' />}
            className='req-hover-tag' data-req-hover='sensitive' data-req-id={identity}
            aria-label={t('命中了敏感词')}>{t('敏')}</Badge>,
        )
      }
      if (!tags.length) return <span key={key} className={`req-none ta-${align}`}>-</span>
      return <span key={key} className={className}>{tags}</span>
    }

    case 'status': {
      if (isRunning(entry)) {
        const phase = window.wbRequestPhase
        // no （ is need ， when this 1 not this empty）： before 
        if (!phase?.badgeHtml) {
          return (
            <span key={key} className={className}>
              <span className='req-status-badge'>
                <Badge shape='tag' variant='info' title={t('请求正在转发中，用时列显示的是已用时')}>
                  <StatusDot pulse className='size-[5px] bg-current' />{t('进行中')}
                </Badge>
              </span>
            </span>
          )
        }
        const elapsed = phase.elapsedLineHtml?.(entry) ?? ''
        return (
          <span key={key} className={className}>
            {/* and when by wbRequestPhase HTML（ 1 ，must 1 ） */}
            <HtmlContent className='req-status-badge' html={phase.badgeHtml(entry)} render={<span />} />
            {elapsed ? <HtmlContent className='req-status-line' html={elapsed} render={<span />} /> : null}
          </span>
        )
      }
      const status = Number(entry.status) || 0
      const ok = isOk(entry)
      // to this status=0 「 no requestfailure」 ： 0 will HTTP ，「failure」
      const title = !ok && status >= 200 && status < 300
        ? t('HTTP {status}，但响应体阶段出错：见错误列', { status })
        : undefined
      return (
        <span key={key} className={className}>
          <Badge shape='tag' variant={ok ? 'success' : 'destructive'} title={title}>
            {status || t('失败')}
          </Badge>
        </span>
      )
    }

    case 'model': {
      const client = String(entry.clientModel ?? '').trim()
      const upstream = String(entry.upstreamModel ?? '').trim()
      const shown = String(entry.model ?? '').trim()
      const clientLevel = String(entry.clientReasoning ?? '').trim()
      const upstreamLevel = String(entry.upstreamReasoning ?? '').trim()
      const tag = (name: string, level: string) => (level ? `${name}(${level})` : name)
      // and request 1 （request）→ ； not 1 （ / ）→ 
      if (!client || !upstream || upstream.toLowerCase() === client.toLowerCase()) {
        const text = tag(shown, upstreamLevel || clientLevel)
        return <span key={key} className={className} title={text}>{text || '—'}</span>
      }
      const upText = tag(upstream, upstreamLevel)
      const downText = tag(client, clientLevel)
      return (
        <span key={key} className={`${className} req-model-split`}>
          <span className='req-model-line'
            title={t('转发到上游的模型：{model}', { model: upText })}>⬆️ {upText}</span>
          <span className='req-model-line sub'
            title={t('下游请求的模型：{model}', { model: downText })}>⬇️ {downText}</span>
        </span>
      )
    }

    case 'client': {
      const label = clientRequestLabel(entry)
      return <span key={key} className={className}
        title={entry.clientEvidence?.userAgent || t('来源客户端：{client}', { client: label })}>{label}</span>
    }

    case 'apiKey': {
      const label = apiKeyRequestLabel(entry)
      return <span key={key} className={className} title={entry.apiKeyId || label}>{label}</span>
    }

    case 'dur': {
      if (isRunning(entry)) {
        // in no has durationMs（）： already use when （ in ）
        return (
          <span key={key} className={className}>
            <span className='req-dur-line' title={t('已用时（请求仍在转发中）')}>{formatElapsed(entry.ts)}</span>
            {runningFirstLine(entry)}
          </span>
        )
      }
      return (
        <span key={key} className={className}>
          <span className='req-dur-line'>{formatDuration(entry.durationMs)}</span>
          <span className='req-dur-line sub' title={t('首响：上游首帧到达的耗时')}>
            {t('首响 {time}', { time: formatFirstResponse(entry.firstResponseMs) })}
          </span>
        </span>
      )
    }

    case 'speed': {
      if (isRunning(entry)) return <span key={key} className={className} />
      const speed = isOk(entry) ? tokensPerSecond(entry) : ''
      return <span key={key} className={`${className}${speed ? '' : ' req-none'}`} title={SPEED_TIP}>{speed || '—'}</span>
    }

    case 'credit': {
      // in ： need etc up after 1 chunk usage to ， no has can to 
      // （ and 「usage」 1 ：empty「-」）。
      if (isRunning(entry)) return <span key={key} className={className} />
      // failurerequest not usage（ after normalize already credit None）： this and 
      // 「usage」 use 1 ， 「failure」「 this 1 no up 」。
      if (!isOk(entry)) {
        return <span key={key} className={`${className} is-empty`} title={t('失败请求不记录用量')}>—</span>
      }
      const value = entry.credit
      if (value === null || value === undefined || !Number.isFinite(Number(value))) {
        return (
          <span key={key} className={`${className} is-empty`} title={t('上游没有返回积分消耗')}>—</span>
        )
      }
      return (
        <span key={key} className={className}
          title={t('本次请求消耗 {credit} 积分', { credit: numberText(value) })}>
          {numberText(value)}
        </span>
      )
    }

    case 'usage': {
      // in ：usage need etc ， no has can to —— empty「-」
      if (isRunning(entry)) return <span key={key} className={className} />
      // failurerequest token by after 1 ，「0」 will as this 
      if (!isOk(entry)) {
        return (
          <span key={key} className={className} title={t('失败请求不记录用量')}>
            <span className='req-usage-line'>in: - / out: - / all: -</span>
            <span className='req-usage-line sub'>{t('缓存读取: - / 命中率: -')}</span>
          </span>
        )
      }
      const line1 = `in: ${formatTokens(entry.promptTokens)} / out: ${formatTokens(entry.completionTokens)} / all: ${formatTokens(entry.totalTokens)}`
      const line2 = t('缓存读取: {cache} / 命中率: {rate}', {
        cache: formatTokens(entry.cacheReadTokens), rate: cacheRate(entry),
      })
      return (
        <span key={key} className={className} title={[line1, line2].join('\n')}>
          <span className='req-usage-line'>{line1}</span>
          <span className='req-usage-line sub'>{line2}</span>
        </span>
      )
    }

    case 'error': {
      // in ：error no has ，empty —— 「-」 will 「 no 」，empty is 「 no to has error when 」
      if (isRunning(entry)) return <span key={key} className={className} />
      if (entry.error) {
        return <span key={key} className={className} title={String(entry.error)}>{String(entry.error)}</span>
      }
      return <span key={key} className={`req-none ${base} ta-${align}`}>-</span>
    }

    case 'detail': {
      const id = entry.id ? String(entry.id) : ''
      if (!id && !entry.clientEvidence) return <span key={key} className={`req-none ${base} ta-${align}`}>-</span>
      return (
        <span key={key} className={className}>
          <Button size='sm' variant='outline' className='req-detail-btn' title={t('请求详情')}
            onClick={() => { void shared().wbRequestDetail?.open?.(id, entry) }}>{t('详情')}</Button>
        </span>
      )
    }

    default:
      return null
  }
}

/** ： by can ， and 1 （ visibleColumns） */
export function headRow(columns: VisibleColumn[], namesHidden: boolean): React.ReactNode {
  return (
    <div className='req-head' key='__head'>
      {columns.map(column => (
        <span key={column.key} data-col={column.key}
          className={`${column.sel.slice(1)}${column.key === 'dur' || column.key === 'speed' ? ' req-num' : ''} ta-${column.align}`}
          title={column.key === 'speed' ? SPEED_TIP : undefined}>
          {column.key === 'target' ? (
            <span className='req-target-label'>
              {column.label}
              <NameEyeButton hidden={namesHidden} />
            </span>
          ) : column.label}
        </span>
      ))}
    </div>
  )
}
