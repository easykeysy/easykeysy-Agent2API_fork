/** set and use set。 */

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
  const [proxyDraft, setProxyDraft] = React.useState<ProxyDraft>(() => draftOfProxy(account?.proxy))
  const [balanceToken, setBalanceToken] = React.useState('')
  // ZCode 「 use 」（ ZCode as empty， that 1 not ）
  const [planChannel, setPlanChannel] = React.useState(() => zcodePlanOf(account))
  const [busy, setBusy] = React.useState(false)
  const [status, setStatus] = React.useState<React.ReactNode>('')
  const [provider, setProvider] = React.useState<ProviderSettings | null>(null)
  const [providerName, setProviderName] = React.useState('')
  const [providerProtocol, setProviderProtocol] = React.useState('chat_completions')
  const [providerBaseUrl, setProviderBaseUrl] = React.useState('')
  /** client（'' = not ；'opencode' = by CLI request） */
  const [providerEmulation, setProviderEmulation] = React.useState('')
  /** ： new Key（empty = not ，「」 by ） and 「none need 」 */
  const [apiKeyDraft, setApiKeyDraft] = React.useState('')
  const [noAuthDraft, setNoAuthDraft] = React.useState(account?.noAuth === true)
  // 「set」 （）： in has ，
  // none —— and planChannel 1 ， 1 
  const [usageDraft, setUsageDraft] = React.useState<UsageDraft>(() => usageDraftOf(account))
  // 「set」** need **（ has has ）：
  // none（ no has when is 1 empty， that 1 not ）
  const [balanceQueryDraft, setBalanceQueryDraft] = React.useState<Record<string, string>>(
    () => balanceQueryDraftOf(account?.balanceQuery),
  )
  // 「limit」 （ / Token then ）： and set 1 gating
  const [limiterDraft, setLimiterDraft] = React.useState<LimiterGroup[]>(() => limiterDraftOf(account))
  // current（ by id ， to 「」）
  const [pane, setPane] = React.useState<PaneId>(initialPane)
  // and limit（ ⋯ conc-dialog 「」）：0~999，0 = not 
  const [concDraft, setConcDraft] = React.useState(() => String(Number(account?.maxConcurrent) || 0))
  const [confirming, setConfirming] = React.useState(false)
  const actionLock = React.useRef(false)
  const paneScroll = React.useRef<HTMLDivElement | null>(null)
  const panePositions = React.useRef<Partial<Record<PaneId, number>>>({})
  const initial = React.useRef({
    enabled, allowRouteSkip, name, proxyDraft, planChannel, usageDraft,
    balanceQueryDraft, limiterDraft, concDraft, noAuthDraft,
  })
  const credentialLabel = providerOf(account).startsWith('custom-') ? t('凭证设置') : t('凭证与套餐')
  const paneInfo: Record<PaneId, { title: string; description: string; icon: string }> = {
    basic: { title: t('基本信息'), description: t('设置账号名称、启用状态与转发行为。'), icon: 'sliders' },
    credential: { title: credentialLabel, description: t('管理当前账号的凭证与可用套餐。'), icon: 'key' },
    query: { title: t('查询设置'), description: t('配置余额查询凭证与自动刷新间隔。'), icon: 'refresh' },
    limiter: { title: t('限制器'), description: t('用明确的规则保护当前账号的额度。'), icon: 'shield' },
    network: { title: t('网络'), description: t('设置当前账号访问上游的出网方式。'), icon: 'proxies' },
    provider: { title: t('连接设置'), description: t('统一管理提供商的协议、地址与兼容选项。'), icon: 'agent' },
  }
  const currentPane = paneInfo[pane]
  React.useLayoutEffect(() => {
    if (paneScroll.current) paneScroll.current.scrollTop = panePositions.current[pane] || 0
  }, [pane])

  function selectPane(next: PaneId): void {
    if (paneScroll.current) panePositions.current[pane] = paneScroll.current.scrollTop
    setPane(next)
  }

  // 「」 that 1 ：directory to （id before 「」，
  // / Base URL —— need ）。
  // id： is by id （ to key）， new not this use 
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
    }).catch(() => { /* directoryunavailable：「 not is 」， not that 1 */ })
    return () => { alive = false }
  }, [id])

  if (!account) {
    return (
      <Dialog open onOpenChange={next => { if (!next) onClose() }}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('账号设置')}</DialogTitle></DialogHeader>
          <DialogBody><p className='detail'>{t('账号不存在，请刷新后重试。')}</p></DialogBody>
          <DialogFooter><div className='mr-auto' /><Button variant='outline' onClick={onClose}>{t('关闭')}</Button></DialogFooter>
        </DialogContent>
      </Dialog>
    )
  }

  // not in when to 1 （ can can 1 ）。 use after ：
  // down async will ，TS not will 「 after 」
  const target: AccountRecord = account
  const peers = peersOf(target, allAccounts())
  const isCatpaw = providerOf(target) === 'catpaw'
  // ：（API Key / none need ） for 。 use provider before ——
  // and after `custom_account_provider` 1 （ after this can ）
  const isCustomAccount = providerOf(target).startsWith('custom-')
  // 「」 can ： has OpenCode Zen （ by IP ）。 use ** already to 
  // **（ up that effect ） not is directory —— this 1 up down has ，
  // and and 「」 is 1 （ emulation / baseUrl when ，
  // by up to ， after —— and after up 1 ）。
  // is this when ****（ as ：directory no to / 
  // ）：otherwise use not to this in is ； after 「 」
  // ， not will 400。
  const allowRotate = isOpencodeZenProvider(provider, target.baseUrl) || proxyDraft.mode === 'rotate'
  // that 1 （「set」 ）： and 「 already config」 by 
  // —— `scheme` is this 1 id，`has*` is in 
  // （ not ， after `store_view::balance_query_public`）
  const balanceSpec = balanceSpecOf(target.balanceQuery?.scheme)
  const balanceConfigured: Record<string, boolean> = {}
  for (const field of balanceSpec?.fields ?? []) {
    balanceConfigured[field.key] = balanceFieldConfigured(target.balanceQuery, field)
  }
  // 「 need no 」 when to 1 in （ not is error： will 「not configured」，
  // use in that 1 to is this after ）
  const balanceNeedsCredentials = Boolean(balanceSpec && balanceSpec.fields.length)
    && !balanceQueryConfigured(target.balanceQuery)
  const providerCount = peers.length + 1
  const before = initial.current
  const same = (left: unknown, right: unknown): boolean => JSON.stringify(left) === JSON.stringify(right)
  const dirtyPanes: Record<PaneId, boolean> = {
    basic: enabled !== before.enabled || allowRouteSkip !== before.allowRouteSkip
      || (name.trim() || before.name) !== before.name || concDraft !== before.concDraft,
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
      if (!ask) throw new Error(t('确认弹窗不可用，请重新打开页面'))
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
    if (dirtyCount && !await askConfirmation(t('放弃未保存的修改？'),
      t('所有分区中尚未保存的修改都会被放弃。已确认的独立清除操作不受影响。'), t('放弃修改'))) return
    onClose()
  }

  /** （， not by —— is 1 ） */
  async function clearBalanceToken(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (!await askConfirmation(t('清除余额查询凭证？'),
      t('确认后立即生效，不受底部保存按钮控制。通常可继续复用转发用的登录凭证。'), t('清除'), true)) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { balanceToken: null })
      setAccount(saved)
      setBalanceToken('')
      toast(t('✅ 已清除余额查询凭证'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('账号已保存，但列表刷新失败：{error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('清除失败：{error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * API Key（ and 1 ： ）。
   * after 「not configured」， when will —— this 1 ，
   * up that 「not configured」 will ， use to after 。
   */
  async function clearApiKey(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (!await askConfirmation(t('清除已存 API Key？'),
      t('确认后立即生效，不受底部保存按钮控制。没有凭证且未开启「无需鉴权」时，此账号将被转发跳过。'), t('清除'), true)) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { apiKey: null })
      setAccount(saved)
      setApiKeyDraft('')
      toast(t('✅ 已清除 API Key（该账号现在没有凭证，转发会被跳过）'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('账号已保存，但列表刷新失败：{error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('清除失败：{error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * this up ， Base URL
   * （ and API Key 1 ： ）。
   *
   * new when will when ， up 
   * —— is after request old ， up not to that old 
   * （issue #455）。 this by is ； new not 
   * this ， when after will etc old that 。
   */
  async function clearAccountBaseUrl(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    const nextBase = providerBaseUrl.trim()
    if (!await askConfirmation(t('改回跟随提供商？'),
      t('这个账号将改用提供商当前保存的 Base URL：{url}。确认后立即生效，不受底部保存按钮控制。',
        { url: nextBase || '—' }), t('改回跟随'))) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { baseUrl: null })
      setAccount(saved)
      toast(t('✅ 已改回跟随提供商的 Base URL'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('账号已保存，但列表刷新失败：{error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('操作失败：{message}', { message: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * 1 already config （：NewAPI / 
   * / ）。
   *
   * and API Key 1 ：** **（ not by that 1 ）
   * —— not ，empty is 「 not 」， can this 
   * （ null to after ， `normalize_balance_query_patch`）。
   * after will 「not configured」， use to after 。
   */
  async function clearBalanceQueryField(key: BalanceFieldKey): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (!await askConfirmation(t('清除查询凭证？'),
      t('确认后立即生效，余额查询可能显示「未配置」。取消账号设置不会恢复已清除的凭证。'), t('清除'), true)) return
    actionLock.current = true
    setBusy(true)
    try {
      const saved = await updateAccountConfirmed(id, { balanceQuery: { [key]: null } })
      setAccount(saved)
      setBalanceQueryDraft(prev => ({ ...prev, [key]: '' }))
      toast(t('✅ 已清除查询凭证'))
      try {
        await refreshSavedAccounts()
      } catch (error) {
        toast(t('账号已保存，但列表刷新失败：{error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      toast(t('清除失败：{error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  /**
   * 「」 that 1 ： no this 1 （） or no → null。
   *
   * and after for （`update_custom_credentials`）：
   * · → new Key（ after will 「none need 」）；
   * · → noAuth（= key， after 1 ）；
   * · empty and no → not （empty is 「 not 」， by ）。
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

  /** 「」 that 1 ： no this 1 （） or no → null（ not ） */
  function readProviderPatch(): { error: string } | { id: string; name: string; protocol: string; baseUrl: string; clientEmulation: string } | null {
    if (!provider) return null
    const nextName = providerName.trim()
    const nextBase = providerBaseUrl.trim()
    // empty / empty Base URL after will 400， that when ** already down ** —— use to 「failure」
    // 。 in before 1 
    if (!nextName) return { error: t('请填写提供商名称') }
    if (!nextBase) return { error: t('请填写提供商的 Base URL') }
    try {
      if (!['http:', 'https:'].includes(new URL(nextBase).protocol)) throw new Error('invalid protocol')
    } catch {
      return { error: t('请填写有效的 HTTP 或 HTTPS 地址') }
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
    // set（）： and 1 ， this 1 ；
    // not in before （ after apply_patch for 1 then will 400）
    const usageChanges = supportsUsage(target) ? readUsageChanges(target, usageDraft) : null
    if (usageChanges && 'error' in usageChanges) {
      selectPane('query')
      setStatus(<span className='text-destructive'>{usageChanges.error}</span>)
      return
    }
    // limit（ / Token then ）： and set 1 ； when already ，
    // this 「 up 」（ for not will empty 1 ， and after 1 ）
    const limiterChanges = readLimiterChanges(target, limiterDraft)
    // （ ）： by —— empty = not ，
    // （ after none when not ）； this 1 no has when null
    const balanceQueryPatch = readBalanceQueryChanges(target.balanceQuery?.scheme, balanceQueryDraft)
    // （）： and / 1 ， this 1 
    // （ is PATCH /api/accounts ， after `update_custom_credentials`）
    const credentialPatch = readCredentialPatch() || {}
    // and limit（「」）： 1 （number not ）， after 
    // 400 this is after —— and conc-dialog 1 clamp（0~999，0 = not ；
    // CodeArts 0 by up default and ， can concurrencyDefault）
    const concRaw = Number(concDraft)
    if (!Number.isFinite(concRaw) || concRaw < 0) {
      selectPane('basic')
      setStatus(<span className='text-destructive'>{t('并发上限必须是 0~{max} 的整数', { max: CONC_MAX })}</span>)
      toast(t('并发上限必须是 0~{max} 的整数', { max: CONC_MAX }), 'err')
      return
    }
    const concNext = Math.min(CONC_MAX, Math.round(concRaw))
    const concCurrent = Number(target.maxConcurrent) || 0
    const concPatch: Record<string, unknown> = concNext !== concCurrent ? { maxConcurrent: concNext } : {}
    if (providerPatch && !await askConfirmation(t('保存提供商共用配置？'),
      t('「{name}」名下 {count} 个账号共用这些连接设置。修改协议或 Base URL 可能中断正在进行的请求。',
        { name: provider?.name || provider?.id || '', count: providerCount }), t('确认保存'))) return

    actionLock.current = true
    setBusy(true)
    try {
      // CatPaw is ** by need ** （ no has this 1 ， not in ）：
      // empty = not （ after to hasBalanceToken 、 not ， this 
      // is empty ； if 「empty」， use set will ）。
      // up that by 。
      const balancePatch = isCatpaw && balanceToken.trim() ? { balanceToken: balanceToken.trim() } : {}
      // ： in ** ** when up （ and after `zcode::normalize_plan`
      // 1 ）。 1 no not will ， will after that 「 → …」 
      // in set when 1 ，
      const planPatch = supportsPlanChannel(target) && planChannel !== zcodePlanOf(target)
        ? { zcodePlan: planChannel }
        : {}
      // （priority）** not in this **： down by （
      // accounts-data reorderQueue）。set「」 ，
      // 1 「」 not this 1 
      const nextName = name.trim() || target.name
      const saved = await updateAccountConfirmed(id, {
        ...(nextName !== target.name ? { name: nextName } : {}),
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
        throw new Error(t('账号更新结果未确认，请刷新后核对设置'))
      }
      setAccount(saved)
      // that 1 in after （ is ，）。failure when already down ，
      // in that —— 「failure」 will 1 
      if (providerPatch) {
        try {
          const updateProvider = shared().wbCustomProvidersUi?.update
          if (!updateProvider) throw new Error(t('账号更新接口不可用，请重新打开页面'))
          await updateProvider(providerPatch)
        } catch (error) {
          selectPane('provider')
          setStatus(<span className='text-destructive'>{t('账号已保存，但提供商未更新：{error}', { error: errorMessage(error) })}</span>)
          try { await refreshSavedAccounts() }
          catch (refreshError) { toast(t('账号已保存，但列表刷新失败：{error}', { error: errorMessage(refreshError) }), 'err') }
          setBusy(false)
          return
        }
      }
      // and success： already ， new is up ， not this use for 「 in …」
      // etc 1 
      onClose()
      toast(t('✅ 账号设置已保存'))
      // after must new ： before not new ， up / is old ，
      // need etc 20 seconds that 1 update —— use to is 「seconds」。
      // 1 ： new failure（ after will ）， not can down 
      // catch 「failure」（ its already success）
      try {
        await refreshSavedAccounts()
        await syncBalancesSnapshot()
      } catch (error) {
        toast(t('账号已保存，但列表刷新失败：{error}', { error: errorMessage(error) }), 'err')
      }
    } catch (error) {
      // after failure（）： in Reason，
      setStatus(<span className='text-destructive'>{errorMessage(error)}</span>)
      toast(t('保存失败：{error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setBusy(false)
    }
  }

  // only in has or when 。
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
      if (!remove) throw new Error(t('账号更新接口不可用，请重新打开页面'))
      if (await remove(provider.id)) onClose()
    } catch (error) {
      toast(t('删除失败：{error}', { error: errorMessage(error) }), 'err')
    } finally {
      actionLock.current = false
      setConfirming(false)
    }
  }

  async function openModels(): Promise<void> {
    if (interactionBusy || actionLock.current) return
    if (dirtyCount && !await askConfirmation(t('放弃未保存的修改？'),
      t('前往模型管理会关闭账号设置，尚未保存的修改不会生效。'), t('前往模型管理'))) return
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
            <div className='acct-dlg-title-row'><DialogTitle>{t('账号设置')}</DialogTitle>
              {isCustomAccount ? <Badge variant='secondary' shape='tag'>{t('自定义 API')}</Badge> : null}
            </div>
            <p><span>{labelOf(account)}</span><span className='acct-dlg-title-dot'>·</span>
              <span className={`acct-dlg-account-state${account.enabled === false ? ' off' : ''}`}>
                <BadgeDot />{account.enabled === false ? t('已停用') : t('已启用')}
              </span>
            </p>
          </div>
        </DialogHeader>
        <DialogBody className='acct-dlg-body'>
          <div className='acct-dlg'>
            <Sidebar className='acct-dlg-rail' aria-label={t('账号设置分组')}>
              <NavList className='acct-dlg-nav'>
                <div className='acct-dlg-nav-label'>{t('当前账号')}</div>
                {accountPanes.map(id => <NavItem key={id} active={pane === id} title={paneInfo[id].title}
                  disabled={interactionBusy} className='shadow-none'
                  icon={<AccountSettingsIcon name={paneInfo[id].icon} />}
                  trailing={dirtyPanes[id] ? <span className='acct-dlg-dirty-dot' aria-label={t('有未保存的修改')} /> : undefined}
                  onClick={() => selectPane(id)}>{paneInfo[id].title}</NavItem>)}
                {provider ? <>
                  <div className='acct-dlg-nav-divider' />
                  <div className='acct-dlg-nav-label'>{t('提供商')}<span>{t('共用')}</span></div>
                  <NavItem active={pane === 'provider'} disabled={interactionBusy} title={paneInfo.provider.title}
                    icon={<AccountSettingsIcon name={paneInfo.provider.icon} />}
                    trailing={dirtyPanes.provider ? <span className='acct-dlg-dirty-dot' aria-label={t('有未保存的修改')} /> : undefined}
                    onClick={() => selectPane('provider')}>{paneInfo.provider.title}</NavItem>
                </> : null}
              </NavList>
              {provider ? <div className='acct-dlg-rail-context'><strong><AccountSettingsIcon name='accounts' size={14} />{providerLabel}</strong>
                <p>{t('{count} 个账号共用连接配置', { count: providerCount })}</p>
              </div> : null}
            </Sidebar>
            <div className='acct-dlg-main'>
              <div className='acct-dlg-pane-head'>
                <div><h3>{currentPane.title}</h3><p>{currentPane.description}</p></div>
                <span className={`acct-dlg-scope${pane === 'provider' ? ' shared' : ''}`}>
                  <AccountSettingsIcon name='accounts' size={13} />
                  {pane === 'provider' ? t('{count} 个账号共用', { count: providerCount }) : t('仅当前账号')}
                </span>
              </div>
              <div ref={paneScroll} className='acct-dlg-scroll' aria-busy={busy}>
                <div className='acct-dlg-pane' inert={interactionBusy ? true : undefined}>
                  {pane === 'basic' ? (
                    <>
                      <SettingGroup title={t('账号信息')}>
                        <Field className='acct-dlg-field'>
                          <FieldLabel htmlFor='account-name-input'>{t('备注名')}</FieldLabel>
                          <Input id='account-name-input' maxLength={100} placeholder={t('账号显示名称')}
                            value={name} onChange={event => setName(event.currentTarget.value)} />
                        </Field>
                        <div className='acct-dlg-readonly'><span>{t('账号 ID')}</span><code>{target.id}</code></div>
                        <Field className='acct-dlg-toggle-field'>
                          <div><FieldLabel htmlFor='account-enabled'>{t('启用账号')}</FieldLabel>
                            <FieldDescription>{t('关闭后不参与转发，余额查询不受影响。')}</FieldDescription>
                          </div>
                          <Switch id='account-enabled' checked={enabled} onCheckedChange={setEnabled} />
                        </Field>
                      </SettingGroup>
                      <SettingGroup title={t('转发行为')}>
                        <Field className='acct-dlg-toggle-field'>
                          <div><FieldLabel htmlFor='account-route-skip'>{t('失败后允许换号')}</FieldLabel>
                            <FieldDescription>{t('关闭后，该账号转发失败时返回本次错误，不再顺延其他账号。禁用、无凭证和限制器等保护仍生效。')}</FieldDescription>
                          </div>
                          <Switch id='account-route-skip' checked={allowRouteSkip} onCheckedChange={setAllowRouteSkip} />
                        </Field>
                        <Field className='acct-dlg-concurrency'>
                          <div><FieldLabel htmlFor='account-conc-input'>{t('并发上限')}</FieldLabel>
                            <FieldDescription>{t('达到上限后，将请求交给其他账号。')}</FieldDescription>
                          </div>
                          <Input id='account-conc-input' type='number' min={0} max={CONC_MAX} step={1}
                            value={concDraft} onChange={event => setConcDraft(event.currentTarget.value)} />
                        </Field>
                      </SettingGroup>
                      <p className='detail acct-dlg-hint'>{concDefault > 0
                        ? t('0 = 按本家默认 {default}', { default: concDefault }) : t('0 = 不限')}</p>
                      {routeContext ? <div className='acct-dlg-route-context'>{routeContext}</div>
                        : <p className='detail acct-dlg-group-note'><AccountSettingsIcon name='route' size={14} />
                          {t('在账号列表中拖动分组或行首把手，调整转发顺序。')}</p>}
                    </>
                  ) : null}

                  {pane === 'credential' && hasCredentialPane ? <>
                    {isCatpaw ? <BalanceTokenField configured={account.hasBalanceToken === true} value={balanceToken}
                      onChange={setBalanceToken} clearBusy={interactionBusy} onClear={() => void clearBalanceToken()} /> : null}
                    {isCustomAccount ? <CustomCredentialFields hasApiKey={target.hasApiKey === true} noAuth={noAuthDraft}
                      value={apiKeyDraft} clearBusy={interactionBusy} onClear={() => void clearApiKey()}
                      onValue={next => { setApiKeyDraft(next); if (next.trim()) setNoAuthDraft(false) }}
                      onToggleNoAuth={next => { setNoAuthDraft(next); if (next) setApiKeyDraft('') }} /> : null}
                    {supportsPlanChannel(target) ? <SettingGroup title={t('使用套餐')}>
                      <PlanChannelField plan={planChannel} hasJwt={account.canClaim === true} onChange={setPlanChannel} />
                    </SettingGroup> : null}
                    {provider ? <div className='acct-dlg-related'>
                      <AccountSettingsIcon name='agent' size={18} />
                      <div><h3>{t('要修改连接地址或协议？')}</h3><p>{t('{count} 个账号共用提供商的连接配置。', { count: providerCount })}</p></div>
                      <Button variant='link' size='xs' onClick={() => selectPane('provider')}>{t('连接设置')}<AccountSettingsIcon name='caretRight' size={12} /></Button>
                    </div> : null}
                  </> : null}

                  {pane === 'query' && supportsUsage(target) ? <>
                    <SettingGroup title={t('自动查询余额')}>
                      <UsageSettingsSection draft={usageDraft} onChange={setUsageDraft} />
                    </SettingGroup>
                    {balanceSpec ? <SettingGroup title={t('查询凭证')} action={<Badge variant='secondary' shape='tag'>{balanceSpec.label}</Badge>}>
                      <BalanceQuerySection spec={balanceSpec} values={balanceQueryDraft} configured={balanceConfigured}
                        onValue={(key, next) => setBalanceQueryDraft(prev => ({ ...prev, [key]: next }))}
                        onClear={key => void clearBalanceQueryField(key)} clearBusy={interactionBusy} idPrefix='account-balance-query' />
                      {balanceNeedsCredentials ? <p className='detail acct-dlg-inline-note'>{t('查询凭证还没配齐：填好后余额列才有读数，在此之前会显示「未配置」。')}</p> : null}
                    </SettingGroup> : null}
                    <p className='detail acct-dlg-group-note'><AccountSettingsIcon name='info' size={14} />{t('余额读数也是限制器余额规则的数据来源。')}</p>
                  </> : null}

                  {pane === 'limiter' ? <>
                    <LimiterSection account={target} draft={limiterDraft} onChange={setLimiterDraft} allowBalance={supportsUsage(target)} />
                    <p className='detail acct-dlg-group-note'><AccountSettingsIcon name='info' size={14} />
                      {t('没有规则或全部停用时不限制；跳过后自动恢复，禁用后需手动启用。')}</p>
                  </> : null}

                  {pane === 'network' ? <>
                    <SettingGroup title={t('出网方式')}>
                      <ProxyForm draft={proxyDraft} onChange={setProxyDraft} idPrefix='account-proxy' allowRotate={allowRotate} />
                      {account.proxy?.error ? <p className='detail text-destructive acct-dlg-block'>
                        {t('当前代理不可用：{error}（转发时会回退直连）', { error: account.proxy.error })}
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
            <div><strong>{status || (busy ? t('正在保存…') : dirtyCount
              ? t('{count} 个分区有未保存的更改', { count: dirtyCount }) : t('没有未保存的更改'))}</strong>
              <p>{dirtyPanes.provider ? t('包含提供商共用配置，将影响 {count} 个账号', { count: providerCount }) : t('各分区的更改将统一保存')}</p>
            </div>
          </div>
          <div className='acct-dlg-footer-actions'>
            <Button variant='outline' disabled={interactionBusy} onClick={() => void close()}>{t('取消')}</Button>
            <Button variant='default' disabled={interactionBusy || !dirtyCount} onClick={() => void save()}>{busy ? t('保存中…') : t('保存更改')}</Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
