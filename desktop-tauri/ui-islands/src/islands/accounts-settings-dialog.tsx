/** 单账号设置弹窗与提供商共用连接设置。 */

import * as React from 'react'
import {
  Badge, BadgeDot, Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader,
  DialogTitle, Input, NavList, NavItem, Sidebar, Switch,
  Field, FieldDescription, FieldLabel,
} from '@ui'
import '../styles/accounts-settings-dialog.css'
import { t } from '../i18n'
import {
  ACCOUNT_CONCURRENCY_MAX as CONC_MAX, errorMessage, esc, shared, toast,
  type AccountRecord, type LimiterGroup,
} from './accounts-shared'
import {
  displayNameOf, providerFeatures, providerOf, supportsPlanChannel, supportsUsage,
  isOpencodeZenProvider, zcodePlanOf,
} from './accounts-domain'
import { PlanChannelField } from './accounts-zcode-plan'
import {
  readUsageChanges, UsageSettingsSection, usageDraftOf, type UsageDraft,
} from './accounts-dialog-usage'
import {
  BalanceQuerySection, balanceFieldConfigured, balanceQueryConfigured, balanceQueryDraftOf,
  balanceSpecOf, readBalanceQueryChanges, type BalanceFieldKey,
} from './balance-query'
import { limiterDraftOf, LimiterSection, readLimiterChanges } from './accounts-dialog-limiter'
import { allAccounts, findAccount, syncBalancesSnapshot } from './accounts-data'
import { refreshSavedAccounts, updateAccountConfirmed } from './accounts-update'
import { draftOfProxy, ProxyForm, readProxyDraft, type ProxyDraft, type ProxyPayload } from './accounts-proxy-form'
import { UserAgentInput, useUserAgentPresets, userAgentEditable } from './add-account-user-agent'
import {
  AccountSettingsIcon, BalanceTokenField, CustomCredentialFields, ProviderSection, SettingGroup,
  type ProviderSettings,
} from './accounts-settings-fields'

type AccountLike = AccountRecord
type PaneId = 'basic' | 'credential' | 'query' | 'limiter' | 'network' | 'provider'

function peersOf(account: AccountLike, all: AccountLike[]): AccountLike[] {
  const provider = providerOf(account)
  return all.filter(item => item.id !== account.id && providerOf(item) === provider)
}

function labelOf(account: AccountLike | null | undefined): string {
  return displayNameOf(account) || account?.id || ''
}

export function AccountSettingsDialog({ id, onClose, initialAccount, initialPane = 'basic', routeContext }: {
  id: string; onClose: () => void
  initialAccount?: AccountRecord
  initialPane?: PaneId
  routeContext?: React.ReactNode
}) {
  const [account, setAccount] = React.useState(() => initialAccount || findAccount(id))
  const [enabled, setEnabled] = React.useState(account?.enabled !== false)
  const [allowRouteSkip, setAllowRouteSkip] = React.useState(account?.allowRouteSkip !== false)
  const [name, setName] = React.useState(account?.name || '')
  /** 账号级自定义 User-Agent 草稿（空串 = 用该家自己的默认值） */
  const [userAgent, setUserAgent] = React.useState(account?.userAgent || '')
  const uaPresets = useUserAgentPresets()
  // 会话式家由插件自己拼出站头，网关覆盖不到 —— 整段不渲染
  const uaEditable = userAgentEditable(providerOf(account || { id }))
  const [proxyDraft, setProxyDraft] = React.useState<ProxyDraft>(() => draftOfProxy(account?.proxy))
  const [balanceToken, setBalanceToken] = React.useState('')
  // ZCode 的「使用哪个套餐」草稿（非 ZCode 账号恒为空串，那一段也不渲染）
  const [planChannel, setPlanChannel] = React.useState(() => zcodePlanOf(account))
  const [busy, setBusy] = React.useState(false)
  const [status, setStatus] = React.useState<React.ReactNode>('')
  const [provider, setProvider] = React.useState<ProviderSettings | null>(null)
  const [providerName, setProviderName] = React.useState('')
  const [providerProtocol, setProviderProtocol] = React.useState('chat_completions')
  const [providerBaseUrl, setProviderBaseUrl] = React.useState('')
  /** 客户端形态伪装草稿（'' = 不伪装；'opencode' = 按官方 CLI 形状补齐请求） */
  const [providerEmulation, setProviderEmulation] = React.useState('')
  /** 自定义账号的凭证草稿：新 Key（留空 = 不改，走「清除」按钮）与「无需鉴权」勾选 */
  const [apiKeyDraft, setApiKeyDraft] = React.useState('')
  const [noAuthDraft, setNoAuthDraft] = React.useState(account?.noAuth === true)
  // 「查询设置」段的草稿（自动查询间隔）：只在真的有余额概念的家渲染，草稿仍
  // 无条件初始化 —— 与 planChannel 同一形态，省一个条件分支
  const [usageDraft, setUsageDraft] = React.useState<UsageDraft>(() => usageDraftOf(account))
  // 「查询设置」段里**余额方案要的凭证**（只有登记了方案的自定义家才有字段）：
  // 草稿同样无条件初始化（没有方案时是一份空草稿，那一段也不渲染）
  const [balanceQueryDraft, setBalanceQueryDraft] = React.useState<Record<string, string>>(
    () => balanceQueryDraftOf(account?.balanceQuery),
  )
  // 「限制器」段的草稿（余额 / Token 规则列表）：与查询设置同一 gating
  const [limiterDraft, setLimiterDraft] = React.useState<LimiterGroup[]>(() => limiterDraftOf(account))
  // 分组导航的当前页（弹窗按账号 id 挂载，切账号自然回到「基本」）
  const [pane, setPane] = React.useState<PaneId>(initialPane)
  // 并发上限草稿（原列表 ⋯ 菜单的 conc-dialog 迁入「基本」分组）：0~999，0 = 不限
  const [concDraft, setConcDraft] = React.useState(() => String(Number(account?.maxConcurrent) || 0))
  const [confirming, setConfirming] = React.useState(false)
  const actionLock = React.useRef(false)
  const paneScroll = React.useRef<HTMLDivElement | null>(null)
  const panePositions = React.useRef<Partial<Record<PaneId, number>>>({})
  const initial = React.useRef({
    enabled, allowRouteSkip, name, userAgent, proxyDraft, planChannel, usageDraft,
    balanceQueryDraft, limiterDraft, concDraft, noAuthDraft,
  })
  const credentialLabel = providerOf(account).startsWith('custom-') ? t('Credentials') : t('Credential & plan')
  const paneInfo: Record<PaneId, { title: string; description: string; icon: string }> = {
    basic: { title: t('Basic information'), description: t('Set the account name, enabled state, and forwarding behavior.'), icon: 'sliders' },
    credential: { title: credentialLabel, description: t('Manage this account\'s credentials and available plans.'), icon: 'key' },
    query: { title: t('Query settings'), description: t('Configure balance query credentials, the automatic refresh interval, and retries after failure.'), icon: 'refresh' },
    limiter: { title: t('Limiter'), description: t('Protect this account\'s quota with explicit rules.'), icon: 'shield' },
    network: { title: t('Network'), description: t('Choose how this account connects to the upstream service.'), icon: 'proxies' },
    provider: { title: t('Connection settings'), description: t('Manage the provider\'s protocol, address, and compatibility options in one place.'), icon: 'agent' },
  }
  const currentPane = paneInfo[pane]
  React.useLayoutEffect(() => {
    if (paneScroll.current) paneScroll.current.scrollTop = panePositions.current[pane] || 0
  }, [pane])

  function selectPane(next: PaneId): void {
    if (paneScroll.current) panePositions.current[pane] = paneScroll.current.scrollTop
    setPane(next)
  }

  // 「提供商」那一段：目录里查得到才算自定义家（id 前缀只说明「长得像」，而记录本身
  // 才带着协议 / Base URL 的现值 —— 三个字段要拿它预填）。
  // 依赖只取 id：弹窗是按 id 挂载的（父组件给了 key），账号数据刷新不该重置用户的编辑
  React.useEffect(() => {
    let alive = true
    setProvider(null)
    const account = initialAccount || findAccount(id)
    if (!account) return () => { alive = false }
    void Promise.resolve(shared().wbCustomProvidersUi?.find?.(providerOf(account))).then(found => {
      if (!alive || !found) return
      setProvider(found)
      setProviderName(found.name || '')
      setProviderProtocol(found.protocol || 'chat_completions')
      setProviderBaseUrl(found.baseUrl || '')
      setProviderEmulation(typeof found.clientEmulation === 'string' ? found.clientEmulation : '')
    }).catch(() => { /* 目录不可用：退化成「不是自定义家」，不注入那一段 */ })
    return () => { alive = false }
  }, [id])

  if (!account) {
    return (
      <Dialog open onOpenChange={next => { if (!next) onClose() }}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('Account settings')}</DialogTitle></DialogHeader>
          <DialogBody><p className='detail'>{t('Account not found; refresh and try again.')}</p></DialogBody>
          <DialogFooter><div className='mr-auto' /><Button variant='outline' onClick={onClose}>{t('Close')}</Button></DialogFooter>
        </DialogContent>
      </Dialog>
    )
  }

  // 账号不存在时只给一块提示（列表可能刚被另一端删了）。用局部常量接住窄化后的账号：
  // 下面几个 async 函数声明会被提升，TS 不会把「早退之后的窄化」带进它们体内
  const target: AccountRecord = account
  const peers = peersOf(target, allAccounts())
  const isCatpaw = providerOf(target) === 'catpaw'
  // 自定义账号：凭证段（API Key / 无需鉴权）只对它渲染。判据用 provider 前缀 ——
  // 与后端 `custom_account_provider` 一致（提供商被删后残留的账号同样该能改凭证）
  const isCustomAccount = providerOf(target).startsWith('custom-')
  // 「轮询」档的可见性：只有 OpenCode Zen 免费档（按 IP 限流）。判据用**已读到的
  // 提供商记录**（上面那个 effect 拉的）而不是目录缓存 —— 这一段的上下文里就有它，
  // 且与「提供商」段编辑的是同一份现值（改了 emulation / baseUrl 未保存时，
  // 档位仍按盘上的值给，保存后才变 —— 与后端闸门读盘上记录同一口径）。
  // 记录本来就是这个档时**恒显示**（哪怕判据此刻为假：目录还没读到 / 提供商被
  // 改过）：否则用户看不到自己这条账号现在是什么状态；后端只挡「真的改成轮询」
  // 的保存，原样发回来不会 400。
  const allowRotate = isOpencodeZenProvider(provider, target.baseUrl) || proxyDraft.mode === 'rotate'
  // 余额方案那一段（「查询设置」里的查询凭证）：方案与「已配置」标记都由账号的
  // 公开形态回答 —— `scheme` 是这一家的方案 id，`has*` 是密字段的存在标记
  // （值永不回显，见后端 `store_view::balance_query_public`）
  const balanceSpec = balanceSpecOf(target.balanceQuery?.scheme)
  const balanceConfigured: Record<string, boolean> = {}
  for (const field of balanceSpec?.fields ?? []) {
    balanceConfigured[field.key] = balanceFieldConfigured(target.balanceQuery, field)
  }
  // 「要填但还没填全」时给一句中性提示（不是错误：余额列会显示「未配置」，
  // 用户在那一列看到的就是这句话的后果）
  const balanceNeedsCredentials = Boolean(balanceSpec && balanceSpec.fields.length)
    && !balanceQueryConfigured(target.balanceQuery)
  const providerCount = peers.length + 1
  const before = initial.current
  const same = (left: unknown, right: unknown): boolean => JSON.stringify(left) === JSON.stringify(right)
  const dirtyPanes: Record<PaneId, boolean> = {
    basic: enabled !== before.enabled || allowRouteSkip !== before.allowRouteSkip
      || (name.trim() || before.name) !== before.name || userAgent.trim() !== before.userAgent
      || concDraft !== before.concDraft,
    credential: Boolean(balanceToken.trim() || apiKeyDraft.trim())
      || noAuthDraft !== before.noAuthDraft || planChannel !== before.planChannel,
    query: !same(usageDraft, before.usageDraft) || !same(balanceQueryDraft, before.balanceQueryDraft),
    limiter: !same(limiterDraft, before.limiterDraft),
    network: !same(proxyDraft, before.proxyDraft),
    provider: Boolean(provider && (
      providerName.trim() !== (provider.name || '') || providerProtocol !== (provider.protocol || 'chat_completions')
      || providerBaseUrl.trim() !== (provider.baseUrl || '') || providerEmulation !== (provider.clientEmulation || '')
    )),
  }
  const dirtyCount = Object.values(dirtyPanes).filter(Boolean).length
  const interactionBusy = busy || confirming

  async function askConfirmation(title: string, text: string, okText: string, danger = false): Promise<boolean> {
    if (actionLock.current) return false
    actionLock.current = true
    setConfirming(true)
    try {
      const ask = shared().wbConfirm?.ask
      if (!ask) throw new Error(t('The confirmation dialog is unavailable. Reopen the page.'))
      return await ask({ title, html: esc(text).replace(/\n/g, '<br>'), okText,
        okClass: danger ? 'danger' : 'primary', bodyClass: danger ? 'danger-zone' : '' })
    } catch (error) {
      toast(errorMessage(error), 'err')
      return false
    } finally {
      actionLock.current = false
      setConfirming(false)
    }
  }

  async function close(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (dirtyCount && !await askConfirmation(t('Discard unsaved changes?'),
      t('Unsaved changes in all sections will be discarded. Previously confirmed standalone clearing operations are unaffected.'), t('Discard changes'))) return
    onClose()
  }

  /** 清除余额凭证（立即落库，不走保存按钮 —— 它是一个独立的撤销动作） */
  async function clearBalanceToken(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (!await askConfirmation(t('Clear balance query credentials?'),
      t('This takes effect immediately after confirmation, independently of the Save button. The login credentials used for forwarding can usually be reused.'), t('Clear'), true)) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { balanceToken: null })
      setAccount(saved)
      setBalanceToken('')
      toast(t('✅ Balance query credential cleared'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('Clear failed: {error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * 清除 API Key（与余额凭证同一形制：立即落库的独立撤销动作）。
   * 清掉之后账号变成「未配置凭证」，转发时会被跳过 —— 所以提示语点明这一点，
   * 而列表上那枚「未配置凭证」徽章会立刻出现，用户看得到后果。
   */
  async function clearApiKey(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (!await askConfirmation(t('Clear the saved API Key?'),
      t('This takes effect immediately after confirmation, independently of the Save button. Without credentials or No authentication, this account will be skipped when forwarding.'), t('Clear'), true)) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { apiKey: null })
      setAccount(saved)
      setApiKeyDraft('')
      toast(t('✅ API Key cleared (the account now has no credential; forwarding will skip it)'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('Clear failed: {error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * 清除这个账号单独保存的上游地址，改回跟随提供商的 Base URL
   * （与清除 API Key 同一形制：立即落库的独立撤销动作）。
   *
   * 早先版本新建提供商时会把当时的地址抄进首个账号，而账号上的地址优先于
   * 提供商 —— 于是改提供商地址后请求仍打旧地址，界面上还看不到那份旧值
   * （issue #455）。这个按钮是存量账号的修复入口；新建的提供商不再产生
   * 这种覆盖项，改提供商地址时后端也会顺手清掉等于旧地址的那份。
   */
  async function clearAccountBaseUrl(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    const nextBase = providerBaseUrl.trim()
    if (!await askConfirmation(t('Follow the provider again?'),
      t('This account will use the Base URL currently saved on the provider: {url}. Takes effect immediately and is not controlled by the Save button below.',
        { url: nextBase || '—' }), t('Follow provider'))) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { baseUrl: null })
      setAccount(saved)
      toast(t('✅ This account follows the provider\'s Base URL again'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('Operation failed: {message}', { message: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * 清除一个已配置的余额查询凭证字段（自定义家：NewAPI 的访问令牌 / 商汤的
   * 密码 / 智谱的控制台令牌）。
   *
   * 与清除 API Key 同一形制：**立即落库的独立撤销动作**（不进保存按钮那一批）
   * —— 密字段的值不回显，输入框留空是「不修改」，清除只能靠这个显式动作
   * （传 null 给后端，语义见 `normalize_balance_query_patch`）。
   * 清掉之后余额列会显示「未配置」，用户看得到后果。
   */
  async function clearBalanceQueryField(key: BalanceFieldKey): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (!await askConfirmation(t('Clear query credentials?'),
      t('This takes effect immediately after confirmation. Balance queries may show Not configured. Canceling account settings will not restore cleared credentials.'), t('Clear'), true)) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { balanceQuery: { [key]: null } })
      setAccount(saved)
      setBalanceQueryDraft(prev => ({ ...prev, [key]: '' }))
      toast(t('✅ Balance credentials cleared'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('Clear failed: {error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * 读「凭证」那一段的改动：没这一段（内置家）或什么都没改 → null。
   *
   * 三个字段的语义与后端逐条对齐（`update_custom_credentials`）：
   *   · 输入框填了 → 写新 Key（后端会顺手摘掉「无需鉴权」）；
   *   · 勾选态变了 → 写 noAuth（开=清 key，后端归一）；
   *   · 输入框留空且勾选态没变 → 什么都不发（留空是「不修改」，清除走按钮）。
   */
  function readCredentialPatch(): Record<string, unknown> | null {
    if (!isCustomAccount) return null
    const key = apiKeyDraft.trim()
    const patch: Record<string, unknown> = {}
    if (noAuthDraft) {
      if (target.noAuth !== true) patch.noAuth = true
    } else if (key) patch.apiKey = key
    else if (target.noAuth === true) patch.noAuth = false
    return Object.keys(patch).length ? patch : null
  }

  /** 读「提供商」那一段的改动：没这一段（内置家）或字段都没动 → null（不提交） */
  function readProviderPatch(): { error: string } | { id: string; name: string; protocol: string; baseUrl: string; clientEmulation: string } | null {
    if (!provider) return null
    const nextName = providerName.trim()
    const nextBase = providerBaseUrl.trim()
    // 空名称 / 空 Base URL 后端会 400，而那时**账号已经存下了** —— 用户看到「保存失败」
    // 却发现账号的改动生效了。所以在账号落库之前先验一遍
    if (!nextName) return { error: t('Enter the provider name') }
    if (!nextBase) return { error: t('Enter the provider\'s Base URL') }
    try {
      if (!['http:', 'https:'].includes(new URL(nextBase).protocol)) throw new Error('invalid protocol')
    } catch {
      return { error: t('Enter a valid HTTP or HTTPS URL') }
    }
    const beforeEmulation = typeof provider.clientEmulation === 'string' ? provider.clientEmulation : ''
    if (
      provider.name === nextName && provider.protocol === providerProtocol
      && provider.baseUrl === nextBase && beforeEmulation === providerEmulation
    ) return null
    return {
      id: provider.id, name: nextName, protocol: providerProtocol,
      baseUrl: nextBase, clientEmulation: providerEmulation,
    }
  }

  async function save(): Promise<void> {
    if (interactionBusy || actionLock.current || !dirtyCount) return
    setStatus('')
    let proxy: ProxyPayload
    try {
      proxy = readProxyDraft(proxyDraft)
    } catch (error) {
      selectPane('network')
      setStatus(<span className='text-destructive'>{errorMessage(error)}</span>)
      toast(errorMessage(error), 'err')
      return
    }
    const providerPatch = readProviderPatch()
    if (providerPatch && 'error' in providerPatch) {
      selectPane('provider')
      setStatus(<span className='text-destructive'>{providerPatch.error}</span>)
      return
    }
    // 查询设置段（自动查询间隔）：与凭证段同一层，跟着这次保存一起落库；
    // 校验不过先在前端挡住（后端 apply_patch 对同一规则也会 400）
    const usageChanges = supportsUsage(target) ? readUsageChanges(target, usageDraft) : null
    if (usageChanges && 'error' in usageChanges) {
      selectPane('query')
      setStatus(<span className='text-destructive'>{usageChanges.error}</span>)
      return
    }
    // 限制器段（余额 / Token 规则列表）：与查询设置同一层；编辑器保存时已校验过，
    // 这里只负责「真改了才带上」（对着现状保存不会凭空多一条变更日志，与后端一致）
    const limiterChanges = readLimiterChanges(target, limiterDraft)
    // 查询凭证段（自定义家的余额方案）：按方案读草稿 —— 密字段留空 = 不修改，
    // 非密字段原样提交（后端判无变化时不写日志）；这一家没有方案时恒 null
    const balanceQueryPatch = readBalanceQueryChanges(target.balanceQuery?.scheme, balanceQueryDraft)
    // 凭证段（自定义账号）：与代理 / 套餐同一层，跟着这次保存一起落库
    // （它走的是 PATCH /api/accounts 的自定义分支，见后端 `update_custom_credentials`）
    const credentialPatch = readCredentialPatch() || {}
    // 并发上限（「基本」分组）：本地先归一（number 输入挡不住手工脏值），后端
    // 400 只该是最后防线 —— 与原 conc-dialog 同一套 clamp（0~999，0 = 不限；
    // CodeArts 家 0 由上游解释成默认并发，见能力表 concurrencyDefault）
    const concRaw = Number(concDraft)
    if (!Number.isFinite(concRaw) || concRaw < 0) {
      selectPane('basic')
      setStatus(<span className='text-destructive'>{t('Concurrency limit must be an integer from 0 to {max}', { max: CONC_MAX })}</span>)
      toast(t('Concurrency limit must be an integer from 0 to {max}', { max: CONC_MAX }), 'err')
      return
    }
    const concNext = Math.min(CONC_MAX, Math.round(concRaw))
    const concCurrent = Number(target.maxConcurrent) || 0
    const concPatch: Record<string, unknown> = concNext !== concCurrent ? { maxConcurrent: concNext } : {}
    if (providerPatch && !await askConfirmation(t('Save shared provider settings?'),
      t('These connection settings are shared by {count} accounts under {name}. Changing the protocol or Base URL may interrupt requests in progress.',
        { name: provider?.name || provider?.id || '', count: providerCount }), t('Confirm save'))) return

    actionLock.current = true
    setBusy(true)
    try {
      // CatPaw 的余额凭证是**按需附加**的字段（别的家没有这一项，输入框也不存在）：
      // 空值 = 不修改（后端公开形态只给 hasBalanceToken 真假、不回显原值，所以这个
      // 输入框永远是空的；若把「空」解释成清除，用户每次保存设置都会把配好的凭证删掉）。
      // 清除走上面那个显式按钮。
      const balancePatch = isCatpaw && balanceToken.trim() ? { balanceToken: balanceToken.trim() } : {}
      // 套餐通道：只在**真的改了**时才带上（取值与后端 `zcode::normalize_plan`
      // 逐字一致）。带一个没变的键不会出错，但会让后端那句「套餐通道 → …」的
      // 变更提示在每次保存设置时都出现一次，看起来像刚改过
      const planPatch = supportsPlanChannel(target) && planChannel !== zcodePlanOf(target)
        ? { zcodePlan: planChannel }
        : {}
      // 顺序（priority）**不在这张表单里**：分组模式下它由拖拽整队写（见
      // accounts-data 的 reorderQueue）。设置弹窗只改「账号自身」的属性，
      // 一次「改备注名」不该顺手把顺序也提交一遍
      const nextName = name.trim() || target.name
      // 空串 = 清除自定义 UA（后端 `apply_patch` 把空串解释成「回到该家默认」）。
      // 只在真的改了时带上，避免每次保存都多一条「User-Agent → 默认」的变更日志。
      const nextUserAgent = userAgent.trim()
      const saved = await updateAccountConfirmed(id, {
        ...(nextName !== target.name ? { name: nextName } : {}),
        ...(nextUserAgent !== (target.userAgent || '').trim() ? { userAgent: nextUserAgent } : {}),
        enabled,
        allowRouteSkip,
        proxy,
        ...balancePatch,
        ...planPatch,
        ...credentialPatch,
        ...concPatch,
        ...(balanceQueryPatch ? { balanceQuery: balanceQueryPatch } : {}),
        ...(usageChanges && 'patch' in usageChanges ? usageChanges.patch : {}),
        ...(limiterChanges && 'patch' in limiterChanges ? limiterChanges.patch : {}),
      })
      if ((nextName !== target.name || target.nameCustom === true) && saved.nameCustom !== true) {
        throw new Error(t('The account update could not be confirmed. Refresh and check the settings.'))
      }
      setAccount(saved)
      // 提供商那一段排在账号之后（账号是本弹窗的主角，先落库）。它失败时账号已经存下了，
      // 所以留在弹窗里把那句话说清楚 —— 笼统报成「保存失败」会把两件事混成一件
      if (providerPatch) {
        try {
          const updateProvider = shared().wbCustomProvidersUi?.update
          if (!updateProvider) throw new Error(t('Account updates are unavailable. Reopen the page.'))
          await updateProvider(providerPatch)
        } catch (error) {
          selectPane('provider')
          setStatus(<span className='text-destructive'>{t('Account saved, but the provider was not updated: {error}', { error: errorMessage(error) })}</span>)
          try { await refreshSavedAccounts() }
          catch (refreshError) { toast(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(refreshError) }), 'err') }
          setBusy(false)
          return
        }
      }
      // 先关窗并反馈成功：改动已经落库，刷新只是让列表跟上，不该让用户对着「保存中…」
      // 再多等一次网络往返
      onClose()
      toast(t('✅ Account settings saved'))
      // 保存后必须主动刷新列表：以前只关窗不刷新，行上的代理 / 优先级仍是旧数据，
      // 要等 20 秒那一轮轮询才更新 —— 用户看到的就是「保存完十几秒才变」。
      // 单独兜一层错：刷新失败只影响本次界面同步（后续轮询会自愈），不能掉进下面的
      // catch 被报成「保存失败」（保存其实已经成功了）
      try {
        await refreshSavedAccounts()
        await syncBalancesSnapshot()
      } catch (error) {
        toast(t('The account was saved, but the list failed to refresh: {error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      // 后端校验失败（如优先级冲突）：留在弹窗里显示原因，方便直接改
      setStatus(<span className='text-destructive'>{errorMessage(error)}</span>)
      toast(t('Save failed: {error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  // 内置账号仅在有余额凭证或套餐通道时显示凭证分区。
  const concDefault = Number(providerFeatures(providerOf(target)).concurrencyDefault) || 0
  const hasCredentialPane = isCatpaw || isCustomAccount || supportsPlanChannel(target)
  const accountPanes: PaneId[] = ['basic', ...(hasCredentialPane ? ['credential' as const] : []),
    ...(supportsUsage(target) ? ['query' as const] : []), 'limiter', 'network']
  const providerLabel = provider?.name || shared().wbProviders?.labelOf?.(providerOf(target)) || providerOf(target)

  async function removeProvider(): Promise<void> {
    if (interactionBusy || actionLock.current || !provider) return
    actionLock.current = true
    setConfirming(true)
    try {
      const remove = shared().wbCustomProvidersUi?.remove
      if (!remove) throw new Error(t('Account updates are unavailable. Reopen the page.'))
      if (await remove(provider.id)) onClose()
    } catch (error) {
      toast(t('Delete failed: {error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setConfirming(false)
    }
  }

  async function openModels(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (dirtyCount && !await askConfirmation(t('Discard unsaved changes?'),
      t('Going to model management closes account settings and discards unsaved changes.'), t('Go to model management'))) return
    onClose()
    shared().wbApp?.showPage?.('gateway')
  }

  return (
    <Dialog open onOpenChange={(next, details) => {
      if (next) return
      details.cancel()
      void close()
    }}>
      <DialogContent className='acct-settings-dialog' showCloseButton={!interactionBusy}>
        <DialogHeader className='acct-dlg-header'>
          <div className='acct-dlg-avatar'><AccountSettingsIcon name={isCustomAccount ? 'agent' : 'accounts'} size={23} /></div>
          <div className='acct-dlg-identity'>
            <div className='acct-dlg-title-row'><DialogTitle>{t('Account settings')}</DialogTitle>
              {isCustomAccount ? <Badge variant='secondary' shape='tag'>{t('Custom API')}</Badge> : null}
            </div>
            <p><span>{labelOf(account)}</span><span className='acct-dlg-title-dot'>·</span>
              <span className={`acct-dlg-account-state${account.enabled === false ? ' off' : ''}`}>
                <BadgeDot />{account.enabled === false ? t('Disabled') : t('Enabled')}
              </span>
            </p>
          </div>
        </DialogHeader>
        <DialogBody className='acct-dlg-body'>
          <div className='acct-dlg'>
            <Sidebar className='acct-dlg-rail' aria-label={t('Account settings sections')}>
              <NavList className='acct-dlg-nav'>
                <div className='acct-dlg-nav-label'>{t('Current account')}</div>
                {accountPanes.map(id => <NavItem key={id} active={pane === id} title={paneInfo[id].title}
                  disabled={interactionBusy} className='shadow-none'
                  icon={<AccountSettingsIcon name={paneInfo[id].icon} />}
                  trailing={dirtyPanes[id] ? <span className='acct-dlg-dirty-dot' aria-label={t('There are unsaved changes')} /> : undefined}
                  onClick={() => selectPane(id)}>{paneInfo[id].title}</NavItem>)}
                {provider ? <>
                  <div className='acct-dlg-nav-divider' />
                  <div className='acct-dlg-nav-label'>{t('Provider')}<span>{t('Shared')}</span></div>
                  <NavItem active={pane === 'provider'} disabled={interactionBusy} title={paneInfo.provider.title}
                    icon={<AccountSettingsIcon name={paneInfo.provider.icon} />}
                    trailing={dirtyPanes.provider ? <span className='acct-dlg-dirty-dot' aria-label={t('There are unsaved changes')} /> : undefined}
                    onClick={() => selectPane('provider')}>{paneInfo.provider.title}</NavItem>
                </> : null}
              </NavList>
              {provider ? <div className='acct-dlg-rail-context'><strong><AccountSettingsIcon name='accounts' size={14} />{providerLabel}</strong>
                <p>{t('Connection settings shared by {count} accounts', { count: providerCount })}</p>
              </div> : null}
            </Sidebar>
            <div className='acct-dlg-main'>
              <div className='acct-dlg-pane-head'>
                <div><h3>{currentPane.title}</h3><p>{currentPane.description}</p></div>
                <span className={`acct-dlg-scope${pane === 'provider' ? ' shared' : ''}`}>
                  <AccountSettingsIcon name='accounts' size={13} />
                  {pane === 'provider' ? t('Shared by {count} accounts', { count: providerCount }) : t('This account only')}
                </span>
              </div>
              <div ref={paneScroll} className='acct-dlg-scroll' aria-busy={busy}>
                <div className='acct-dlg-pane' inert={interactionBusy ? true : undefined}>
                  {pane === 'basic' ? (
                    <>
                      <SettingGroup title={t('Account info')}>
                        <Field className='acct-dlg-field'>
                          <FieldLabel htmlFor='account-name-input'>{t('Label')}</FieldLabel>
                          <Input id='account-name-input' maxLength={100} placeholder={t('Account display name')}
                            value={name} onChange={event => setName(event.currentTarget.value)} />
                        </Field>
                        <div className='acct-dlg-readonly'><span>{t('Account ID')}</span><code>{target.id}</code></div>
                        <Field className='acct-dlg-toggle-field'>
                          <div><FieldLabel htmlFor='account-enabled'>{t('Enable account')}</FieldLabel>
                            <FieldDescription>{t('When disabled, this account is excluded from forwarding. Balance queries are unaffected.')}</FieldDescription>
                          </div>
                          <Switch id='account-enabled' checked={enabled} onCheckedChange={setEnabled} />
                        </Field>
                      </SettingGroup>
                      <SettingGroup title={t('Forwarding behavior')}>
                        <Field className='acct-dlg-toggle-field'>
                          <div><FieldLabel htmlFor='account-route-skip'>{t('Allow account switching after failure')}</FieldLabel>
                            <FieldDescription>{t('When off, forwarding failures on this account return the error without switching to another account. Disabled-account, missing-credential, and limiter protections still apply.')}</FieldDescription>
                          </div>
                          <Switch id='account-route-skip' checked={allowRouteSkip} onCheckedChange={setAllowRouteSkip} />
                        </Field>
                        <Field className='acct-dlg-concurrency'>
                          <div><FieldLabel htmlFor='account-conc-input'>{t('Concurrency limit')}</FieldLabel>
                            <FieldDescription>{t('When the limit is reached, requests are passed to other accounts.')}</FieldDescription>
                          </div>
                          <Input id='account-conc-input' type='number' min={0} max={CONC_MAX} step={1}
                            value={concDraft} onChange={event => setConcDraft(event.currentTarget.value)} />
                        </Field>
                      </SettingGroup>
                      <p className='detail acct-dlg-hint'>{concDefault > 0
                        ? t('0 = provider default {default}', { default: concDefault }) : t('0 = unlimited')}</p>
                      {uaEditable ? <SettingGroup title={t('Request identity')}>
                        <Field className='acct-dlg-field'>
                          <FieldLabel htmlFor='account-user-agent'>User-Agent</FieldLabel>
                          <UserAgentInput id='account-user-agent' value={userAgent}
                            onChange={setUserAgent} presets={uaPresets} disabled={interactionBusy} />
                          <FieldDescription>
                            {t('Applies to this account\'s upstream requests, including chats, balance queries, model fetching and testing, check-ins, and credential refreshes. Leave empty for the provider\'s defaults; select a preset or enter a value.')}
                          </FieldDescription>
                        </Field>
                      </SettingGroup> : null}
                      {routeContext ? <div className='acct-dlg-route-context'>{routeContext}</div>
                        : <p className='detail acct-dlg-group-note'><AccountSettingsIcon name='route' size={14} />
                          {t('Drag a group or a row handle in the account list to change the forwarding order.')}</p>}
                    </>
                  ) : null}

                  {pane === 'credential' && hasCredentialPane ? <>
                    {isCatpaw ? <BalanceTokenField configured={account.hasBalanceToken === true} value={balanceToken}
                      onChange={setBalanceToken} clearBusy={interactionBusy} onClear={() => void clearBalanceToken()} /> : null}
                    {isCustomAccount ? <CustomCredentialFields hasApiKey={target.hasApiKey === true} noAuth={noAuthDraft}
                      value={apiKeyDraft} clearBusy={interactionBusy} onClear={() => void clearApiKey()}
                      onValue={next => { setApiKeyDraft(next); if (next.trim()) setNoAuthDraft(false) }}
                      onToggleNoAuth={next => { setNoAuthDraft(next); if (next) setApiKeyDraft('') }} /> : null}
                    {supportsPlanChannel(target) ? <SettingGroup title={t('Plan in use')}>
                      <PlanChannelField plan={planChannel} hasJwt={account.canClaim === true} onChange={setPlanChannel} />
                    </SettingGroup> : null}
                    {provider ? <div className='acct-dlg-related'>
                      <AccountSettingsIcon name='agent' size={18} />
                      <div><h3>{t('Need to change the connection address or protocol?')}</h3><p>{t('{count} accounts share this provider\'s connection settings.', { count: providerCount })}</p></div>
                      <Button variant='link' size='xs' onClick={() => selectPane('provider')}>{t('Connection settings')}<AccountSettingsIcon name='caretRight' size={12} /></Button>
                    </div> : null}
                  </> : null}

                  {pane === 'query' && supportsUsage(target) ? <>
                    <SettingGroup title={t('Balance query')}>
                      <UsageSettingsSection draft={usageDraft} onChange={setUsageDraft} />
                    </SettingGroup>
                    {balanceSpec ? <SettingGroup title={t('Query credentials')} action={<Badge variant='secondary' shape='tag'>{balanceSpec.label}</Badge>}>
                      <BalanceQuerySection spec={balanceSpec} values={balanceQueryDraft} configured={balanceConfigured}
                        onValue={(key, next) => setBalanceQueryDraft(prev => ({ ...prev, [key]: next }))}
                        onClear={key => void clearBalanceQueryField(key)} clearBusy={interactionBusy} idPrefix='account-balance-query' />
                      {balanceNeedsCredentials ? <p className='detail acct-dlg-inline-note'>{t('Balance credentials are not complete: the balance column only gets readings once they are filled in; until then it shows "Not configured".')}</p> : null}
                    </SettingGroup> : null}
                    <p className='detail acct-dlg-group-note'><AccountSettingsIcon name='info' size={14} />{t('余额读数也是限制器余额规则的数据来源。')}</p>
                  </> : null}

                  {pane === 'limiter' ? <>
                    <LimiterSection account={target} draft={limiterDraft} onChange={setLimiterDraft} allowBalance={supportsUsage(target)} />
                    <p className='detail acct-dlg-group-note'><AccountSettingsIcon name='info' size={14} />
                      {t('No limits apply if there are no rules or all rules are off. Skipped accounts recover automatically; disabled accounts must be enabled manually.')}</p>
                  </> : null}

                  {pane === 'network' ? <>
                    <SettingGroup title={t('Connection method')}>
                      <ProxyForm draft={proxyDraft} onChange={setProxyDraft} idPrefix='account-proxy' allowRotate={allowRotate} />
                      {account.proxy?.error ? <p className='detail text-destructive acct-dlg-block'>
                        {t('Current proxy unavailable: {error} (no automatic fallback to a direct connection)', { error: account.proxy.error })}
                      </p> : null}
                    </SettingGroup>
                    <p className='detail acct-dlg-group-note'><AccountSettingsIcon name='shield' size={14} />{t('代理只作用于当前账号，不改变其他账号的出网方式。')}</p>
                  </> : null}
                  {pane === 'provider' && provider ? <ProviderSection provider={provider} count={providerCount}
                    name={providerName} protocol={providerProtocol} baseUrl={providerBaseUrl}
                    accountBaseUrl={target.baseUrl || ''} emulation={providerEmulation}
                    onName={setProviderName} onProtocol={setProviderProtocol} onBaseUrl={setProviderBaseUrl}
                    onEmulation={setProviderEmulation} onClearAccountBaseUrl={() => void clearAccountBaseUrl()}
                    disabled={interactionBusy} onRemove={() => void removeProvider()} onModels={() => void openModels()} /> : null}
                </div>
              </div>
            </div>
          </div>
        </DialogBody>
        <DialogFooter className='acct-dlg-footer'>
          <div className={`acct-dlg-save-state${dirtyCount ? ' dirty' : ''}`} role='status'>
            <AccountSettingsIcon name={status ? 'info' : dirtyCount ? 'timer' : 'shield'} size={16} />
            <div><strong>{status || (busy ? t('Saving…') : dirtyCount
              ? t('Unsaved changes in {count} sections', { count: dirtyCount }) : t('No unsaved changes'))}</strong>
              <p>{dirtyPanes.provider ? t('Includes shared provider settings affecting {count} accounts', { count: providerCount }) : t('Changes across all sections are saved together')}</p>
            </div>
          </div>
          <div className='acct-dlg-footer-actions'>
            <Button variant='outline' disabled={interactionBusy} onClick={() => void close()}>{t('Cancel')}</Button>
            <Button variant='default' disabled={interactionBusy || !dirtyCount} onClick={() => void save()}>{busy ? t('Saving…') : t('Save changes')}</Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
