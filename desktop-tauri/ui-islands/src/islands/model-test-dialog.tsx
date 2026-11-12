/**
 * Agent2API · 「」 —— React 。
 *
 * in **** that 「」（models-page.tsx `case 'act'`）。 this 1 ：
 * 1 is 、 not ， from that need 1 ， already 
 * is ； by 「」 —— that is 1 1 parameter， not is 1 。
 *
 * ── ：parameter in down 、 in up ──────────────────────────────────
 * 1. **parameter**（ Dialog）： + / use / for / /
 * etc / / 。default「request」—— use 「」、、
 * OpenAI Chat Completions、config、 etc ；
 * 2. ****（ Dialog， by ）： in / 1 1 / failure in this 1 。
 *
 * is ** in DialogContent children ** （ add-account-modal.tsx ）：
 * React is Base UI 「」 ， when Esc and by not ；
 * must is ， Portal will in Popup after 、 in up 。 need to 
 * `overlayForceRender` —— need down （parameter），「empty this 1 」 is 。
 * = to parameter（parameter not ）；parameter 。
 *
 * ── ：， by ──────────────────────────
 * 1 and 1 `POST /api/models/test`（ 1 1 request）， after this 1 and this 1 
 * **** `UpstreamService::forward` —— token new 、、、 etc 、
 * request and to ， and 1 。** will **， and 
 * 「」request（statisticsexcluding， after `is_test`）—— this 1 in parameter
 * （must）， its its ： / / / / ⓘ
 * （tip-q + Tooltip），「empty then …」 placeholder， and ID 1 not 。
 * for and to prototype/model-test-lite.html。
 *
 * ── in ： no has abort ， 1 id ──────────────────────
 * （`invoke('api_request')`）** no has abort**： 1 already in use not 。 id by 
 * ** before **（`test_id`）request 1 up ， in when use 1 id has 「request」
 * （`terminateStatsRequest` → `POST /api/stats/requests/terminate?id=`），
 * up 。 for ：、request that 1 、 in use ， is this 1 id。
 * in after to parameter（`slots` empty、seq in ）， not in up 。
 *
 * ── 「 not 」─────────────────────────────────────
 * · ** not **： and in request can can ， will 「 no 、
 * Why not 」；
 * · ** not **： by 、 not ， that is ， not is 。
 */

import * as React from 'react'
import {
  Badge, Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle,
  Label, MultiSelect, Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  Switch, Textarea, Tooltip, TooltipContent, TooltipTrigger,
} from '@ui'
import {
  DEFAULT_PROVIDER_ID, byPriorityOrder, displayNameOf, isRateLimited, positionMap, providerFeatures,
} from './accounts-domain'
import {
  POOL_VALUE_PREFIX, poolItemLabel, type AccountRecord, type PoolItem,
} from './accounts-shared'
import * as customSource from './models-custom-source'
import type { ManageModel } from './models-custom-source'
import { levels as reasoningLevels } from './models-reasoning'
import {
  bindingsOf, errorMessage, getSnapshot, levelOf, modelRowOf, providerLabelOf, toast,
} from './models-panel-state'
import { t } from '../i18n'
import { ModelApiKeyField } from '../../../ui-shared/model-api-key-field'
import { modelApiKeyAllows } from '../../../ui-shared/model-api-key'
import { useModelApiKeys } from '../../../ui-shared/use-model-api-keys'
import { useRememberedModelApiKey } from '../../../ui-shared/model-api-key-memory'
import { gatewayBase } from './assistant-state'
import { ModelTestResult, modelTestOk as isOk, type ModelTestSlot as Slot, type TestModelResponse } from './model-test-result'

/* ─── ─────────────────────────────────── */

/** that 1 ： `(provider, id)` （ CapabilityContext ） */
export type ModelTestTarget = { provider: string; id: string }

/** 1 request（ and after `api::model_test` for ，snake_case） */
type TestModelRequest = {
  provider: string
  model: string
  account_id?: string
  prompt?: string
  system_prompt?: string
  /** etc ；empty = not request（ up etc ） */
  reasoning?: string
  stream?: boolean
  /** use for （chat_completions / responses / anthropic） */
  protocol?: string
  /** when （empty = config，direct = ，pool:<id> = ） */
  proxy?: string
  /** before id（request that 1 id， is in use ） */
  test_id?: string
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：workbuddyDesktop /
 * wbApp is use ， declare 1 will not 1 TS2717 —— and when 
 * 。file use to that （ its models-panel-state / accounts-domain
 * already has that ）。
 */
type WindowBridge = {
  workbuddyDesktop?: {
    testModel(payload: TestModelRequest, apiKey: string): Promise<TestModelResponse | null | undefined>
    getModelApiKeys?(): Promise<unknown>
    /** 「request」： by id （ in ，module header） */
    terminateStatsRequest?(id: string): Promise<unknown>
    /** （「」 ） */
    getProxyPool?(): Promise<{ items?: PoolItem[] } | null | undefined>
  }
  wbApp?: {
    getState?: () => { accounts?: { accounts?: AccountRecord[] } } | null | undefined
  }
  wbProviders?: {
    PROTOCOL_OPTIONS?: Array<{ value: string; label: string }>
  }
}

function bridge(): WindowBridge {
  return window as unknown as WindowBridge
}

/* ─── ─────────────────────────────────── */

/**
 * use empty when after use default（ after `api::model_test::DEFAULT_TEST_PROMPT`）。
 * must is 1 ：「」 not is 1 has —— need is **request**，、
 * up ，「 not 」 this 。
 */
const DEFAULT_PROMPT = '你好'

/** limit（ and after `MAX_PROMPT_CHARS` ； after will ， this 1 ） */
const MAX_PROMPT_CHARS = 4000

/** for （ and providers.js PROTOCOL_OPTIONS and after TestProtocol 1 ） */
const FALLBACK_PROTOCOLS: Array<{ value: string; label: string }> = [
  { value: 'chat_completions', label: 'OpenAI - Chat Completions' },
  { value: 'responses', label: 'OpenAI - Responses' },
  { value: 'anthropic', label: 'Anthropic - Messages' },
]

/** when in down （ and after parse_proxy_override direct for ） */
const PROXY_DIRECT = 'direct'

/** 「」（ and 1 ； source is empty， ManageModel） */
const SOURCE_LABEL: Record<string, string> = {
  remote: t('远程目录'),
  builtin: t('内置清单'),
  manual: t('手动登记'),
}

/* ─── ─────────────────────────────── */

/** （； and is 1 ） */
function allAccounts(): AccountRecord[] {
  return bridge().wbApp?.getState?.()?.accounts?.accounts || []
}

/**
 * this available ： = enabled + has （ after ， and 「」 down ），
 * use 1 `byPriorityOrder`（ 1 ， this 1 ）。
 */
function usableAccounts(provider: string): AccountRecord[] {
  return allAccounts()
    .filter(account => (account.provider || DEFAULT_PROVIDER_ID) === provider
      && account.available !== false && account.enabled !== false)
    .sort(byPriorityOrder)
}

/**
 * ： and 「」 down （models-fetch-modal.tsx accountLabel） 1 ——
 * （Qoder / AutoClaw ） use ， its use 。
 */
function accountLabel(account: AccountRecord): string {
  const email = String(account.email || '').trim()
  if (email && providerFeatures(account.provider).emailAsName) return email
  return displayNameOf(account) || email || t('未命名账号')
}

/** id： `crypto.randomUUID`（WebView2 and localhost is up down ）， to v4 */
function newTestId(): string {
  const uuid = globalThis.crypto?.randomUUID?.()
  if (uuid) return uuid
  const hex = (count: number): string =>
    Array.from({ length: count }, () => Math.floor(Math.random() * 16).toString(16)).join('')
  return `${hex(8)}-${hex(4)}-4${hex(3)}-a${hex(3)}-${hex(12)}`
}

/**
 * 「」 that by ： this ** no has available**（enabled + ）→ 1 not ，
 * this is 1 by 。returnempty = can ；otherwisereturn need in by title up Reason。
 *
 * ── **enabled not **（，）────────────────
 * this by use is 「、 need not need enabled」—— is ，
 * etc by 1 use in 。 after as to 
 * （`ForwardRequest::ignore_model_gate`）： that ，
 * 「 already in in 」 ； default not ，
 * 、 up etc not —— is this in this up ****。
 */
export function testBlockReason(provider: string, model: ManageModel): string {
  if (model.kind === 'image') {
    return t('图片生成模型请通过 /v1/images/generations 调用，暂不支持对话测试')
  }
  if (!usableAccounts(provider).length) {
    return t('该提供商没有可用账号（要在账号页启用一个、且凭证完整），一行都发不出去')
  }
  return ''
}

/* ─── ─────────────────────────────────── */

/** 「i + 1 」（ use and up its 1 ）。box is ；
 * plain is parameter 1 —— after 1 「must」 （ will ） */
function NoteBlock({ children, plain = false }: { children: React.ReactNode; plain?: boolean }) {
  return (
    <p className={plain
      ? 'flex items-center gap-2 text-xs leading-[1.6] text-subtle'
      : 'flex gap-2 rounded-md border border-border bg-surface-2 px-3 py-2.5 text-xs leading-[1.65] text-subtle'}>
      <span aria-hidden='true'
        className={plain
          ? 'size-4 flex-none rounded-full border border-border-strong text-center text-[10px] leading-[14px]'
          : 'mt-px size-4 flex-none rounded-full border border-border-strong text-center text-[10px] leading-[14px]'}>i</span>
      <span>{children}</span>
    </p>
  )
}

/** 「 already etc 1.4s」 seconds： in when when （ after not empty） */
function useClock(active: boolean): number {
  const [now, setNow] = React.useState(() => Date.now())
  React.useEffect(() => {
    if (!active) return
    setNow(Date.now())
    const timer = window.setInterval(() => setNow(Date.now()), 400)
    return () => window.clearInterval(timer)
  }, [active])
  return now
}

/* ─── ───────────────────────────────── */

export function ModelTestDialog({ target, onClose }: { target: ModelTestTarget; onClose: () => void }) {
  const provider = target.provider
  const model = modelRowOf(provider, target.id)

  const usable = usableAccounts(provider)
  const positions = positionMap(allAccounts())

  const [systemPrompt, setSystemPrompt] = React.useState('')
  const [prompt, setPrompt] = React.useState(DEFAULT_PROMPT)
  const [protocol, setProtocol] = React.useState('chat_completions')
  const [proxy, setProxy] = React.useState('')
  const [pool, setPool] = React.useState<PoolItem[] | null>(null)
  const [poolError, setPoolError] = React.useState('')
  const [reasoning, setReasoning] = React.useState('')
  const [stream, setStream] = React.useState(true)
  const keyState = useModelApiKeys(async () => {
    const load = bridge().workbuddyDesktop?.getModelApiKeys
    if (!load) throw new Error(t('当前环境不支持读取 API Key'))
    return load()
  })
  const { value: apiKeyId, ready: keySelectionReady, select: setApiKeyId } = useRememberedModelApiKey('model-test', gatewayBase)
  const selectedKey = keyState.keys.find(key => key.id === apiKeyId)
  const keyAllowed = Boolean(selectedKey && modelApiKeyAllows(selectedKey, target.id, provider))
  const selectedKeyRef = React.useRef(selectedKey)
  selectedKeyRef.current = selectedKey
  /**
   * default** before that can **（ 1 ）。
   * for current in after ： will 1 429， use need is 「 this not 」。
   */
  const [picked, setPicked] = React.useState<string[]>(() => {
    const first = usable.find(account => !isRateLimited(account, target.id)) || usable[0]
    return first ? [first.id] : []
  })
  const [slots, setSlots] = React.useState<Slot[]>([])
  const [folds, setFolds] = React.useState<ReadonlySet<string>>(() => new Set())

  /** ： in / will in （ to not new 1 ） */
  const seq = React.useRef(0)
  const slotsRef = React.useRef<Slot[]>([])
  slotsRef.current = slots

  const running = slots.some(slot => slot.state === 'running')
  const now = useClock(running)

  /** when 1 （「」 down use ） */
  React.useEffect(() => {
    let alive = true
    void (async () => {
      try {
        const getPool = bridge().workbuddyDesktop?.getProxyPool
        if (typeof getPool !== 'function') throw new Error(t('当前环境不支持代理池'))
        const data = await getPool()
        if (!alive) return
        if (!data || typeof data !== 'object' || !Array.isArray(data.items)) {
          throw new Error(t('代理池响应异常'))
        }
        setPool(data.items)
        setPoolError('')
      } catch (error) {
        if (!alive) return
        setPool([])
        setPoolError(errorMessage(error))
      }
    })()
    return () => { alive = false }
  }, [])

  /** in in request：seq （）+ ， after empty to parameter */
  const abort = React.useCallback((quiet = false) => {
    const inflight = slotsRef.current.filter(slot => slot.state === 'running')
    seq.current += 1
    for (const slot of inflight) {
      void bridge().workbuddyDesktop?.terminateStatsRequest?.(slot.testId)?.catch(() => {})
    }
    setSlots([])
    if (inflight.length && !quiet) toast(t('已中止在途测试（{n} 个账号）', { n: inflight.length }))
  }, [])

  // （ use parameter / in directory new after ） when 1 ： already in request not this 
  // up 。 not （ already no ）， quiet。
  React.useEffect(() => () => { abort(true) }, [abort])

  /** 1 ： by testId for ； （ in / ） */
  async function send(slot: Slot, runId: number): Promise<void> {
    const body: TestModelRequest = {
      provider,
      model: target.id,
      account_id: slot.accountId,
      prompt: prompt.trim(),
      system_prompt: systemPrompt.trim(),
      reasoning,
      stream,
      protocol,
      ...(proxy ? { proxy } : {}),
      test_id: slot.testId,
    }
    const settle = (patch: Partial<Slot>): void => {
      if (seq.current !== runId) return
      setSlots(previous => previous.map(item => (item.testId === slot.testId ? { ...item, ...patch } : item)))
    }
    try {
      const key = selectedKeyRef.current
      if (!key) throw new Error(t('请先选择一把 API Key'))
      const result = await bridge().workbuddyDesktop?.testModel(body, key.key)
      if (!result) {
        settle({ state: 'done', transport: t('桥接调用返回空（本机网关没有响应这次测试）') })
        return
      }
      settle({ state: 'done', result })
    } catch (error) {
      settle({ state: 'done', transport: errorMessage(error) })
    }
  }

  /** 1 ： and （ 1 1 request， not ） */
  function run(): void {
    if (!keySelectionReady || !keyState.ready || !selectedKey) {
      toast(t('请先选择一把 API Key'), 'err')
      return
    }
    if (!keyAllowed) {
      toast(t('所选 API Key 无权调用此模型或提供商'), 'err')
      return
    }
    const chosen = usable.filter(account => picked.includes(account.id))
    if (!chosen.length) {
      toast(t('至少选一个测试账号'), 'err')
      return
    }
    const runId = seq.current + 1
    seq.current = runId
    const startedAt = Date.now()
    const next: Slot[] = chosen.map(account => ({
      accountId: account.id,
      label: accountLabel(account),
      position: positions.get(account.id)?.position || 0,
      testId: newTestId(),
      startedAt,
      state: 'running',
    }))
    setFolds(new Set())
    setSlots(next)
    for (const slot of next) void send(slot, runId)
  }

  /** ： when = in （parameter， down can ） */
  function closeResults(): void {
    if (running) abort()
    else setSlots([])
  }

  /** that ID （ use ） */
  const modelTag = <span className='ml-2 font-mono text-[12px] font-normal text-subtle'>{target.id}</span>

  /** not in （ ， or directory new after up not ）—— and 「 can 」 1 */
  if (!model) {
    return (
      <Dialog open onOpenChange={next => { if (!next) onClose() }}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('测试模型')}</DialogTitle></DialogHeader>
          <DialogBody>
            <p className='text-sm leading-[1.7] text-subtle'>
              {t('这一行已不在当前清单里（模型被移除、或目录刷新后上游不再提供它）。关闭后刷新列表再试。')}
            </p>
          </DialogBody>
          <DialogFooter>
            <div className='mr-auto' />
            <Button variant='outline' onClick={onClose}>{t('关闭')}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    )
  }

  // when etc 「 not that by 」（ model_rules::reasoning 4 ），
  // that is this 1 default —— and default chip up is 1 。
  // this enabled when not （ by not ）， etc not and 。
  const defaultClosed = bindingsOf(model).find(binding => binding.isDefault)?.enabled === false
  const boundLevel = levelOf(model.id, model.id, provider)
  const aliases = bindingsOf(model)
    .filter(binding => !binding.isDefault)
    .map(binding => binding.alias)
  // 「」（off / none） not to ： in **** up is 「 not 」，**request**
  // is 1 up not —— CatPaw resolve_effort for 400。 need 「 not 」，
  // 「」 this 1 （ up no etc when not ）
  //
  // by ** this 1 ** （ `models-reasoning.levels` ）：
  // is 「 this 1 × this 1 」 ， to 1 this not ，
  // 400 is 、 not is —— that is this by no use use 。
  const levelOptions = reasoningLevels(getSnapshot().data, model)
    .filter(level => level !== 'off' && level !== 'none')
  const followLabel = defaultClosed
    ? t('跟随映射（该行未启用，本次不注入）')
    : boundLevel ? t('跟随映射（当前 {level}）', { level: boundLevel }) : t('跟随映射（未绑定等级）')
  /** ⓘ ： no has / etc ， 1 「」 —— */
  const targetTip = [
    t('以模型本名直发'),
    aliases.length
      ? t('另有 {n} 条别名映射（{aliases}），别名不参与本次', { n: aliases.length, aliases: aliases.join(t('、')) })
      : '',
    defaultClosed
      ? t('该行当前未启用：测试照常按本名直发（「先测通、再决定要不要启用」正是这颗按钮的用法），但本行的思考等级绑定不参与本次，生产路由也要等绑定打开后才会放行')
      : boundLevel
        ? t('映射上绑定的思考等级是 {level}，「跟随映射」按它注入', { level: boundLevel })
        : t('映射上未绑定思考等级，「跟随映射」等于这次不注入'),
  ].filter(Boolean).join(t('；')) + t('。')

  const protocols = bridge().wbProviders?.PROTOCOL_OPTIONS || FALLBACK_PROTOCOLS
  const protocolLabel = protocols.find(item => item.value === protocol)?.label || protocol
  const proxyItems: Array<{ value: string; label: string; disabled?: boolean }> = [
    { value: '', label: t('跟随账号配置') },
    { value: PROXY_DIRECT, label: t('直连（不使用代理）') },
    // already use ： will （failure）， 
    // not is use need that —— and 1 
    ...(pool ?? []).map(item => ({
      value: `${POOL_VALUE_PREFIX}${item.id}`,
      label: poolItemLabel(item),
      disabled: item.enabled === false,
    })),
  ]
  if (pool === null) {
    proxyItems.push({ value: '__hint_pool__', label: t('正在读取代理列表…'), disabled: true })
  } else if (!pool.length) {
    proxyItems.push({
      value: '__hint_pool__',
      label: poolError ? t('代理列表读取失败') : t('还没有代理（去「网络代理」页添加）'),
      disabled: true,
    })
  }
  const proxyLabel = proxyItems.find(item => item.value === proxy)?.label || t('跟随账号配置')

  const accountOptions = usable.map(account => ({
    value: account.id,
    // 「 in 」 by ** this **（ is by ： 1 can can for A 、 for B ）
    label: `#${positions.get(account.id)?.position || 0} ${accountLabel(account)}`
      + (isRateLimited(account, target.id) ? t(' · 限额中') : ''),
  }))

  function slotBlock(slot: Slot): React.ReactNode {
    const foldKey = `think-${slot.testId}`
    return <ModelTestResult key={slot.testId} slot={slot} model={target.id} now={now} expanded={folds.has(foldKey)}
      onToggle={() => setFolds(previous => {
        const next = new Set(previous)
        if (next.has(foldKey)) next.delete(foldKey)
        else next.add(foldKey)
        return next
      })} />
  }

  /**
   * ： in already ****（ has in etc ）—— is 1 
   * 1 ， to after 1 etc etc ； after 1 ， 1 success / failure。
   */
  function results(): React.ReactNode {
    if (running) {
      const done = slots.length - slots.filter(slot => slot.state === 'running').length
      return (
        <>
          {/* when this is down that ， */}
          {slots.length > 1 ? (
            <p className='text-xs text-subtle'>
              {t('正在测 ')}<b className='text-foreground'>{slots.length}</b>{t(' 个账号')}
              {done ? <>{t('，已完成 ')}<b className='text-foreground'>{done}</b></> : null}{t('。')}
            </p>
          ) : null}
          {slots.map(slot => slotBlock(slot))}
        </>
      )
    }
    const okCount = slots.filter(isOk).length
    return (
      <>
        {/* up ： and already 1 ； 0 up ，Note to */}
        {slots.length > 1 ? (
          <p className='text-xs text-subtle'>
            <b className={okCount ? 'text-success' : 'text-foreground'}>{okCount}</b>{t(' 成功 ·')}
            <b className={slots.length - okCount ? 'text-destructive' : 'text-foreground'}> {slots.length - okCount}</b>{t(' 失败')}
          </p>
        ) : null}
        {slots.map(slot => slotBlock(slot))}
        {okCount === 0 ? (
          <NoteBlock>
            {t('两边原因不一样时各自处理。如果')}<b className='text-foreground'>{t('同一条错误出现在所有账号上')}</b>{t('，通常就不是账号问题了：先确认这个模型还在上游目录里（「获取模型」刷一次），再看默认绑定指向的上游模型对不对。')}
          </NoteBlock>
        ) : null}
      </>
    )
  }

  return (
    // parameter： open（ as true）。 is 「 when 、」， 1 by onClose 
    // （Esc / / ✕ by Base UI to onOpenChange）。
    <Dialog open onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('测试模型')}{modelTag}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          {/* ： is 1 （ and 「 can 」 1 use ）； ⓘ， not 1 */}
          <div className='flex items-center gap-2 rounded-md border border-border bg-surface-inset px-3 py-2.5'>
            <span className='min-w-0 flex-1 truncate font-mono text-[12.5px] font-semibold text-primary-fg'
              title={model.id}>{model.id}</span>
            <Badge shape='tag' variant='brand'>{providerLabelOf(provider)}</Badge>
            <Badge shape='tag' variant='outline'>
              {customSource.isCustom(provider) ? t('自定义家') : (SOURCE_LABEL[model.source] || t('来源未知'))}
            </Badge>
            <Tooltip>
              <TooltipTrigger render={<span className='tip-q' tabIndex={0} aria-label={t('本次测试的发送口径')} />}>?</TooltipTrigger>
              <TooltipContent>{targetTip}</TooltipContent>
            </Tooltip>
          </div>

          <ModelApiKeyField {...keyState} value={apiKeyId} onChange={setApiKeyId} id='model-test-api-key'
            disabled={!keySelectionReady} rememberSelection
            onRefresh={() => { void keyState.refresh().catch(() => {}) }} t={t} />
          {selectedKey && !keyAllowed ? <p role='status' className='text-xs text-destructive'>{t('所选 API Key 无权调用此模型或提供商')}</p> : null}

          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='model-test-system'>{t('系统提示词')}</Label>
            <Textarea id='model-test-system' rows={2} maxLength={MAX_PROMPT_CHARS}
              placeholder={t('留空则不携带；设置页的提示词模式照常生效')}
              value={systemPrompt}
              onChange={event => setSystemPrompt(event.currentTarget.value)} />
          </div>

          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='model-test-prompt'>{t('用户提示词')}</Label>
            <Textarea id='model-test-prompt' rows={2} maxLength={MAX_PROMPT_CHARS}
              placeholder={t('留空用默认问候「你好」')}
              value={prompt}
              onChange={event => setPrompt(event.currentTarget.value)} />
          </div>

          <div className='grid grid-cols-2 gap-4'>
            <div className='flex flex-col gap-1.5'>
              <div className='flex items-center gap-1.5'>
                <Label htmlFor='model-test-protocol'>{t('对话协议')}</Label>
                <Tooltip>
                  <TooltipTrigger render={<span className='tip-q' tabIndex={0} aria-label={t('对话协议的说明')} />}>?</TooltipTrigger>
                  <TooltipContent>{t('按所选网关对话协议组装请求并校验响应转换（Chat Completions / Responses / Anthropic Messages）。')}</TooltipContent>
                </Tooltip>
              </div>
              <Select value={protocol} onValueChange={next => setProtocol(String(next))}>
                <SelectTrigger id='model-test-protocol' className='w-full'>
                  <SelectValue className='min-w-0 truncate'>{protocolLabel}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {protocols.map(item => (
                    <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className='flex flex-col gap-1.5'>
              <div className='flex items-center gap-1.5'>
                <Label htmlFor='model-test-proxy'>{t('测试节点')}</Label>
                <Tooltip>
                  <TooltipTrigger render={<span className='tip-q' tabIndex={0} aria-label={t('测试节点的说明')} />}>?</TooltipTrigger>
                  <TooltipContent>{t('仅本次测试临时生效，不修改账号保存的代理配置。可选「网络代理」页里的节点或强制直连。')}</TooltipContent>
                </Tooltip>
              </div>
              <Select value={proxy} onValueChange={next => setProxy(String(next))}>
                <SelectTrigger id='model-test-proxy' className='w-full'>
                  <SelectValue className='min-w-0 truncate'>{proxyLabel}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {proxyItems.map(item => (
                    <SelectItem key={item.value} value={item.value} disabled={item.disabled}>{item.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className='grid grid-cols-2 gap-4'>
            <div className='flex flex-col gap-1.5'>
              <Label htmlFor='model-test-reasoning'>{t('思考等级')}</Label>
              <Select value={reasoning} onValueChange={next => setReasoning(String(next))}>
                <SelectTrigger id='model-test-reasoning' className='w-full'>
                  <SelectValue>{reasoning ? t('{level}（本次指定）', { level: reasoning }) : followLabel}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value=''>{followLabel}</SelectItem>
                  {levelOptions.map(level => (
                    <SelectItem key={level} value={level}>{t('{level}（本次指定）', { level })}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className='flex flex-col gap-1.5'>
              <div className='flex items-center gap-1.5'>
                <Label htmlFor='model-test-stream'>{t('流式请求')}</Label>
                <Tooltip>
                  <TooltipTrigger render={<span className='tip-q' tabIndex={0} aria-label={t('流式请求的说明')} />}>?</TooltipTrigger>
                  <TooltipContent>{t('默认开，与真实请求一致：可同时验证 SSE 链路与首字延迟。')}</TooltipContent>
                </Tooltip>
              </div>
              <Switch id='model-test-stream' checked={stream}
                onCheckedChange={value => setStream(Boolean(value))} />
            </div>
          </div>

          <div className='flex flex-col gap-1.5'>
            <div className='flex items-center gap-2'>
              <Label htmlFor='model-test-accounts'>{t('测试账号')}</Label>
              <Tooltip>
                <TooltipTrigger render={<span className='tip-q' tabIndex={0} aria-label={t('测试账号的说明')} />}>?</TooltipTrigger>
                <TooltipContent>{t('勾几个就并行测几次；默认选队列里最靠前的一个。')}</TooltipContent>
              </Tooltip>
              <Button variant='ghost' size='xs' className='ml-auto'
                disabled={!usable.length || picked.length === usable.length}
                onClick={() => setPicked(usable.map(account => account.id))}>
                {t('全选（{n}）', { n: usable.length })}
              </Button>
            </div>
            <MultiSelect id='model-test-accounts' value={picked} onValueChange={setPicked}
              options={accountOptions} placeholder={t('选一个账号（至少一个）')}
              searchPlaceholder={t('搜索账号…')}
              emptyHint={t('这家没有可用账号（要在账号页启用一个、且凭证完整）')} />
          </div>

          <NoteBlock plain>{t('真实链路 · 消耗少量额度 · 日志带「测试」标记，不计报表')}</NoteBlock>
        </DialogBody>
        <DialogFooter className='justify-end'>
          <Button variant='outline' onClick={onClose}>{t('关闭')}</Button>
          <Button variant='default' disabled={!picked.length || !usable.length || !keySelectionReady || !keyState.ready || !keyAllowed} onClick={run}>{t('开始测试')}</Button>
        </DialogFooter>

        {/* ── ： in parameter up 1 （ and overlayForceRender by module header）── */}
        <Dialog open={slots.length > 0} onOpenChange={next => { if (!next) closeResults() }}>
          <DialogContent overlayForceRender className='w-[min(680px,calc(100vw-48px))]'>
            <DialogHeader>
              <DialogTitle>{running ? t('测试中…') : t('测试结果')}{modelTag}</DialogTitle>
            </DialogHeader>
            <DialogBody>{results()}</DialogBody>
            <DialogFooter className='justify-end'>
              {running ? (
                <>
                  <span className='mr-auto text-xs text-subtle'>{t('关闭即中止')}</span>
                  <Button variant='outline' onClick={() => abort()}>{t('中止测试')}</Button>
                  <Button variant='default' disabled>{t('测试中…')}</Button>
                </>
              ) : (
                <>
                  <Button variant='outline' onClick={closeResults}>{t('关闭')}</Button>
                  <Button variant='default' onClick={run}>{t('再测一次')}</Button>
                </>
              )}
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </DialogContent>
    </Dialog>
  )
}