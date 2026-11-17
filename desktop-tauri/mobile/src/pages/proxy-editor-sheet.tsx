import { useEffect, useRef, useState } from 'react'
import {
  Button, Checkbox, Input, Label, Select, SelectContent, SelectItem, SelectTrigger,
  SelectValue, Sheet, SheetBody, SheetContent, SheetFooter, SheetHeader, SheetTitle, Switch, SegmentedControl,
} from '@ui'
import {
  fetchProxySettings, importProxyItems, saveProxyItem, saveProxySettings, testProxyDraft,
  type PoolItem, type PoolPayload, type ProxyDraft,
} from '../api/gateway'
import { serverIdentity } from '../lib/servers'
import { store } from '../state/store'
import { t } from '../i18n'
import { ProxyBatchEditor, type ProxyBatchHandle } from '../../../ui-shared/proxy-batch-editor'
import { proxyHost, type ProxyImportDraft, type ProxyImportReport } from '../../../ui-shared/proxy-import'

export function ProxyEditorSheet({ item, onClose, onSaved }: {
  item: PoolItem | null
  onClose: () => void
  onSaved: (data: PoolPayload) => void
}) {
  const [name, setName] = useState(item?.name || '')
  const [protocol, setProtocol] = useState(item?.protocol === 'socks5' ? 'socks5' : 'http')
  const [host, setHost] = useState(item?.host || '')
  const [port, setPort] = useState(item?.port ? String(item.port) : '')
  const [username, setUsername] = useState(item?.username || '')
  const [password, setPassword] = useState(item?.password || '')
  const [useAuth, setUseAuth] = useState(Boolean(item?.username || item?.password))
  const [enabled, setEnabled] = useState(item?.enabled !== false)
  const [nonDirect, setNonDirect] = useState(item?.nonDirect === true)
  const [mode, setMode] = useState<'single' | 'batch'>('single')
  const [batchBusy, setBatchBusy] = useState(false)
  const batch = useRef<ProxyBatchHandle>(null)
  const [busy, setBusy] = useState<'save' | 'test' | ''>('')
  const [error, setError] = useState('')
  const [result, setResult] = useState('')
  const identity = useRef(serverIdentity()).current
  const locked = Boolean(busy) || batchBusy
  useEffect(() => { setError(''); setResult('') }, [name, protocol, host, port, username, password, useAuth, nonDirect])

  async function saveBatch(items: ProxyImportDraft[]): Promise<ProxyImportReport> {
    if (serverIdentity() !== identity) throw new Error(t('The gateway has changed. Reopen the proxy dialog.'))
    const data = await importProxyItems(items)
    if (serverIdentity() !== identity) throw new Error(t('The gateway has changed. Reopen the proxy dialog.'))
    if (!Array.isArray(data.items) || !data.imported) throw new Error(t('Unexpected batch response. Refresh the proxy list to check what was saved before retrying.'))
    onSaved(data)
    return data.imported
  }

  async function testBatch(draft: ProxyImportDraft) {
    if (serverIdentity() !== identity) throw new Error(t('The gateway has changed. Reopen the proxy dialog.'))
    return testProxyDraft(draft)
  }

  function draft(): ProxyDraft {
    if (!name.trim()) throw new Error(t('Enter a proxy name'))
    if (useAuth && ([...username.trim()].length > 200 || [...password].length > 200)) throw new Error(t('Proxy usernames and passwords must not exceed 200 characters'))
    if (useAuth && protocol === 'socks5' && (new TextEncoder().encode(username.trim()).length > 255 || new TextEncoder().encode(password).length > 255)) {
      throw new Error(t('SOCKS5 usernames and passwords must not exceed 255 bytes'))
    }
    const hostText = proxyHost(host, t)
    const number = Number(port)
    if (!Number.isInteger(number) || number < 1 || number > 65535) throw new Error(t('Proxy port must be an integer from 1 to 65535'))
    return { name: name.trim(), protocol, host: hostText, port: number, username: useAuth ? username.trim() : '', password: useAuth ? password : '', enabled, nonDirect }
  }

  async function act(mode: 'save' | 'test'): Promise<void> {
    if (locked) return
    if (serverIdentity() !== identity) { setError(t('The gateway has changed. Reopen the proxy dialog.')); return }
    let payload: ProxyDraft
    try { payload = draft() } catch (cause) { setError(message(cause)); return }
    setBusy(mode)
    setError('')
    setResult('')
    try {
      if (mode === 'save') {
        const data = await saveProxyItem(payload, item?.id)
        if (serverIdentity() !== identity) return
        onSaved(data)
        store.toast(item ? t('Proxy changes saved') : t('Proxy added'))
        if (item || !batch.current?.hasDraft()) onClose()
        else { setName(''); setHost(''); setPort(''); setUsername(''); setPassword('') }
      } else {
        const data = await testProxyDraft(payload)
        if (serverIdentity() !== identity) return
        if (!data.success) setError(data.error || t('Connection failed'))
        else setResult(data.ip ? t('Exit IP {ip}', { ip: data.ip }) : t('Connected'))
      }
    } catch (cause) {
      if (serverIdentity() === identity) setError(message(cause))
    } finally { if (serverIdentity() === identity) setBusy('') }
  }

  return <Sheet open onOpenChange={(open, details) => {
    if (open) return
    if (locked) { details.cancel(); return }
    onClose()
  }}>
    <SheetContent>
      <SheetHeader><SheetTitle>{item ? t('Edit proxy') : t('New proxy')}</SheetTitle></SheetHeader>
      <SheetBody>
        {!item ? <SegmentedControl value={mode} onValueChange={setMode} disabled={locked} aria-label={t('Add mode')}
          options={[{ value: 'single', label: t('Add one') }, { value: 'batch', label: t('Add in bulk') }]} /> : null}
        <div className={mode === 'single' ? 'flex flex-col gap-4' : 'hidden'} inert={locked}>
        <Label className='flex flex-col gap-2'>{t('Proxy name')}<Input maxLength={60} value={name} onChange={event => setName(event.currentTarget.value)} placeholder={t('e.g. Hong Kong HTTP endpoint')} /></Label>
        <Label className='flex flex-col gap-2'>{t('Protocol')}
          <Select value={protocol} onValueChange={value => setProtocol(value || 'http')}>
            <SelectTrigger><SelectValue>{protocol.toUpperCase()}</SelectValue></SelectTrigger>
            <SelectContent><SelectItem value='http'>HTTP</SelectItem><SelectItem value='socks5'>SOCKS5</SelectItem></SelectContent>
          </Select>
        </Label>
        <div className='grid grid-cols-[minmax(0,1fr)_90px] gap-3'>
          <Label className='flex flex-col gap-2'>{t('Host')}<Input value={host} onChange={event => setHost(event.currentTarget.value)} placeholder='127.0.0.1' autoComplete='off' /></Label>
          <Label className='flex flex-col gap-2'>{t('Port')}<Input type='number' min={1} max={65535} value={port} onChange={event => setPort(event.currentTarget.value)} placeholder='8080' /></Label>
        </div>
        <p className='text-xs leading-relaxed text-subtle'>{t('The gateway device connects to this address. A local proxy must run on that device.')}</p>
        <Label className='flex items-center gap-2 text-xs font-normal'><Checkbox checked={useAuth} onCheckedChange={value => setUseAuth(value === true)} />{t('需要用户名和密码（可选）')}</Label>
        {useAuth ? <>
          <Label className='flex flex-col gap-2'>{t('Username')}<Input value={username} onChange={event => setUsername(event.currentTarget.value)} autoComplete='off' /></Label>
          <Label className='flex flex-col gap-2'>{t('Password')}<Input type='password' value={password} onChange={event => setPassword(event.currentTarget.value)} autoComplete='new-password' /></Label>
        </> : null}
        <Label className='flex items-center gap-2 text-xs font-normal'><Checkbox checked={nonDirect} onCheckedChange={setNonDirect} />{t('非直连代理')}</Label>
        <p className='text-xs leading-relaxed text-subtle'>{t('Connect through the gateway device\'s system proxy first, then through this proxy. If the system proxy is unavailable, requests will not fall back to a direct connection.')}</p>
        <div className='flex items-center justify-between'><Label>{t('Enable this proxy')}</Label><Switch checked={enabled} onCheckedChange={setEnabled} aria-label={t('Enable this proxy')} /></div>
        {error ? <p className='text-xs leading-relaxed text-destructive'>{error}</p> : null}
        {result ? <p className='text-xs text-success'>{result}</p> : null}
        </div>
        {!item ? <div className={mode === 'batch' ? '' : 'hidden'}>
          <ProxyBatchEditor ref={batch} t={t} locked={Boolean(busy)} onBusyChange={setBatchBusy} save={saveBatch} test={testBatch}
            onNotice={(message, failed) => { if (serverIdentity() === identity) store.toast(message, failed ? 'err' : 'ok') }} />
        </div> : null}
      </SheetBody>
      <SheetFooter>
        {mode === 'single' ? <>
          <Button variant='outline' disabled={locked} onClick={() => void act('test')}>{busy === 'test' ? t('Testing…') : t('Test connection')}</Button>
          <Button disabled={locked} onClick={() => void act('save')}>{busy === 'save' ? t('Saving…') : item ? t('Save proxy') : t('Add proxy')}</Button>
        </> : <Button variant='outline' disabled={locked} onClick={onClose}>{t('Close')}</Button>}
      </SheetFooter>
    </SheetContent>
  </Sheet>
}

export function ProxyTestSettingsSheet({ onClose }: { onClose: () => void }) {
  const [url, setUrl] = useState('')
  const [defaultUrl, setDefaultUrl] = useState('')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')
  const identity = serverIdentity()
  useEffect(() => {
    let alive = true
    void fetchProxySettings().then(data => {
      if (!alive || serverIdentity() !== identity) return
      setUrl(data.testUrl || '')
      setDefaultUrl(data.defaults?.testUrl || '')
    }).catch(cause => { if (alive && serverIdentity() === identity) setError(message(cause)) })
      .finally(() => { if (alive && serverIdentity() === identity) setLoading(false) })
    return () => { alive = false }
  }, [identity])

  async function save(): Promise<void> {
    if (loading || saving) return
    setSaving(true)
    setError('')
    try {
      await saveProxySettings(url.trim())
      if (serverIdentity() !== identity) return
      store.toast(t('✅ Connectivity test settings saved'))
      onClose()
    } catch (cause) { if (serverIdentity() === identity) setError(message(cause)) }
    finally { if (serverIdentity() === identity) setSaving(false) }
  }

  return <Sheet open onOpenChange={(open, details) => {
    if (open) return
    if (saving) { details.cancel(); return }
    onClose()
  }}>
    <SheetContent><SheetHeader><SheetTitle>{t('Connectivity test settings')}</SheetTitle></SheetHeader>
      <SheetBody>
        <Label className='flex flex-col gap-2'>{t('Test target URL')}<Input value={url} disabled={loading || saving} placeholder={defaultUrl} onChange={event => setUrl(event.currentTarget.value)} autoComplete='off' /></Label>
        <p className='text-xs leading-[1.8] text-subtle'>{t('Access this URL through the proxy to check for an HTTP response. Leave it empty to use the gateway default.')}</p>
        <Button variant='outline' disabled={loading || saving} onClick={() => setUrl('')}>{t('Restore Defaults')}</Button>
        {error ? <p className='text-xs text-destructive'>{error}</p> : null}
      </SheetBody><SheetFooter><Button disabled={loading || saving} onClick={() => void save()}>{saving ? t('Saving…') : t('Save settings')}</Button></SheetFooter>
    </SheetContent>
  </Sheet>
}

function message(cause: unknown): string { return cause instanceof Error ? cause.message : String(cause) }
