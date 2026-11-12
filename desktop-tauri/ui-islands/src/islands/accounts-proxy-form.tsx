/** set and use 。 */

import * as React from 'react'
import {
  Button, Checkbox, Input, Label, MultiSelect, SegmentedControl,
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
  Field, FieldGroup, FieldLabel,
} from '@ui'
import { t } from '../i18n'
import {
  errorMessage, esc, poolItemLabel, shared, toast,
  type AccountRecord, type PoolItem,
} from './accounts-shared'
import { proxyPoolOptions } from './accounts-data'
import { DIALOG_SEG_CLASS } from './dialog-segmented'
import type { ClashProxySnapshot } from '../../../ui-shared/clash-proxy'

/** will ， 1 8 。 */
const ROTATE_TEST_LIMIT = 8

export type ProxyPayload =
  | null
  | { source: 'pool'; proxyId: string }
  | { source: 'rotate'; all: boolean; proxyIds?: string[] }
  | { source: 'clash'; listenerUid: string }
  | { source: 'custom'; protocol: 'http' | 'socks5'; host: string; port: number; username: string; password: string }

export type ProxyDraft = {
  mode: 'none' | 'pool' | 'rotate' | 'clash' | 'custom' | 'retired'
  listenerUid: string
  /** id（mode === 'pool'） */
  proxyId: string
  /** ：true = enabled （""，default；false = in rotateIds ） */
  rotateAll: boolean
  /** ： need id（rotateAll as true when not use ） */
  rotateIds: string[]
  protocol: 'http' | 'socks5'
  host: string
  port: string
  username: string
  password: string
}

/** proxy → （ and after workbuddy-proxy.mjs 1 ） */
export function draftOfProxy(proxy: AccountRecord['proxy']): ProxyDraft {
  const source = proxy?.config?.source || proxy?.source
  const config = proxy?.config || null
  const rotate = source === 'rotate'
  return {
    mode: source === 'pool' ? 'pool'
      : rotate ? 'rotate'
        : source === 'clash' ? 'clash' : source === 'custom' ? 'custom' : source === 'subscription' ? 'retired' : 'none',
    listenerUid: source === 'clash' ? String(config?.listenerUid || '') : '',
    proxyId: source === 'pool' ? String(config?.proxyId || '') : '',
    // `all` by 「」（ after 1 ： that is this 1 default）
    rotateAll: rotate ? config?.all !== false : true,
    rotateIds: rotate && Array.isArray(config?.proxyIds) ? config.proxyIds.map(String) : [],
    protocol: config?.protocol === 'socks5' ? 'socks5' : 'http',
    host: config?.host || '',
    port: config?.port === undefined || config?.port === null ? '' : String(config.port),
    username: config?.username || '',
    password: config?.password || '',
  }
}

/** → payload；「 to use 」 error（ toast / ） */
export function readProxyDraft(draft: ProxyDraft): ProxyPayload {
  if (draft.mode === 'none') return null
  if (draft.mode === 'retired') throw new Error(t('原代理来源已下线，请选择已保存的代理、自定义地址或直连'))
  if (draft.mode === 'clash') {
    if (!draft.listenerUid) throw new Error(t('请先选择 Clash Verge 出口'))
    return { source: 'clash', listenerUid: draft.listenerUid }
  }
  if (draft.mode === 'pool') {
    if (!draft.proxyId) throw new Error(t('请先选择已保存的代理'))
    return { source: 'pool', proxyId: draft.proxyId }
  }
  if (draft.mode === 'rotate') {
    // 「」 not proxyIds： after add （ after 1 is this ）
    if (draft.rotateAll) return { source: 'rotate', all: true }
    if (!draft.rotateIds.length) throw new Error(t('轮询至少要选择一条代理（或改选「全部」）'))
    return { source: 'rotate', all: false, proxyIds: [...draft.rotateIds] }
  }
  const host = draft.host.trim()
  const port = Number(draft.port)
  if (!host) throw new Error(t('请填写代理主机地址'))
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error(t('代理端口必须是 1-65535 的整数'))
  return {
    source: 'custom',
    protocol: draft.protocol === 'socks5' ? 'socks5' : 'http',
    host,
    port,
    username: draft.username.trim(),
    password: draft.password,
  }
}

function clashOptionLabel(option: NonNullable<ClashProxySnapshot['options']>[number]): string {
  return `${option.name || option.uid} :${option.port}${option.profileActive === false ? t('（其他订阅，可能未生效）') : ''}${option.enabled === false ? t('（已在 Clash 中禁用）') : ''}`
}

export function ProxyForm({
  draft,
  onChange,
  idPrefix,
  allowRotate = false,
}: {
  draft: ProxyDraft
  onChange: (next: ProxyDraft) => void
  idPrefix: string
  /** is to 「」（ has OpenCode Zen as ，） */
  allowRotate?: boolean
}) {
  const [testing, setTesting] = React.useState(false)
  const [testResult, setTestResult] = React.useState<React.ReactNode>(null)
  /** ：「 already 」 and 「」 use （， accounts-data） */
  const [pool, setPool] = React.useState<PoolItem[] | null>(null)
  const [poolError, setPoolError] = React.useState('')
  const [clash, setClash] = React.useState<ClashProxySnapshot | null>(null)
  const [clashError, setClashError] = React.useState('')
  const options = clash?.options || []
  const set = (patch: Partial<ProxyDraft>): void => onChange({ ...draft, ...patch })

  const loadClash = React.useCallback((): void => {
    setClashError('')
    void Promise.resolve(shared().workbuddyDesktop?.getProxies?.()).then(data => {
      if (!data?.clash || !Array.isArray(data.clash.options)) throw new Error(t('代理列表响应异常'))
      setClash(data.clash)
    }).catch(error => {
      setClash({ available: false, options: [] })
      setClashError(errorMessage(error))
    })
  }, [])
  React.useEffect(() => { if (draft.mode === 'clash' && clash === null) loadClash() }, [draft.mode, clash, loadClash])

  /** 1 （ ： and 「 new 」） */
  const loadPool = React.useCallback((force = false): void => {
    setPoolError('')
    void proxyPoolOptions({ force })
      .then(items => setPool(items))
      .catch(error => {
        setPool([])
        setPoolError(errorMessage(error))
      })
  }, [])
  // has in （ or is ） use that when —— its use not as 1 
  // request；， to this 1 not will 
  React.useEffect(() => {
    if ((draft.mode === 'pool' || draft.mode === 'rotate') && pool === null) loadPool()
  }, [draft.mode, pool, loadPool])

  /**
   * current will use to （ and after `proxy_rotate::candidate_ids` ：
   * 「」= enabled ，； = that ）。
   */
  function rotateCandidateIds(): string[] {
    if (draft.rotateAll) {
      return (pool ?? []).filter(item => item.enabled !== false).map(item => item.id)
    }
    return [...draft.rotateIds]
  }

  /** use current */
  async function test(): Promise<void> {
    if (testing) return
    // ：（ testProxyPoolItem —— will that 
    // 「 up 」， and that by 1 ）。 can can ， 1 is 
    // ， before ROTATE_TEST_LIMIT and 「 has no 」；
    // as 1 not when ， use 「」 「」。
    if (draft.mode === 'rotate') {
      const ids = rotateCandidateIds()
      if (!ids.length) {
        toast(t('轮询至少要选择一条代理（或改选「全部」）'), 'err')
        return
      }
      const targets = ids.slice(0, ROTATE_TEST_LIMIT)
      setTesting(true)
      setTestResult(t('正在连接上游…'))
      let ok = 0
      const failures: string[] = []
      for (const id of targets) {
        const name = pool?.find(item => item.id === id)?.name || id
        try {
          const data = await shared().workbuddyDesktop?.testProxyPoolItem?.(id)
          if (data?.success) ok += 1
          else failures.push(`${name}（${data?.error || t('连接失败')}）`)
        } catch (error) {
          failures.push(`${name}（${errorMessage(error)}）`)
        }
      }
      const rest = ids.length - targets.length
      const summary = t('✅ {ok}/{total} 个出口可用', { ok, total: targets.length })
        + (rest > 0 ? t('（只测了前 {limit} 个，还有 {rest} 个未测）', { limit: ROTATE_TEST_LIMIT, rest }) : '')
      setTestResult(
        <span className={failures.length ? 'text-destructive' : 'text-success'}>
          {summary}{failures.length ? `　${esc(failures.join(t('；')))}` : ''}
        </span>,
      )
      setTesting(false)
      return
    }
    // use ： is ** that already **（testProxyPoolItem will 
    // 「 up 」， and that 1 by 1 ）—— when to 
    if (draft.mode === 'pool') {
      if (!draft.proxyId) {
        toast(t('请先选择已保存的代理'), 'err')
        return
      }
      setTesting(true)
      setTestResult(t('正在连接上游…'))
      try {
        const data = await shared().workbuddyDesktop?.testProxyPoolItem?.(draft.proxyId)
        setTestResult(data?.success
          ? (
              <span className='text-success'>
                {t('✅ 出口可用')}
                {data.ip ? t('　出口 IP {ip}', { ip: esc(data.ip) }) : ''}
                {data.durationMs !== undefined && data.durationMs !== '' ? `　${esc(String(data.durationMs))}ms` : ''}
              </span>
            )
          : <span className='text-destructive'>❌ {esc(data?.error || t('连接失败'))}</span>)
      } catch (error) {
        setTestResult(<span className='text-destructive'>❌ {esc(errorMessage(error))}</span>)
      } finally {
        setTesting(false)
      }
      return
    }
    let proxy: ProxyPayload
    try {
      proxy = readProxyDraft(draft)
    } catch (error) {
      toast(errorMessage(error), 'err')
      return
    }
    setTesting(true)
    setTestResult(t('正在连接上游…'))
    try {
      const data = await shared().workbuddyDesktop?.testProxy?.({ proxy })
      if (data?.success) {
        setTestResult(
          <span className='text-success'>
            {t('✅ 出口可用')}
            {data.status !== undefined && data.status !== '' ? `　HTTP ${esc(String(data.status))}` : ''}
            {data.ip ? t('　出口 IP {ip}', { ip: esc(data.ip) }) : ''}
            {data.durationMs !== undefined && data.durationMs !== '' ? `　${esc(String(data.durationMs))}ms` : ''}
          </span>,
        )
      } else {
        setTestResult(<span className='text-destructive'>❌ {esc(data?.error || t('连接失败'))}</span>)
      }
    } catch (error) {
      setTestResult(<span className='text-destructive'>❌ {esc(errorMessage(error))}</span>)
    } finally {
      setTesting(false)
    }
  }

  return (
    <div className='acct-proxy-form flex flex-col'>
      {draft.mode === 'retired' ? <p className='mb-3 text-xs text-destructive'>{t('原代理来源已下线，请选择已保存的代理、自定义地址或直连')}</p> : null}
      {/* ： 1 。 （ and 1 ）——
           this RadioGroup 1 ， for not up 。 */}
      <SegmentedControl<ProxyDraft['mode']>
        aria-label={t('代理方式')}
        className={DIALOG_SEG_CLASS}
        value={draft.mode}
        onValueChange={value => set({ mode: value })}
        options={[
          { value: 'none', label: t('无代理（直连）') },
          { value: 'pool', label: t('已保存的代理') },
          // for OpenCode Zen （ allowRotate ）
          ...(allowRotate ? [{
            value: 'rotate' as const,
            label: t('轮询（换 IP）'),
            title: t('逐请求轮换出口（换 IP）：OpenCode Zen 免费档按 IP 限流，单个 IP 的额度用完会对它一律 429'),
          }] : []),
          // Clash Verge already down ： has config in up when this 1 ，
          // use can current、 and has will （ no ）
          ...(draft.mode === 'clash' ? [{ value: 'clash' as const, label: 'Clash Verge' }] : []),
          { value: 'custom', label: t('自定义') },
        ]}
      />

      {draft.mode === 'pool' ? (
        <div className='mt-3'>
          <Field orientation='horizontal' className='field-row'>
            <FieldLabel htmlFor={`${idPrefix}-pool-exit`}>{t('代理')}</FieldLabel>
            {/* value as （empty = no ）： from undefined to will Base UI
                「 → 」 ， not to undefined */}
            <Select value={draft.proxyId} disabled={!pool?.length}
              onValueChange={value => set({ proxyId: String(value) })}>
              <SelectTrigger id={`${idPrefix}-pool-exit`} className='min-w-[220px]'>
                <SelectValue>
                  {pool?.find(item => item.id === draft.proxyId)
                    ? poolItemLabel(pool.find(item => item.id === draft.proxyId)!)
                    : draft.proxyId ? t('当前代理不可用，请重新选择') : (pool === null ? t('正在读取…') : pool.length ? t('请选择代理') : t('还没有保存的代理'))}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {(pool ?? []).map(item => (
                  // already use not optional： after failure（），
                  // use will to 「 already use 」。 need enabled「」。
                  // current is use when （ is 1 label）
                  <SelectItem key={item.id} value={item.id} disabled={item.enabled === false}>
                    {poolItemLabel(item)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button variant='outline' size='sm' onClick={() => loadPool(true)}>{t('重新读取')}</Button>
          </Field>
          <div className='detail mt-1.5'>
            {poolError
              ? t('读取失败：{error}', { error: poolError })
              : pool === null
                ? t('正在读取代理列表…')
                : pool.length
                  ? t('地址与端口由「网络代理」页管理：那边改一次、测一次，所有引用它的账号一起生效')
                  : t('「网络代理」页还没有出口 —— 去那里添加手动地址或同步 Clash Verge 的端点')}
          </div>
        </div>
      ) : null}

      {draft.mode === 'rotate' ? (
        <div className='mt-3'>
          {/* 「」 is default： up = enabled use （ new 
              ， not this ）。。
               this 1 ** not .field-row**：set label 
               will `.acct-dlg-main .field-row > label` 76px ，
              「（）」 76px ，nowrap will after 
              （）—— use flex ， in after */}
          <div className='flex flex-wrap items-center gap-x-2.5 gap-y-1'>
            <Checkbox id={`${idPrefix}-rotate-all`} checked={draft.rotateAll}
              aria-label={t('轮询全部代理节点')}
              onCheckedChange={value => set({ rotateAll: value === true })} />
            <Label htmlFor={`${idPrefix}-rotate-all`} className='cursor-pointer text-xs text-subtle'>
              {t('全部代理节点（推荐）')}
            </Label>
            <span className='detail'>
              {t('「网络代理」页里每个启用的出口轮流使用；之后新加的出口自动加入')}
            </span>
          </div>
          {!draft.rotateAll ? (
            <Field orientation='horizontal' className='field-row mt-2.5'>
              <FieldLabel htmlFor={`${idPrefix}-rotate-exits`}>{t('轮询的代理')}</FieldLabel>
              <MultiSelect id={`${idPrefix}-rotate-exits`}
                value={draft.rotateIds}
                onValueChange={values => set({ rotateIds: values.map(String) })}
                // use ： after use when will （`candidate_ids` enabled），
                // can etc use 1 not 
                options={(pool ?? []).map(item => ({
                  value: item.id,
                  label: poolItemLabel(item),
                  disabled: item.enabled === false,
                }))}
                placeholder={t('选择要轮询的代理（可多选）')}
                aria-label={t('轮询的代理')} />
              <Button variant='outline' size='sm' onClick={() => loadPool(true)}>{t('重新读取')}</Button>
            </Field>
          ) : null}
          <div className='detail mt-1.5'>
            {poolError
              ? t('读取失败：{error}', { error: poolError })
              : pool === null
                ? t('正在读取代理列表…')
                : pool.length
                  ? t('每条请求换一个出口（按顺序轮流），上游看到的 IP 因此是变化的；出口的地址与端口由「网络代理」页管理')
                  : t('「网络代理」页还没有出口 —— 去那里添加手动地址或同步 Clash Verge 的端点')}
          </div>
        </div>
      ) : null}

      {draft.mode === 'clash' ? <div className='mt-3'>
        <Field orientation='horizontal' className='field-row'>
          <FieldLabel htmlFor={`${idPrefix}-clash-exit`}>{t('出口')}</FieldLabel>
          <Select value={draft.listenerUid} disabled={!options.length} onValueChange={value => set({ listenerUid: String(value) })}>
            <SelectTrigger id={`${idPrefix}-clash-exit`} className='min-w-[220px]'>
              <SelectValue>{options.find(option => option.uid === draft.listenerUid)
                ? clashOptionLabel(options.find(option => option.uid === draft.listenerUid)!)
                : draft.listenerUid ? t('当前代理不可用，请重新选择') : t('请选择出口')}</SelectValue>
            </SelectTrigger>
            <SelectContent>{options.filter(option => option.uid).map(option => <SelectItem key={option.uid} value={String(option.uid)} disabled={option.enabled === false}>
              {clashOptionLabel(option)}
            </SelectItem>)}</SelectContent>
          </Select>
          <Button variant='outline' size='sm' onClick={loadClash}>{t('重新读取')}</Button>
        </Field>
        <p className='detail mt-1.5'>{clashError ? t('读取失败：{error}', { error: clashError })
          : clash === null ? t('正在读取 Clash Verge 配置…')
            : clash.available === false ? t('不可用：{error}', { error: clash.error || t('未检测到 Clash Verge 配置') })
              : t('读取自 {dir}；端口由 Clash Verge 管理，这里实时同步', { dir: clash.dir || 'Clash Verge' })}</p>
        <p className='detail mt-1.5'>{t('既有 Clash Verge 直接引用继续可用；也可在「网络代理」页同步端点，再改选「已保存的代理」。')}</p>
      </div> : null}

      {draft.mode === 'custom' ? (
        <div className='mt-3'>
          <FieldGroup orientation='horizontal' className='field-row'>
            <Field className='contents'>
              <FieldLabel htmlFor={`${idPrefix}-protocol`}>{t('协议')}</FieldLabel>
              <Select value={draft.protocol} onValueChange={value => set({ protocol: value as 'http' | 'socks5' })}>
                <SelectTrigger id={`${idPrefix}-protocol`} className='min-w-[110px]'>
                  <SelectValue>{draft.protocol === 'socks5' ? 'SOCKS5' : 'HTTP'}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='http'>HTTP</SelectItem>
                  <SelectItem value='socks5'>SOCKS5</SelectItem>
                </SelectContent>
              </Select>
            </Field>
            <Field className='contents'>
              <FieldLabel htmlFor={`${idPrefix}-host`} className='ml-2'>{t('主机')}</FieldLabel>
              <Input id={`${idPrefix}-host`} className='min-w-[140px]' placeholder='127.0.0.1'
                value={draft.host} onChange={event => set({ host: event.currentTarget.value })} />
            </Field>
            <Field className='contents'>
              <FieldLabel htmlFor={`${idPrefix}-port`}>{t('端口')}</FieldLabel>
              <Input id={`${idPrefix}-port`} type='number' min={1} max={65535} className='max-w-[110px]'
                placeholder='7890' value={draft.port} onChange={event => set({ port: event.currentTarget.value })} />
            </Field>
          </FieldGroup>
          <FieldGroup orientation='horizontal' className='field-row mt-2.5'>
            <Field className='contents'>
              <FieldLabel htmlFor={`${idPrefix}-user`}>{t('用户名')}</FieldLabel>
              <Input id={`${idPrefix}-user`} placeholder={t('可选')} autoComplete='off'
                value={draft.username} onChange={event => set({ username: event.currentTarget.value })} />
            </Field>
            <Field className='contents'>
              <FieldLabel htmlFor={`${idPrefix}-pass`}>{t('密码')}</FieldLabel>
              <Input id={`${idPrefix}-pass`} type='password' placeholder={t('可选')} autoComplete='new-password'
                value={draft.password} onChange={event => set({ password: event.currentTarget.value })} />
            </Field>
          </FieldGroup>
        </div>
      ) : null}

      <FieldGroup orientation='horizontal' className='field-row mt-3'>
        <Button variant='outline' size='sm' disabled={testing} onClick={() => void test()}>
          {testing ? t('测试中…') : t('测试出口')}
        </Button>
        <span className='detail'>{testResult}</span>
      </FieldGroup>
    </div>
  )
}
