import * as React from 'react'
import {
  Button,
  Checkbox,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from '@ui'
import {
  durationText, errorMessage, loadSettings, saveItem, saveSettings, toast, wb,
  type ProxyPoolItem,
} from './proxies-state'
import { t } from '../i18n'
// 赞助位外链：与文档页/更新日志同一条路（data-external 委托 + 系统浏览器打开）。
// 链接本体在 sponsor.ts（唯一定义点），这里只取用。
import { handleExternalClick } from './update-shared'
import { PROXYLANE_URL } from './sponsor'
import { ProxyBatchEditor, type ProxyBatchHandle } from '../../../ui-shared/proxy-batch-editor'
import { proxyHost, type ProxyImportDraft, type ProxyImportReport } from '../../../ui-shared/proxy-import'

/* ─── 新增 / 编辑弹窗 ───────────────────────── */

/**
 * 赞助位里的行内链接：一处样式、一条链接（同一行的品牌名与「去获取」都用它）。
 * 「链接只裹需要点的字」是全站赞助位的口径 —— 句子按片段拼，翻译时片段可以
 * 各自贴合目标语言的语序。
 */
function SponsorLink({ children }: { children: React.ReactNode }) {
  return (
    <a
      className='font-semibold text-primary-fg no-underline hover:underline'
      href={PROXYLANE_URL}
      data-external={PROXYLANE_URL}
      onClick={handleExternalClick}
    >{children}</a>
  )
}

export function ProxyModal({
  item,
  onClose,
}: {
  item: ProxyPoolItem | null
  onClose: () => void
}) {
  const editing = item !== null
  const [name, setName] = React.useState(item?.name || '')
  const [protocol, setProtocol] = React.useState<'http' | 'socks5'>(item?.protocol === 'socks5' ? 'socks5' : 'http')
  const [host, setHost] = React.useState(item?.host || '')
  const [port, setPort] = React.useState(item?.port ? String(item.port) : '')
  const [username, setUsername] = React.useState(item?.username || '')
  const [password, setPassword] = React.useState(item?.password || '')
  const [enabled, setEnabled] = React.useState(item ? item.enabled : true)
  const [nonDirect, setNonDirect] = React.useState(item?.nonDirect === true)
  const [batchBusy, setBatchBusy] = React.useState(false)
  const batch = React.useRef<ProxyBatchHandle>(null)
  const [useAuth, setUseAuth] = React.useState(Boolean(item?.username || item?.password))
  const [saving, setSaving] = React.useState(false)
  const [testing, setTesting] = React.useState(false)
  const [status, setStatus] = React.useState<React.ReactNode>(null)
  const busy = saving || testing || batchBusy

  React.useEffect(() => { setStatus(null) }, [name, protocol, host, port, username, password, useAuth, nonDirect])

  async function saveBatch(items: ProxyImportDraft[]): Promise<ProxyImportReport> {
    const data = await saveItem(null, { items })
    if (!data?.imported) throw new Error(t('Unexpected batch response. Refresh the proxy list to check what was saved before retrying.'))
    return data.imported
  }

  async function testBatch(draft: ProxyImportDraft) {
    const data = await wb().workbuddyDesktop?.testProxy?.({ proxy: { ...draft, source: 'custom' } })
    if (!data) throw new Error(t('Proxy testing is not supported in this environment (bridge method missing)'))
    return data
  }

  /** 表单 → 归一化后的提交对象；形状非法时返回一句面向用户的错误 */
  function draftPayload(): { payload: Record<string, unknown> } | { error: string } {
    const trimmed = name.trim()
    if (!trimmed) return { error: t('Enter a proxy name') }
    if (useAuth && ([...username.trim()].length > 200 || [...password].length > 200)) {
      return { error: t('Proxy usernames and passwords must not exceed 200 characters') }
    }
    if (useAuth && protocol === 'socks5' && (new TextEncoder().encode(username.trim()).length > 255 || new TextEncoder().encode(password).length > 255)) {
      return { error: t('SOCKS5 usernames and passwords must not exceed 255 bytes') }
    }
    let hostText: string
    try { hostText = proxyHost(host, t) }
    catch (error) { return { error: errorMessage(error) } }
    const portNumber = Number(port)
    if (!Number.isInteger(portNumber) || portNumber < 1 || portNumber > 65535) {
      return { error: t('Proxy port must be an integer from 1 to 65535') }
    }
    return {
      payload: {
        name: trimmed,
        source: 'manual',
        protocol,
        host: hostText,
        port: portNumber,
        username: useAuth ? username.trim() : '',
        password: useAuth ? password : '',
        enabled,
        nonDirect,
      },
    }
  }

  /** 用当前表单内容测一次（**未保存也能测**：先确认能通，再决定保存） */
  async function testDraft(): Promise<void> {
    if (busy) return
    const draft = draftPayload()
    if ('error' in draft) {
      setStatus(<span className='text-destructive'>❌ {draft.error}</span>)
      return
    }
    const payload = draft.payload
    const proxy = {
      source: 'custom',
      protocol: payload.protocol,
      host: payload.host,
      port: payload.port,
      username: payload.username,
      password: payload.password,
      nonDirect: payload.nonDirect,
    }
    setTesting(true)
    setStatus(t('Connecting to upstream…'))
    try {
      const data = await wb().workbuddyDesktop?.testProxy?.({ proxy })
      if (data?.success) {
        const suffix = durationText(data.durationMs)
        setStatus(
          <span className='text-success'>
            {t('✅ Exit available')}
            {data.ip ? t(' Exit IP {ip}', { ip: data.ip }) : ''}
            {suffix ? `　${suffix}` : ''}
          </span>,
        )
      } else {
        setStatus(<span className='text-destructive'>❌ {data?.error || t('Connection failed')}</span>)
      }
    } catch (error) {
      setStatus(<span className='text-destructive'>❌ {errorMessage(error)}</span>)
    } finally {
      setTesting(false)
    }
  }

  async function save(): Promise<void> {
    if (busy) return
    const draft = draftPayload()
    if ('error' in draft) {
      setStatus(<span className='text-destructive'>❌ {draft.error}</span>)
      return
    }
    setSaving(true)
    setStatus(null)
    try {
      await saveItem(item, draft.payload)
      toast(editing ? t('Saved "{name}"', { name: name.trim() }) : t('Added "{name}"', { name: name.trim() }))
      if (editing || !batch.current?.hasDraft()) onClose()
      else { setName(''); setHost(''); setPort(''); setUsername(''); setPassword(''); setStatus(null) }
    } catch (error) {
      setStatus(<span className='text-destructive'>❌ {errorMessage(error)}</span>)
    } finally { setSaving(false) }
  }

  return (
    <Dialog open onOpenChange={(open, details) => {
      if (open) return
      if (busy) { details.cancel(); return }
      onClose()
    }}>
      <DialogContent className={editing ? 'max-w-[560px]' : 'w-[min(1120px,calc(100vw-48px))]'}>
        <DialogHeader>
          <DialogTitle>{editing ? t('Edit proxy') : t('New proxy')}</DialogTitle>
        </DialogHeader>
        <DialogBody className={editing ? 'flex flex-col gap-3.5' : 'grid gap-6 lg:grid-cols-[minmax(300px,0.8fr)_minmax(0,1.2fr)]'}>
          <div className='flex min-w-0 flex-col gap-3.5' inert={busy}>
          {!editing ? <h3 className='text-sm font-semibold'>{t('Add one')}</h3> : null}
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='proxy-name'>{t('Name')}</Label>
            <Input id='proxy-name' maxLength={60} autoComplete='off' spellCheck={false}
              placeholder={t('e.g. Hong Kong HTTP endpoint')}
              value={name} onChange={event => setName(event.currentTarget.value)} />
          </div>

          <div className='flex flex-row items-end gap-2.5'>
            <div className='flex flex-col gap-1.5'>
              <Label htmlFor='proxy-protocol'>{t('Protocol')}</Label>
              {/* 协议名 HTTP / SOCKS5 是固定叫法，不进词典 */}
              <Select value={protocol} onValueChange={value => setProtocol(value as 'http' | 'socks5')}>
                <SelectTrigger id='proxy-protocol' className='min-w-[110px]'>
                  <SelectValue>{protocol === 'socks5' ? 'SOCKS5' : 'HTTP'}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='http'>HTTP</SelectItem>
                  <SelectItem value='socks5'>SOCKS5</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className='flex min-w-0 flex-1 flex-col gap-1.5'>
              <Label htmlFor='proxy-host'>{t('Host')}</Label>
              <Input id='proxy-host' placeholder='127.0.0.1' autoComplete='off' spellCheck={false}
                value={host} onChange={event => setHost(event.currentTarget.value)} />
            </div>
            <div className='flex flex-col gap-1.5'>
              <Label htmlFor='proxy-port'>{t('Port')}</Label>
              <Input id='proxy-port' type='number' min={1} max={65535} className='max-w-[110px]'
                placeholder='7890' value={port} onChange={event => setPort(event.currentTarget.value)} />
            </div>
          </div>
          <p className='text-xs leading-[1.8] text-subtle'>{t('The gateway device connects to this address. A local proxy must run on that device.')}</p>
          <Label className='flex cursor-pointer items-center gap-2 text-xs font-normal'>
            <Checkbox checked={useAuth} onCheckedChange={value => setUseAuth(value === true)} />
            {t('Username and password required (optional)')}
          </Label>
          {useAuth ? <div className='flex flex-row items-end gap-2.5'>
            <div className='flex min-w-0 flex-1 flex-col gap-1.5'>
              <Label htmlFor='proxy-user'>{t('Username')}</Label>
              <Input id='proxy-user' placeholder={t('Optional')} autoComplete='off'
                value={username} onChange={event => setUsername(event.currentTarget.value)} />
            </div>
            <div className='flex min-w-0 flex-1 flex-col gap-1.5'>
              <Label htmlFor='proxy-pass'>{t('Password')}</Label>
              <Input id='proxy-pass' type='password' placeholder={t('Optional')} autoComplete='new-password'
                value={password} onChange={event => setPassword(event.currentTarget.value)} />
            </div>
          </div> : null}

          <Label className='flex cursor-pointer items-center gap-2 text-xs font-normal'>
            <Checkbox checked={nonDirect} onCheckedChange={setNonDirect} />{t('Chain through system proxy')}
          </Label>
          <p className='text-xs leading-relaxed text-subtle'>{t('Connect through the gateway device\'s system proxy first, then through this proxy. If the system proxy is unavailable, requests will not fall back to a direct connection.')}</p>

          <div className='flex flex-row items-center gap-2.5'>
            <Switch checked={enabled} onCheckedChange={setEnabled} aria-label={t('Enable this proxy')} />
            <span className='text-[12.5px]'>
              {t('Enabled (disabling blocks requests from accounts using this proxy; testing remains available)')}
            </span>
          </div>

          {/* 赞助位：一行提示（另两处在 README 顶部与「文档」页的网络代理示例）。
              品牌名与句尾的「去获取」都是行内链接（SponsorLink），翻译时片段
              可以各自贴合目标语言的语序 */}
          <p className='text-xs leading-[1.65] text-subtle'>
            {t('Need a clean exit?')}
            <SponsorLink>ProxyLane</SponsorLink>
            {t(' native static residential IPs for AI accounts: low Scamalytics fraud score, unlimited traffic, 25% off your first purchase (AGENT2API25)')}
            {' '}
            <SponsorLink>{t('Get one')}</SponsorLink>
          </p>

          <div className='min-h-[18px] text-xs text-subtle' role='status'>{status}</div>
          {!editing ? <div className='mt-auto flex gap-2'>
            <Button variant='outline' disabled={busy} onClick={() => void testDraft()}>{testing ? t('Testing…') : t('Test exit')}</Button>
            <Button disabled={busy} onClick={() => void save()}>{saving ? t('Saving…') : t('Add proxy')}</Button>
          </div> : null}
          </div>
          {!editing ? <div className='min-w-0 border-t border-border pt-5 lg:border-t-0 lg:border-l lg:pt-0 lg:pl-6'>
            <ProxyBatchEditor ref={batch} t={t} locked={saving || testing} onBusyChange={setBatchBusy}
              save={saveBatch} test={testBatch} onNotice={(message, failed) => toast(message, failed ? 'err' : 'ok')} />
          </div> : null}
        </DialogBody>
        <DialogFooter>
          {editing ? <Button variant='outline' disabled={busy} onClick={() => void testDraft()}>
            {testing ? t('Testing…') : t('Test exit')}
          </Button> : null}
          <div className='mr-auto' />
          <Button variant='outline' disabled={busy} onClick={onClose}>{editing ? t('Cancel') : t('Close')}</Button>
          {editing ? <Button variant='default' disabled={busy} onClick={() => void save()}>
            {saving ? t('Saving…') : t('Save')}
          </Button> : null}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── 连通性测试设置 ───────────────────────── */

export function ProxySettingsModal({ onClose }: { onClose: () => void }) {
  const [loading, setLoading] = React.useState(true)
  const [saving, setSaving] = React.useState(false)
  const [testUrl, setTestUrl] = React.useState('')
  const [defaultUrl, setDefaultUrl] = React.useState('')
  const [error, setError] = React.useState('')

  React.useEffect(() => {
    let alive = true
    void loadSettings().then(data => {
      if (!alive) return
      if (!data) throw new Error(t('Unexpected proxy settings response'))
      setTestUrl(data.testUrl || '')
      setDefaultUrl(data.defaults?.testUrl || '')
    }).catch(cause => { if (alive) setError(errorMessage(cause)) })
      .finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [])

  async function save(): Promise<void> {
    if (saving || loading) return
    const url = testUrl.trim()
    if (url && !/^https?:\/\//i.test(url)) {
      setError(t('The test URL must start with http:// or https://'))
      return
    }
    setSaving(true)
    setError('')
    try {
      await saveSettings({ testUrl: url })
      toast(t('✅ Connectivity test settings saved'))
      onClose()
    } catch (cause) {
      setError(errorMessage(cause))
    } finally { setSaving(false) }
  }

  return <Dialog open onOpenChange={open => { if (!open && !saving) onClose() }}>
    <DialogContent className='max-w-[500px]'>
      <DialogHeader><DialogTitle>{t('Connectivity test settings')}</DialogTitle></DialogHeader>
      <DialogBody>
        <div className='flex flex-col gap-1.5'>
          <Label htmlFor='proxy-settings-url'>{t('Test target URL')}</Label>
          <Input id='proxy-settings-url' value={testUrl} placeholder={defaultUrl} autoComplete='off' spellCheck={false}
            disabled={loading || saving} className='font-mono text-xs'
            onChange={event => setTestUrl(event.currentTarget.value)} />
        </div>
        <p className='text-xs leading-[1.8] text-subtle'>
          {t('Access this URL through the proxy to check for an HTTP response. Leave it empty to use the gateway default.')}
        </p>
        {loading ? <p className='text-xs text-subtle'>{t('Loading…')}</p> : null}
        {error ? <p className='text-xs text-destructive'>❌ {error}</p> : null}
      </DialogBody>
      <DialogFooter>
        <Button variant='ghost' className='mr-auto' disabled={loading || saving} onClick={() => { setTestUrl(''); setError('') }}>{t('Restore Defaults')}</Button>
        <Button variant='outline' disabled={saving} onClick={onClose}>{t('Cancel')}</Button>
        <Button disabled={loading || saving} onClick={() => void save()}>{saving ? t('Saving…') : t('Save settings')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
