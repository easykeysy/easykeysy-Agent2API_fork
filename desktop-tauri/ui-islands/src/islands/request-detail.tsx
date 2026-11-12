import * as React from 'react'
import { createRoot } from 'react-dom/client'
import { requestApiKeyLabel, requestClientLabel } from '../../../ui-shared/model-api-key'
import { ClientIdentificationDetail } from '../../../ui-shared/client-identification-detail'
import type { ClientEvidence } from '../../../ui-shared/client-identification'
import { clientIdentificationApi } from './settings-client-identification'
import {
  Badge, BadgeDot, Button, Dialog, DialogBody, DialogContent, DialogFooter,
  DialogHeader, DialogTitle, SegmentedControl, StatusDot, Table, TableBody,
  TableCell, TableHead, TableHeader, TableRow, type SegmentedControlOption,
} from '@ui'
import { t } from '../i18n'
import { HtmlContent } from '@ui'
import {
  PreviewPane, RawPane, RequestEmpty as Empty, hasDebugTraffic,
  type RawState, type DebugState, type DebugTraffic,
} from './request-raw-panes'
import { RouteReplayFacts } from './routing-replay'
import type { ReplayRow, RouteDecision } from './routing-types'

/**
 * 「request」 ****（request / by / for / · ）。
 *
 * ui/request-detail.js —— that use innerHTML .modal-mask / .seg /
 * .req-detail-* that ， index.html #req-detail-modal。
 * for and ** 1 **：`window.wbRequestDetail.open(id, row)` /
 * `.close()`， use （requests-panel.js 「」 by ） 1 not use ； by 
 * React ， not #req-detail-modal 1 id（ that DOM by delete）。
 *
 * ── ─────────────────────────────────────
 * ① request（default）—— ： when / / when / /
 * / / / error / / in 。** use 
 * for ** `open(id, row)`（current 1 ， not request）；
 * row （ in before new ） when 「 already new ，」。
 * ② by —— （ for `route` ）： /
 * / ，「 when Why this 」。 and 「」
 * （ that 「、 no 」）。 row， not request； old or before 
 * failure request no has when ** **（ and 1 ：
 * 1 empty）。 「 in by 」 by request。
 * ③ for —— `getStatsRequestRaw(id)` down （request + response），
 * and in conversation-preview.js（， this and to empty）。
 * in **** request up （ and up 1 ，
 * for OmniProxy ）， 404 / empty to empty： Reason is 
 * 「request when no 」， its is already —— not is error。
 * ④ —— ** up **（`getDebugTraffic` ：
 * request / request / response / response， already ）， as 。
 * no （ or this already 500 ） when ** ** ——
 * is optional ， not this use in empty。
 *
 * ── need need ──────────────────────────────
 * · in open when ** and **， to （ that and ，
 * not ）；
 * · ** not request**， —— pane 、 use hidden 
 * display，pane DOM（ <details> ）；
 * by scrollTops / （display:none will 、）；
 * · when ** after 1 open**（ seq token），
 * to old response 1 。
 *
 * ── and / request ─────────────────────────────────
 * in 「」 when `open(id, row)`； （ when / when / 
 * etc ） is requests-panel.js that IIFE has ， this by 1 has 1 ，
 * 、 when 1 。
 * in （`isRunning`） when 「request」： after 
 * `terminateStatsRequest(id)` ， up and by 「request already 
 * 」。** not update** —— new will that ，
 * by and in hint ； not in in （ or up ） when after 404，hint 。
 *
 * and conc-dialog / request-clear-modal 1 ： by need 
 * div， unmount + remove（ not index.html empty）。
 */
/* ─── and ─────────────────────────────── */

/** 。`route` / `raw` by is available， not and 「default in 」 */
type TabKey = 'detail' | 'route' | 'preview' | 'raw'

const TAB_OPTIONS: readonly SegmentedControlOption<TabKey>[] = [
  { value: 'detail', label: t('请求详情') },
  { value: 'route', label: t('路由回放') },
  { value: 'preview', label: t('预览对话') },
  { value: 'raw', label: t('原始报文') },
]
/** default as 「request」（ old ；`route` / `raw` to before not in ） */
const DEFAULT_TAB: TabKey = 'detail'

/** 1 （ after AttemptDetail use to ） */
type AttemptDetail = {
  provider?: string
  providerLabel?: string
  account?: string
  /** up ；null / = this no has */
  status?: number | string | null
  error?: unknown
  /** ：Reason + （ms） */
  retries?: { reason?: unknown; delayMs?: unknown }[]
  notice?: unknown
  /** this 1 to up （ = no ）*/
  bodyBytes?: unknown
}

/** in ： + */
type SensitiveHit = { word?: unknown; count?: unknown }

/** is 「」 need this （ for and 1 ） */
type ProviderFields = { provider?: string; providerLabel?: string }

/**
 * for 。** to ** ； its （phase / phaseElapsedMs etc ）
 * to `wbRequestPhase.badgeHtml`， that 。 for JS， / 
 * 1 by empty。
 */
type RequestRow = {
  ts?: number | string | null
  status?: number | string | null
  error?: unknown
  /** ： use when / （ms）、、 —— after can can to / / null */
  durationMs?: unknown
  firstResponseMs?: unknown
  attempts?: unknown
  promptTokens?: unknown
  completionTokens?: unknown
  totalTokens?: unknown
  cacheReadTokens?: unknown
  /** and etc ： down / up 1 */
  model?: unknown
  clientModel?: unknown
  upstreamModel?: unknown
  clientReasoning?: unknown
  upstreamReasoning?: unknown
  accountId?: unknown
  accountName?: unknown
  attemptDetails?: AttemptDetail[]
  sensitiveHits?: SensitiveHit[]
  /** （ after `route` ； old / no to is null） */
  route?: unknown
  /** request（ after `is_test` to ； no has this ） */
  isTest?: boolean
  client?: string
  clientLabel?: string
  clientEvidence?: ClientEvidence
  apiKeyId?: string
  apiKeyName?: string
  apiKeySource?: string
} & ProviderFields

/** by ： can / in （ use ）/ already （ use and not ） */
type TerminateState = 'idle' | 'pending' | 'accepted'

/** / ： 1 1 */
const GRID_CELL = 'border-b border-hairline px-2.5 py-[7px] align-top'
const ATTEMPT_CELL = 'border border-hairline px-2 py-1 align-top'
const ATTEMPT_HEAD = 'border border-hairline bg-surface-2 px-2 py-1 text-[11.5px] tracking-normal'
/** will as can ， */
const NO_HOVER = 'hover:[&>td]:bg-transparent'

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：
 * workbuddyDesktop / wbApp / wbConfirm is use ， if declare 1 ，
 * and will not 1 TS2717 —— and when 。file
 * wbRequestDetail（file）。
 */
type SharedWindow = {
  /** ：getStatsRequestRaw / getDebugTraffic not to to 404， not in in 404 */
  workbuddyDesktop?: {
    getStatsRequestRaw(id: string): Promise<unknown>
    getDebugTraffic(id: string): Promise<unknown>
    terminateStatsRequest(id: string): Promise<unknown>
  }
  /**
   * toast and esc in ** not use **， as ： old use esc 
   * innerHTML， that JSX （React ， esc 1 
   * will ， `&` `&amp;`）；request by old hint，
   * not toast（toast seconds after no ）。
   */
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    esc?: (value: unknown) => string
  }
  /** ask html is HTML（ use ），okClass='danger' when */
  wbConfirm?: { ask?: (options: { title?: string; html?: string; okText?: string; okClass?: string }) => Promise<boolean> }
  /** for （， and empty） */
  wbConversationPreview?: { render?: (requestBody: unknown, responseBody: unknown) => string }
  /** and when （ 1 need in 1 ， request-phase.js module header） */
  wbRequestPhase?: { badgeHtml?: (entry: unknown) => string; elapsedLineHtml?: (entry: unknown) => string }
  /** directory（ after label → directory → id） */
  wbProviders?: { labelOf?: (id: string) => string }
  /** Token and （units.js； in 「 / 」 and 「k / M」 in that 1 ） */
  wbUnits?: { formatTokens?: (value: unknown) => string }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/** after ： not to （ old is when 1 workbuddyDesktop， not to ） */
function api() {
  const bridge = shared().workbuddyDesktop
  if (!bridge) throw new Error(t('后端桥不可用'))
  return bridge
}

/**
 * after 1 open： when to old response（ old seq）。
 * not is state —— in await after ，must can **** to 「 in is not is 
 * new 1 」，React state update not to this 1 。
 */
let seq = 0
/* ─── and ───────────────────── */
// requests-panel.js is that IIFE has ， this is 1 ， when 1 。

/** when ：（ timeCell ， 1 down ） */
function fmtTime(ts: RequestRow['ts']): string {
  if (!ts) return '—'
  const d = new Date(ts)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
    + ` ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

/** use when ：seconds to ms，minutes up to seconds（ and formatDuration ） */
function fmtDuration(ms: unknown): string {
  const rounded = Math.round(Number(ms) || 0)
  if (rounded < 1000) return `${rounded}ms`
  const seconds = Math.floor(rounded / 1000)
  if (seconds >= 60) return t('{m}分{s}秒', { m: Math.floor(seconds / 60), s: seconds % 60 })
  const millis = rounded % 1000
  return millis > 0 ? t('{s}秒{ms}ms', { s: seconds, ms: millis }) : t('{n}秒', { n: seconds })
}

/** ： no has （null / 0 / old ）「-」， not fmtDuration 0ms */
function fmtFirstResponse(ms: unknown): string {
  const value = Number(ms)
  if (!Number.isFinite(value) || value <= 0) return '-'
  return fmtDuration(value)
}

/** （tok/s） （ and SPEED_TIP 1 ， when 1 ） */
const SPEED_TIP = t('生成速度：输出 token ÷ 流式段用时（总用时 − 首响）；流式段不足总用时一半时按总用时计')

/**
 * （tok/s）： token ÷ use when （total use when − ）。
 *
 * and requests-page tokensPerSecond （ that is 1 ， this is 
 * ，， when 1 ）。， not by total use when 
 * ： ≥ 500ms（response duration ≈ ，）、 ≥ total use when 
 * 1 （ up token to after 1 down ，
 * is 「 down 」 not is 「」， can to 800+ tok/s ）。
 * / total use when as 0（failure token 、 old ） to empty —— use this 1 。
 */
function fmtTokensPerSecond(row: RequestRow): string {
  const tokens = Number(row.completionTokens) || 0
  const duration = Number(row.durationMs) || 0
  if (tokens <= 0 || duration <= 0) return ''
  const first = Number(row.firstResponseMs)
  const streamMs = Number.isFinite(first) && first > 0 && first < duration ? duration - first : 0
  const streaming = streamMs >= 500 && streamMs * 2 >= duration
  const rate = tokens / ((streaming ? streamMs : duration) / 1000)
  return `${rate >= 100 ? Math.round(rate) : rate.toFixed(1)} tok/s`
}

/** ： and to units.js formatTokens（ and / 1 ，
 * in 「 / 」 and 「k / M」 in that 1 ）； not in when */
function fmtTokens(value: unknown): string {
  return shared().wbUnits?.formatTokens?.(value) ?? (Number(value) || 0).toLocaleString('zh-CN')
}

/** in 「 already use when 」：seconds， not 1 seconds 1 seconds（） */
function fmtElapsed(ts: RequestRow['ts']): string {
  const elapsed = Math.max(0, Date.now() - (Number(ts) || 0))
  return t('{n}秒', { n: Math.max(1, Math.floor(elapsed / 1000)) })
}

/** success and after RequestEntry::is_success for ：2xx and no has error need */
function isOk(entry: RequestRow): boolean {
  const status = Number(entry.status) || 0
  return status >= 200 && status < 300 && !entry.error
}

/** in ：status=0 and no has error need （status=0 need is old failure） */
function isRunning(entry: RequestRow | null | undefined): boolean {
  return (Number(entry?.status) || 0) === 0 && !entry?.error
}

/** ： after label → before providers directory → id（） */
function providerName(entry: ProviderFields): string {
  const id = String(entry.provider ?? '').trim()
  const label = String(entry.providerLabel ?? '').trim()
  return label || (id ? shared().wbProviders?.labelOf?.(id) || id : '')
}

/** for （row / debug / raw response ； not ） */
function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

/** this request has no has （ old / before failure is null）：「 by 」 is */
function hasRoute(row: RequestRow | null | undefined): row is RequestRow & { route: RouteDecision } {
  return isPlainObject(row?.route)
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** use ， and 1 。 */
function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <TableRow className={NO_HOVER}>
      <TableHead scope='row' className={`w-[76px] whitespace-nowrap text-left text-[12.5px] font-semibold text-muted-foreground ${GRID_CELL}`}>
        {label}
      </TableHead>
      <TableCell className={GRID_CELL}>{children}</TableCell>
    </TableRow>
  )
}

/**
 * ： and statusCell —— in to `request-phase.js`（：
 * in / etc response / response in / in ， no has when 「 in 」， after 1 current
 * when ），2xx need when to title failure in response。（isRunning /
 * isOk） is up that ， and 。
 */
function StatusCell({ row }: { row: RequestRow }) {
  if (isRunning(row)) {
    const phase = shared().wbRequestPhase
    // no when before （ and for request-phase optional 1 
    // ）： is need ， when this 1 not this empty。 use has 
    // .req-live-dot（1.2s + reduced-motion ）， BadgeDot is 
    if (!phase?.badgeHtml) {
      return (
        <Badge variant='info' shape='tag' title={t('请求正在转发中，用时列显示的是已用时')}>
          <StatusDot pulse className='size-[5px] bg-current' />
          {t('进行中')}
        </Badge>
      )
    }
    const elapsed = phase.elapsedLineHtml?.(row) ?? ''
    return (
      <span className='inline-flex flex-wrap items-center gap-1.5'>
        {/* and when by request-phase.js HTML（ 1 need in 
             1 ， that module header）， this 、 not */}
        <HtmlContent html={phase.badgeHtml(row)} render={<span />} />
        {elapsed ? (
          <HtmlContent className='text-[11.5px] text-muted-foreground' html={elapsed} render={<span />} />
        ) : null}
      </span>
    )
  }
  const status = Number(row.status) || 0
  const ok = isOk(row)
  // 2xx error need （request in responsefailure） will as success， to title 
  const title = !ok && status >= 200 && status < 300 ? t('HTTP {status}，但响应体阶段出错', { status }) : ''
  return (
    <Badge variant={ok ? 'success' : 'destructive'} shape='tag' title={title || undefined}>
      {status || t('失败')}
    </Badge>
  )
}

/**
 * ： and modelCell —— down / up and not when （⬆️ up 
 * to / ⬇️ down request ），otherwise `model` 。 etc ：
 * `( etc )` after （ up = etc ， down = client etc ；empty = none etc ）。
 */
function ModelCell({ row }: { row: RequestRow }) {
  const client = String(row.clientModel ?? '').trim()
  const upstream = String(row.upstreamModel ?? '').trim()
  const shown = String(row.model ?? '').trim()
  const clientLevel = String(row.clientReasoning ?? '').trim()
  const upstreamLevel = String(row.upstreamReasoning ?? '').trim()
  const tag = (name: string, level: string) => (level ? `${name}(${level})` : name)
  if (!client || !upstream || upstream.toLowerCase() === client.toLowerCase()) {
    return <>{tag(shown, upstreamLevel || clientLevel) || '—'}</>
  }
  const upText = tag(upstream, upstreamLevel)
  const downText = tag(client, clientLevel)
  return (
    <span className='flex flex-col gap-0.5'>
      <span title={t('转发到上游的模型：{model}', { model: upText })}>⬆️ {upText}</span>
      <span className='text-muted-foreground' title={t('下游请求的模型：{model}', { model: downText })}>⬇️ {downText}</span>
    </span>
  )
}

/** 1 ： has error failure、 has status success、none is */
function attemptResult(item: AttemptDetail): React.ReactNode {
  const status = Number(item?.status)
  const hasStatus = item?.status !== null && item?.status !== undefined && Number.isFinite(status)
  const error = item?.error ? String(item.error) : ''
  if (error) {
    const text = hasStatus
      ? t('失败（{status}）：{error}', { status: String(status), error })
      : t('失败：{error}', { error })
    return <span className='break-words text-destructive'>{text}</span>
  }
  if (hasStatus) return <span className='text-success'>{t('成功（{status}）', { status: String(status) })}</span>
  return <span className='text-muted-foreground'>{t('无结果记录')}</span>
}

/**
 * ： up 1 （ / / / / ）。
 * after AttemptDetail； attempts when （， 1 ），
 * Reason and title（ 1 not down ， not this 1 ）。
 */
function AttemptsTable({ row }: { row: RequestRow }) {
  const details = Array.isArray(row.attemptDetails) ? row.attemptDetails : []
  if (!details.length) return <>-</>
  const attempts = Number(row.attempts) || 1
  return (
    <Table className='text-[11.5px]'>
      <TableHeader>
        <TableRow className={NO_HOVER}>
          {[t('轮次'), t('提供商'), t('账号'), t('结果'), t('体积'), t('内部重试'), t('提示')].map(label => (
            <TableHead key={label} className={ATTEMPT_HEAD}>{label}</TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {details.map((item, index) => {
          const retries = Array.isArray(item?.retries) ? item.retries : []
          const retryTitle = retries.map(retry => {
            const delayMs = Number(retry?.delayMs)
            const delay = Number.isFinite(delayMs) && delayMs > 0
              ? (delayMs >= 1000
                  ? t('，{n}秒后重试', { n: Math.round(delayMs / 1000) })
                  : t('，{n}毫秒后重试', { n: Math.round(delayMs) }))
              : ''
            return `${String(retry?.reason || t('未知原因'))}${delay}`
          }).join('\n')
          const notice = String(item?.notice ?? '').trim()
          // ： up that by request（413 / PARSE_REQUEST_DATA_EXCEPTION），
          const bodyRaw = Number(item?.bodyBytes)
          const bodyText = Number.isFinite(bodyRaw) && bodyRaw > 0
            ? bodyRaw >= 1024 * 1024
              ? `${(bodyRaw / 1024 / 1024).toFixed(1)} MB`
              : bodyRaw >= 1024
                ? `${Math.round(bodyRaw / 1024)} KB`
                : `${bodyRaw} B`
            : '—'
          const bodyTitle = Number.isFinite(bodyRaw) && bodyRaw > 0
            ? t('实际发给上游 {bytes} 字节', { bytes: bodyRaw.toLocaleString() })
            : undefined
          return (
            <TableRow key={index} className={NO_HOVER}>
              <TableCell className={ATTEMPT_CELL}>{index + 1}</TableCell>
              <TableCell className={ATTEMPT_CELL}>
                {providerName(item) || (item?.provider ? String(item.provider) : t('未知'))}
              </TableCell>
              <TableCell className={ATTEMPT_CELL}>{item?.account ? String(item.account) : '—'}</TableCell>
              <TableCell className={ATTEMPT_CELL}>{attemptResult(item)}</TableCell>
              <TableCell className={`${ATTEMPT_CELL} whitespace-nowrap`} title={bodyTitle}>
                {bodyText}
              </TableCell>
              <TableCell className={`${ATTEMPT_CELL} whitespace-nowrap`} title={retryTitle || undefined}>
                {retries.length ? t('↻ {n} 次', { n: retries.length }) : '-'}
              </TableCell>
              <TableCell className={ATTEMPT_CELL}>{notice || '—'}</TableCell>
            </TableRow>
          )
        })}
        {details.length < attempts ? (
          <TableRow className={NO_HOVER}>
            <TableCell colSpan={7} className={`${ATTEMPT_CELL} text-muted-foreground`}>
              {t('另有 {missing} 次尝试未记录明细（只保留最早的 {kept} 条）', {
                missing: attempts - details.length, kept: details.length,
              })}
            </TableCell>
          </TableRow>
        ) : null}
      </TableBody>
    </Table>
  )
}
/**
 * ② ： by 。 this request when ， by when 、Why
 * is up 。 is （`route` ）， and 「」 ——
 * that 「、 no 」， this 「 when Why this 」。
 *
 * old is 1 （ ① after 1 ）， after can 
 * 、 not use in 76px 1 —— this ** not compact**，
 * and by request 1 （） 1 。
 * no has （ old / before failure） when not ， is hasRoute。
 */
export function RequestRoutePane({ row }: { row: RequestRow & { route: RouteDecision } }) {
  return (
    <div className='flex flex-col items-start gap-3'>
      <RouteReplayFacts route={row.route}
        accountId={String(row.accountId || '')} accountName={String(row.accountName || '')} />
      <Button variant='outline' size='sm' onClick={() => {
        void window.wbRoutingPanel?.openReplay(row as unknown as ReplayRow)
      }}>{t('在路由页查看')}</Button>
    </div>
  )
}

/* ─── ①：request（ row）────── */

export function RequestDetailPane({ row }: { row: RequestRow | null }) {
  // in before new ： for by id not to 。 row，
  // no has can —— Reason use ， 1 empty
  if (!row) return <Empty text={t('列表数据已刷新，请重试（关闭后重新打开这条详情）')} />
  const account = String(row.accountName ?? '').trim()
  const speed = fmtTokensPerSecond(row)
  return (
    // in table up （ to up ）， error / success not will 
    <Table className='text-[12.5px] text-subtle [&_tr:last-child>td]:border-b-0 [&_tr:last-child>th]:border-b-0'>
      <TableBody>
        <Field label={t('时间')}>{fmtTime(row.ts)}</Field>
        <Field label={t('状态')}><StatusCell row={row} /></Field>
        <Field label={t('耗时')}>
          {/* in already use when ： no ， not （ and ） */}
          {isRunning(row) ? (
            <>
              {fmtElapsed(row.ts)}
              <span className='text-[11.5px] text-muted-foreground'>{t('（请求仍在转发中）')}</span>
            </>
          ) : (
            t('{duration} / 首帧 {first}', {
              duration: fmtDuration(row.durationMs), first: fmtFirstResponse(row.firstResponseMs),
            })
          )}
        </Field>
        <Field label={t('重试次数')}>{String(Number(row.attempts) || 1)}</Field>
        <Field label='API Key'>{requestApiKeyLabel(row, t)}</Field>
        <Field label={t('来源客户端')}><ClientIdentificationDetail entry={row} api={clientIdentificationApi} t={t} /></Field>
        <Field label={t('模型')}><ModelCell row={row} /></Field>
        <Field label={t('提供商')}>
          {providerName(row) || '—'}
          {account ? <span className='text-[11.5px] text-muted-foreground'>{t('（账号：{account}）', { account })}</span> : null}
        </Field>
        <Field label={t('令牌')}>
          {t('输入 {input} · 输出 {output} · 总计 {total} · 缓存读 {cache}', {
            input: fmtTokens(row.promptTokens),
            output: fmtTokens(row.completionTokens),
            total: fmtTokens(row.totalTokens),
            cache: fmtTokens(row.cacheReadTokens),
          })}
        </Field>
        <Field label={t('速度')}><span title={SPEED_TIP}>{isRunning(row) ? '—' : speed || '—'}</span></Field>
        {/* request** by need 1 **： is this 1 （ is 、 not ），
             not as 1 「」 —— that has when etc no has */}
        {row.isTest ? (
          <Field label={t('来源')}>
            <Badge variant='brand' shape='tag'
              title={t('面板内发起的模型调用，消耗额度并记录 Key，不计入生产报表')}>
              {requestClientLabel(row, t)}
            </Badge>
          </Field>
        ) : null}
        <Field label={t('错误')}>
          {row.error ? (
            <div className='flex items-center gap-2'>
              <span className='max-h-40 min-w-0 flex-1 overflow-auto whitespace-pre-wrap text-destructive [overflow-wrap:anywhere]'>{String(row.error)}</span>
              <Button variant='outline' size='xs' data-copy={String(row.error)}>{t('复制')}</Button>
            </div>
          ) : '—'}
        </Field>
        <Field label={t('尝试明细')}><AttemptsTable row={row} /></Field>
        <Field label={t('敏感词')}>
          {/* in （ and 「」 1 ， this and ） */}
          {Array.isArray(row.sensitiveHits) && row.sensitiveHits.length ? (
            <span className='inline-flex flex-wrap gap-1.5'>
              {row.sensitiveHits.map((hit, index) => (
                <Badge key={index} variant='sensitive' shape='tag'>
                  {String(hit?.word ?? '')} × {String(Number(hit?.count) || 0)}
                </Badge>
              ))}
            </span>
          ) : '—'}
        </Field>
      </TableBody>
    </Table>
  )
}
/* ─── ───────────────────────────────── */

type RequestDetailProps = {
  id: string
  /** current 1 for （ can can ）；request */
  row: RequestRow | null
  /** open （ seq ）； to old response */
  token: number
  onClose: () => void
}

function RequestDetail({ id, row, token, onClose }: RequestDetailProps) {
  const [active, setActive] = React.useState<TabKey>(DEFAULT_TAB)
  const [raw, setRaw] = React.useState<RawState>(null)
  const [debug, setDebug] = React.useState<DebugState>(null)
  const [hint, setHint] = React.useState('—')
  const [terminate, setTerminate] = React.useState<TerminateState>('idle')

  /** ：await use （ or ）， response 1 */
  const aliveRef = React.useRef(true)
  /** （DialogBody）； when in this / pane */
  const bodyRef = React.useRef<HTMLDivElement | null>(null)
  const scrollTops = React.useRef<Record<TabKey, number>>({ detail: 0, route: 0, preview: 0, raw: 0 })

  /** to response： not is new 1 （ or already ） */
  const isStale = React.useCallback(() => !aliveRef.current || token !== seq, [token])

  /** down （ ②）：404 and failure as 1 empty（Reason for use is 1 ） */
  async function loadRaw() {
    try {
      const response = await api().getStatsRequestRaw(id)
      if (isStale()) return
      // ： is {id, requestBody, responseBody, truncated}； not is for or 
      // empty（ after for 「 has none」returnempty）「 no has 」
      const body = isPlainObject(response) ? response : null
      const hasText = String(body?.requestBody ?? '').trim() !== ''
        || String(body?.responseBody ?? '').trim() !== ''
      setRaw(body && hasText ? { body, truncated: !!body.truncated } : 'gone')
    } catch (error) {
      if (isStale()) return
      setRaw('gone')
      console.warn('读取原始正文失败（预览对话显示空态）:', errorMessage(error))
    }
  }

  /** up （ ③）： has ； no has （404 etc ） */
  async function loadDebug() {
    try {
      const response = await api().getDebugTraffic(id)
      if (isStale()) return
      const data: DebugTraffic = isPlainObject(response) ? response : {}
      // empty and 404 is 1 ： no has can ， not ， not hint
      if (!hasDebugTraffic(data)) {
        setDebug('gone')
        return
      }
      setDebug({ data })
      setHint(data.truncated
        ? t('这条报文由旧版本截断保存，缺失内容无法恢复')
        : t('内容按原样保存，请求头中的凭据字段已替换为 [redacted]'))
    } catch (error) {
      if (isStale()) return
      setDebug('gone')
      // no is Reason（ after 404 already ）， hint 1 
      setHint(t('在设置 → 通用里开启「调试模式」后，新发生的请求才会保存报文'))
      console.warn('读取调试报文失败（原始报文标签隐藏）:', errorMessage(error))
    }
  }

  // and （ old open() is ）
  React.useEffect(() => {
    aliveRef.current = true
    if (id) { void loadRaw(); void loadDebug() }
    else { setRaw('gone'); setDebug('gone') }
    return () => { aliveRef.current = false }
    // in when 1 ： is 「 when 、」，id not will in 
  }, [])

  /**
   * current pane ： hidden（display:none），pane DOM 
   * —— will 、scrollTop ， when 、
   * when use layout effect （ before ， not to ）。
   */
  React.useLayoutEffect(() => {
    const body = bodyRef.current
    if (body) body.scrollTop = scrollTops.current[active]
  }, [active])

  /** ：， not request、 not pane */
  function switchTab(next: TabKey) {
    if (next === 'raw' && !isPlainObject(debug)) return // 标签还没出现，点不到就不用防
    if (next === 'route' && !hasRoute(row)) return
    if (next === active) return
    const body = bodyRef.current
    if (body) scrollTops.current[active] = body.scrollTop
    setActive(next)
  }

  /**
   * current this in request： → after 。
   *
   * ** not update**： （ up 、 「request already 」） by 
   * ， will that 1 ； this success by （） and in hint
   * down will 。failure（404 = already / up ；error）
   * Reason and by 。
   */
  async function handleTerminate() {
    if (!id || !isRunning(row)) return
    const ask = shared().wbConfirm?.ask
    if (!ask) return
    // （ confirm in Tauri WebView not ， app.js）
    const ok = await ask({
      title: t('终止请求'),
      html: t('确定终止这条正在转发的请求？<br>上游连接会立即断开，明细将记为「请求已被手动终止」。'),
      okText: t('终止'),
      okClass: 'danger',
    })
    if (!ok) return
    // is use for ** this id** ， / not ， this not token；
    // not already （React for after setState is empty）
    setTerminate('pending')
    try {
      await api().terminateStatsRequest(id)
      if (!aliveRef.current) return
      setTerminate('accepted')
      setHint(t('已受理终止：上游连接已断开，列表稍后刷新为「请求已被手动终止」'))
    } catch (error) {
      if (!aliveRef.current) return
      setTerminate('idle')
      setHint(t('终止失败：{error}', { error: errorMessage(error) }))
    }
  }

  const showRaw = isPlainObject(debug)
  const showRoute = hasRoute(row)
  // in 「 / by is available」 when use ， 
  // effect 
  const tabOptions = React.useMemo(
    () => TAB_OPTIONS.filter(tab => (tab.value !== 'raw' || showRaw) && (tab.value !== 'route' || showRoute)),
    [showRaw, showRoute]
  )

  return (
    <Dialog open onOpenChange={next => { if (!next) onClose() }}>
      {/* old is .modal-wide（min(920px, 100%)）， this DialogContent default */}
      <DialogContent className='w-[min(920px,calc(100vw-48px))]'>
        <DialogHeader>
          <DialogTitle>{t('请求详情')}</DialogTitle>
        </DialogHeader>
        <DialogBody ref={bodyRef} className='gap-3'>
          <SegmentedControl options={tabOptions} value={active} onValueChange={switchTab}
            aria-label={t('详情内容')} className='self-start' />
          {/* pane （ use hidden ， not ）： when pane DOM and 
              ， by up layout effect 。pane not role="tabpanel" ——
               is radiogroup （ not is tablist）， tabpanel is */}
          <section hidden={active !== 'detail'}><RequestDetailPane row={row} /></section>
          {showRoute ? (
            <section hidden={active !== 'route'}><RequestRoutePane row={row} /></section>
          ) : null}
          <section hidden={active !== 'preview'}><PreviewPane state={raw} /></section>
          {showRaw ? (
            <section hidden={active !== 'raw'}>
              <RawPane data={(debug as { data: DebugTraffic }).data} />
            </section>
          ) : null}
        </DialogBody>
        <DialogFooter>
          <span className='min-w-0 text-[11.5px] text-muted-foreground'>{hint}</span>
          <div className='mr-auto' />
          {/* in in when （ old use hidden ， this ） */}
          {isRunning(row) ? (
            <Button variant='destructive' disabled={terminate !== 'idle'}
              onClick={() => { void handleTerminate() }} title={t('断开上游连接，明细记为「请求已被手动终止」')}>
              {t('终止请求')}
            </Button>
          ) : null}
          <Button variant='outline' onClick={onClose}>{t('关闭')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
/* ─── ： and old window.wbRequestDetail 1 ─── */

let root: ReturnType<typeof createRoot> | null = null
let host: HTMLElement | null = null

function unmountDetail() {
  if (root) {
    root.unmount()
    root = null
  }
  if (host) {
    host.remove()
    host = null
  }
}

/**
 * ：（Esc / / ✕ to this ）。 when seq 1 —— after 
 * response not this 。
 */
function closeDetail() {
  seq += 1
  unmountDetail()
}

/**
 * request （「」 when ）。`row` is use from current 1 by id
 * for （ can can as null）；request， for / by id 
 * —— row 1 。
 */
function openDetail(id: unknown, row?: unknown) {
  const key = String(id ?? '').trim()
  if (!key && !isPlainObject(row)) return
  unmountDetail() // 重复打开先拆掉上一份（旧实现同样是 close() 打头）
  const token = ++seq
  host = document.createElement('div')
  document.body.append(host)
  root = createRoot(host)
  root.render(
    <RequestDetail
      id={key}
      row={isPlainObject(row) ? (row as RequestRow) : null}
      token={token}
      onClose={closeDetail}
    />
  )
}

declare global {
  interface Window {
    /** request （ ui/request-detail.js；open(id, row) / close() and 1 ） */
    wbRequestDetail?: { open(id: unknown, row?: unknown): void; close(): void }
  }
}

window.wbRequestDetail = { open: openDetail, close: closeDetail }
