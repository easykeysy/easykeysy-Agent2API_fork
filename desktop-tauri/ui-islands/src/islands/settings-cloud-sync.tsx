import * as React from 'react'
import {
  Badge,
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  SegmentedControl,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Card, CardHeader, CardContent } from '@ui'
import { errorMessage, shared, toast } from './settings-model'
import { t } from '../i18n'
import { Alert } from '@ui'
import {
  emptyForm, formFromView, formatDateText, formatTime, payloadOf, PROVIDER_S3, PROVIDER_WEBDAV,
  S3_PRESETS, WEBDAV_PRESETS, type CloudSyncView, type Form, type RemoteInfo,
} from './settings-cloud-sync-model'

/**
 * 设置 → 数据 → 云同步（对标 cc-switch 的 WebDAV / S3 同步区块）。
 *
 * ── 为什么独立成文件、自带数据流 ────────────────────────────
 * settings-page.tsx 已经 2300 多行，而这一块与设置页的其它面板**没有状态交集**
 * （它自己读写 `/api/cloud-sync`）。与 `settings-prompt-editor.tsx` 同一先例：
 * 面板独立成文件、自己在挂载时取数，宿主只负责放一个 `<CloudSyncPanel />`。
 *
 * ── 表单为什么不复用 settings-state 的快照 ──────────────────
 * 那一层服务的是「读一次、整页共享」的设置项；本面板是**一份可编辑表单**
 * （用户改一半切走再回来、保存前不能丢），用局部 state 更直接。密码类字段
 * 按「留空 = 沿用」的语义处理 —— 这条判断在**后端**做（空串即沿用库里那份），
 * 前端不需要另存「是否触碰过」，少一个会与后端不一致的状态。
 *
 * ── 两种服务商共用一套流程 ──────────────────────────────────
 * 后端把两套配置都存在同一个 `cloudSync` 键里（切换不丢已填内容），所以这里
 * 只切表单的 `provider`，不隐藏 / 不互斥。测试、保存、上传、下载都只作用于
 * 当前选中的那一家。
 *
 * ── 下载是「替换」语义 ──────────────────────────────────────
 * 与整库恢复一致：本机业务数据会被远端快照整体替换（日志、请求记录、面板
 * 登录态不在同步范围，保持原样）。所以下载前必须有二次确认，并把「会先自动
 * 备份」写清楚。
 */

type Busy = 'load' | 'test' | 'save' | 'upload' | 'download' | 'remote' | 'key' | null
type DialogKind = 'upload' | 'download' | 'key' | null

export function CloudSyncPanel() {
  const bridge = shared().workbuddyDesktop
  const [view, setView] = React.useState<CloudSyncView | null>(null)
  const [form, setForm] = React.useState<Form>(emptyForm)
  const [dirty, setDirty] = React.useState(false)
  const [busy, setBusy] = React.useState<Busy>('load')
  const [loadError, setLoadError] = React.useState('')
  const loadingRef = React.useRef<Promise<void> | null>(null)
  const busyRef = React.useRef(busy)
  busyRef.current = busy
  const dirtyRef = React.useRef(dirty)
  dirtyRef.current = dirty
  const [remote, setRemote] = React.useState<RemoteInfo | null>(null)
  const [dialog, setDialog] = React.useState<DialogKind>(null)
  const [keyPassword, setKeyPassword] = React.useState('')

  const isWebDav = form.provider === PROVIDER_WEBDAV
  const webdavPreset = WEBDAV_PRESETS.find(item => item.baseUrl && form.webdavBaseUrl.includes(new URL(item.baseUrl).host))
  const s3Preset = S3_PRESETS.find(item => item.id !== 'custom' && item.label === 'Cloudflare R2' && form.s3Region === 'auto') ?? S3_PRESETS[0]

  /** 取一次设置并铺进表单（保留用户正在编辑的密码输入） */
  const load = React.useCallback((keepPassword = false, silent = false): Promise<void> => {
    if (loadingRef.current) return loadingRef.current
    const reading = (async () => {
      setBusy(current => current ?? 'load')
      try {
        const api = shared().workbuddyDesktop
        if (!api?.getCloudSync) throw new Error(t('Backend bridge unavailable'))
        const data = (await api.getCloudSync()) as CloudSyncView | null | undefined
        if (!data || !data.webdav || !data.s3) throw new Error(t('Incomplete config response'))
        setLoadError('')
        setView(data)
        if (!silent || !dirtyRef.current) {
          setForm(current => {
            const next = formFromView(data)
            if (keepPassword) {
              next.webdavPassword = current.webdavPassword
              next.s3SecretAccessKey = current.s3SecretAccessKey
              next.syncPassword = current.syncPassword
            }
            return next
          })
          setDirty(false)
        }
      } catch (error) {
        const message = errorMessage(error)
        setLoadError(message)
        if (!silent) toast(t('Failed to read cloud sync settings: {message}', { message }), 'err')
      } finally {
        setBusy(null)
      }
    })()
    loadingRef.current = reading
    void reading.finally(() => { if (loadingRef.current === reading) loadingRef.current = null })
    return reading
  }, [])

  React.useEffect(() => {
    const reload = () => {
      if (busyRef.current !== null && busyRef.current !== 'load') return
      void load(true, true)
    }
    reload()
    window.addEventListener('wb:settings-load', reload)
    const unsubscribe = shared().workbuddyDesktop?.onBackendReady?.(reload)
    return () => {
      window.removeEventListener('wb:settings-load', reload)
      if (typeof unsubscribe === 'function') unsubscribe()
    }
  }, [load])

  const patch = (changes: Partial<Form>) => {
    setForm(current => ({ ...current, ...changes }))
    setDirty(true)
  }

  const disabled = busy !== null

  /* ─── 动作 ─────────────────────────────────────────────── */

  const test = async () => {
    if (!bridge?.testCloudSync) return
    setBusy('test')
    try {
      const result = (await bridge.testCloudSync(payloadOf(form))) as { remotePath?: string } | null
      toast(t('✅ Connected: {path}', { path: result?.remotePath || '' }))
    } catch (error) {
      toast(t('Connection failed: {message}', { message: errorMessage(error) }), 'err')
    } finally {
      setBusy(null)
    }
  }

  const save = async () => {
    if (!bridge?.saveCloudSync) return
    setBusy('save')
    try {
      const data = (await bridge.saveCloudSync(payloadOf(form))) as CloudSyncView | null | undefined
      if (data) {
        setView(data)
        setForm(formFromView(data))
      }
      setDirty(false)
      toast(t('✅ Cloud sync settings saved'))
    } catch (error) {
      toast(t('Save failed: {message}', { message: errorMessage(error) }), 'err')
      return
    } finally {
      setBusy(null)
    }
    // 保存后自动测一次连接（与 cc-switch 同款：保存成功 ≠ 凭据可用）
    await test()
  }

  const fetchRemote = async (): Promise<RemoteInfo | null> => {
    if (!bridge?.getCloudSyncRemote) return null
    setBusy('remote')
    try {
      return (await bridge.getCloudSyncRemote()) as RemoteInfo
    } catch (error) {
      toast(t('Failed to read remote info: {message}', { message: errorMessage(error) }), 'err')
      return null
    } finally {
      setBusy(null)
    }
  }

  const beginUpload = async () => {
    if (dirty) {
      toast(t('Save the settings before syncing'), 'err')
      return
    }
    const info = await fetchRemote()
    if (!info) return
    setRemote(info.empty ? null : info)
    setDialog('upload')
  }

  const confirmUpload = async () => {
    if (!bridge?.cloudSyncUpload) return
    setDialog(null)
    setBusy('upload')
    try {
      const result = (await bridge.cloudSyncUpload()) as { bytes?: number; rows?: Record<string, number> } | null
      toast(t('✅ Uploaded to the cloud ({size})', { size: `${Math.round((Number(result?.bytes) || 0) / 1024)} KB` }))
      await load()
    } catch (error) {
      toast(t('Upload failed: {message}', { message: errorMessage(error) }), 'err')
    } finally {
      setBusy(null)
    }
  }

  const beginDownload = async () => {
    if (dirty) {
      toast(t('Save the settings before syncing'), 'err')
      return
    }
    const info = await fetchRemote()
    if (!info) return
    if (info.empty) {
      toast(t('No sync data available to download on the remote'))
      return
    }
    if (info.compatible === false) {
      toast(info.incompatibleReason || t('Remote data is incompatible with this version'), 'err')
      return
    }
    setRemote(info)
    if (info.keyRequired) {
      // 远端说用了另一把密钥，但文件却没传上来（旧版本上传 / 手删了文件）：
      // 这时输入密码也解不出来，直接说清楚，别让用户白输一次
      if (info.hasKeyArtifact === false) {
        toast(
          t('The remote snapshot uses a different account key but does not include the key file, so it cannot be decrypted here. Re-upload once from the original machine.'),
          'err'
        )
        return
      }
      setKeyPassword('')
      setDialog('key')
      return
    }
    setDialog('download')
  }

  const confirmDownload = async () => {
    if (!bridge?.cloudSyncDownload) return
    setDialog(null)
    setBusy('download')
    try {
      const result = (await bridge.cloudSyncDownload()) as { deviceName?: string; safetyBackup?: string } | null
      const note = result?.safetyBackup
        ? t(', the pre-replace data was backed up as {name}', { name: result.safetyBackup })
        : ''
      toast(t('✅ Restored from the cloud{note}', { note }))
      await load()
      // 账号、配置、签到 / 余额记录都换了 —— 主界面（账号列表、导航计数）要重绘
      await shared().wbApp?.refresh?.()
    } catch (error) {
      toast(t('Download failed: {message}', { message: errorMessage(error) }), 'err')
    } finally {
      setBusy(null)
    }
  }

  const importKey = async () => {
    if (!bridge?.importCloudSyncKey) return
    if (!keyPassword) {
      toast(t('Enter the sync password'), 'err')
      return
    }
    setBusy('key')
    try {
      await bridge.importCloudSyncKey(keyPassword)
      setKeyPassword('')
      toast(t('✅ Account key imported; continue with the download'))
      const info = await fetchRemote()
      setDialog(null)
      if (info && !info.empty) {
        setRemote(info)
        setDialog('download')
      }
    } catch (error) {
      toast(t('Key import failed: {message}', { message: errorMessage(error) }), 'err')
    } finally {
      setBusy(null)
    }
  }

  /* ─── 渲染 ─────────────────────────────────────────────── */

  const status = view?.status || {}
  const showAutoError = Boolean(status.lastError) && status.lastErrorSource === 'auto'
  const lastSync = formatTime(status.lastSyncAt)

  return (
    <>
      <Card render={<section />} className='panel'>
        <CardHeader className='panel-head'>
          <h2>{t('Cloud sync')}</h2>
          <span
            className='tip-q'
            data-tip={t(
              'Sync accounts, gateway configuration, custom providers, Agent profiles, proxy pool / subscriptions, and check-in and balance records to WebDAV or S3-compatible storage, then pull them back on another machine in one click. Event logs, request details, debug payloads and panel login state are out of scope (they are this machine\'s runtime traces). Uploaded account credentials stay encrypted; the account key that decrypts them is itself encrypted with the sync password and uploaded alongside — the sync password is never stored in the cloud, so you must re-enter it when restoring on another machine.'
            )}
          ></span>
          <Badge variant={loadError ? 'destructive' : view?.enabled ? 'success' : 'outline'}>
            {!view && busy === 'load' ? t('Loading…') : loadError ? t('Unavailable') : view?.enabled ? t('Enabled') : t('Disabled')}
          </Badge>
          <div className='head-actions'>
            <Button variant='outline' disabled={disabled} onClick={() => void load(true)}>
              {t('Refresh')}
            </Button>
          </div>
        </CardHeader>
        <CardContent className='panel-body'>
          {loadError ? (
            <div className='hint' role='status'>
              {t('Could not read cloud sync settings. Please try again later.')}<br />{loadError}
            </div>
          ) : (
            <>
              <div className='retention-list'>
                <div className='retention-row'>
                  <Label>{t('Provider')}</Label>
                  <div className='routing-row'>
                    <SegmentedControl
                      options={[
                        { value: PROVIDER_WEBDAV, label: 'WebDAV' },
                        { value: PROVIDER_S3, label: t('S3-compatible storage') },
                      ]}
                      value={form.provider}
                      disabled={disabled}
                      aria-label={t('Provider')}
                      onValueChange={value => patch({ provider: String(value) })}
                    />
                  </div>
                </div>

                {isWebDav ? (
                  <>
                    <div className='retention-row'>
                      <Label>{t('Preset')}</Label>
                      <div className='routing-row'>
                        <Select
                          value={webdavPreset?.id || 'custom'}
                          disabled={disabled}
                          onValueChange={value => {
                            const preset = WEBDAV_PRESETS.find(item => item.id === value)
                            if (preset?.baseUrl) patch({ webdavBaseUrl: preset.baseUrl })
                          }}
                        >
                          <SelectTrigger className='min-w-[220px]'>
                            <SelectValue>{webdavPreset?.label || t('Custom')}</SelectValue>
                          </SelectTrigger>
                          <SelectContent>
                            {WEBDAV_PRESETS.map(item => (
                              <SelectItem key={item.id} value={item.id}>
                                {item.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-webdav-url'>{t('Server URL')}</Label>
                      <Input
                        id='cloud-webdav-url'
                        value={form.webdavBaseUrl}
                        disabled={disabled}
                        placeholder='https://dav.example.com/dav/'
                        onChange={event => patch({ webdavBaseUrl: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-webdav-user'>{t('Username')}</Label>
                      <Input
                        id='cloud-webdav-user'
                        value={form.webdavUsername}
                        disabled={disabled}
                        autoComplete='off'
                        onChange={event => patch({ webdavUsername: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-webdav-password'>{t('Password')}</Label>
                      <Input
                        id='cloud-webdav-password'
                        type='password'
                        value={form.webdavPassword}
                        disabled={disabled}
                        autoComplete='off'
                        placeholder={view?.webdav.hasPassword ? t('Saved; leave blank to keep') : t('App-specific password')}
                        onChange={event => patch({ webdavPassword: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-webdav-ua'>{t('Custom User-Agent')}</Label>
                      <Input
                        id='cloud-webdav-ua'
                        value={form.webdavUserAgent}
                        disabled={disabled}
                        autoComplete='off'
                        placeholder={t('Leave blank to use the default')}
                        onChange={event => patch({ webdavUserAgent: event.target.value })}
                      />
                      <div className='hint'>
                        {t(
                          'Optional. Some services (such as CSTCloud Data Capsule) validate the credential\'s bound client type via the User-Agent; if so, enter a string containing that type name (for example rclone/v1.67.0). Leave blank to keep the default.'
                        )}
                      </div>
                    </div>
                    {webdavPreset?.hint ? <div className='hint'>{webdavPreset.hint}</div> : null}
                  </>
                ) : (
                  <>
                    <div className='retention-row'>
                      <Label>{t('Preset')}</Label>
                      <div className='routing-row'>
                        <Select
                          value={s3Preset.id}
                          disabled={disabled}
                          onValueChange={value => {
                            const preset = S3_PRESETS.find(item => item.id === value)
                            if (preset) patch({ s3Region: preset.regionPlaceholder })
                          }}
                        >
                          <SelectTrigger className='min-w-[220px]'>
                            <SelectValue>{s3Preset.label}</SelectValue>
                          </SelectTrigger>
                          <SelectContent>
                            {S3_PRESETS.map(item => (
                              <SelectItem key={item.id} value={item.id}>
                                {item.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-s3-region'>{t('Region')}</Label>
                      <Input
                        id='cloud-s3-region'
                        value={form.s3Region}
                        disabled={disabled}
                        placeholder={s3Preset.regionPlaceholder}
                        onChange={event => patch({ s3Region: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-s3-bucket'>{t('Bucket')}</Label>
                      <Input
                        id='cloud-s3-bucket'
                        value={form.s3Bucket}
                        disabled={disabled}
                        onChange={event => patch({ s3Bucket: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-s3-key'>{t('Access Key ID')}</Label>
                      <Input
                        id='cloud-s3-key'
                        value={form.s3AccessKeyId}
                        disabled={disabled}
                        autoComplete='off'
                        onChange={event => patch({ s3AccessKeyId: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-s3-secret'>{t('Secret Access Key')}</Label>
                      <Input
                        id='cloud-s3-secret'
                        type='password'
                        value={form.s3SecretAccessKey}
                        disabled={disabled}
                        autoComplete='off'
                        placeholder={view?.s3.hasSecretKey ? t('Saved; leave blank to keep') : ''}
                        onChange={event => patch({ s3SecretAccessKey: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-s3-endpoint'>{t('Endpoint')}</Label>
                      <Input
                        id='cloud-s3-endpoint'
                        value={form.s3Endpoint}
                        disabled={disabled}
                        placeholder={t('Leave blank to use the official AWS address')}
                        onChange={event => patch({ s3Endpoint: event.target.value })}
                      />
                    </div>
                    <div className='retention-row'>
                      <Label htmlFor='cloud-s3-ua'>{t('Custom User-Agent')}</Label>
                      <Input
                        id='cloud-s3-ua'
                        value={form.s3UserAgent}
                        disabled={disabled}
                        autoComplete='off'
                        placeholder={t('Leave blank to use the default')}
                        onChange={event => patch({ s3UserAgent: event.target.value })}
                      />
                      <div className='hint'>
                        {t(
                          'Optional. Some services (such as CSTCloud Data Capsule) validate the credential\'s bound client type via the User-Agent; if so, enter a string containing that type name (for example rclone/v1.67.0). Leave blank to keep the default.'
                        )}
                      </div>
                    </div>
                    {s3Preset.hint ? <div className='hint'>{s3Preset.hint}</div> : null}
                  </>
                )}

                <div className='retention-row'>
                  <Label htmlFor='cloud-remote-root'>{t('Remote root directory')}</Label>
                  <Input
                    id='cloud-remote-root'
                    value={form.remoteRoot}
                    disabled={disabled}
                    placeholder='agent2api-sync'
                    onChange={event => patch({ remoteRoot: event.target.value })}
                  />
                </div>
                <div className='retention-row'>
                  <Label htmlFor='cloud-profile'>{t('Profile')}</Label>
                  <Input
                    id='cloud-profile'
                    value={form.profile}
                    disabled={disabled}
                    placeholder='default'
                    onChange={event => patch({ profile: event.target.value })}
                  />
                </div>
                <div className='retention-row'>
                  <Label htmlFor='cloud-sync-password'>{t('Sync password')}</Label>
                  <Input
                    id='cloud-sync-password'
                    type='password'
                    value={form.syncPassword}
                    disabled={disabled}
                    autoComplete='new-password'
                    placeholder={view?.hasSyncPassword ? t('Saved; leave blank to keep') : t('At least 8 characters; needed when restoring on another machine')}
                    onChange={event => patch({ syncPassword: event.target.value })}
                  />
                  <div className='hint'>
                    {t(
                      'Used to encrypt the account key that goes to the cloud. Remember it — it is not stored in the cloud, and you must enter the same password when restoring on another machine, otherwise the account data cannot be decrypted.'
                    )}
                  </div>
                </div>
                <div className='retention-row'>
                  <Label htmlFor='cloud-auto-sync'>{t('Auto sync')}</Label>
                  <Switch
                    id='cloud-auto-sync'
                    checked={form.autoSync}
                    disabled={disabled || !form.enabled}
                    onCheckedChange={checked => patch({ autoSync: checked })}
                  />
                  <div className='hint'>
                    {t('Upload only when account data, settings, check-in results or balances change. Changes to export or query timestamps alone do not trigger an upload.')}
                  </div>
                </div>
                <div className='retention-row'>
                  <Label htmlFor='cloud-auto-sync-interval'>{t('Minimum automatic upload interval')}</Label>
                  <Select value={String(form.autoSyncIntervalMinutes)} disabled={disabled || !form.autoSync || !form.enabled}
                    onValueChange={value => { if (value !== null) patch({ autoSyncIntervalMinutes: Number(value) }) }}>
                    <SelectTrigger id='cloud-auto-sync-interval' className='min-w-[220px]'>
                      <SelectValue>{t('{minutes} minutes', { minutes: form.autoSyncIntervalMinutes })}</SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {[...new Set([5, 15, 30, 60, form.autoSyncIntervalMinutes])].sort((a, b) => a - b).map(minutes => (
                        <SelectItem key={minutes} value={String(minutes)}>{t('{minutes} minutes', { minutes })}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <div className='hint'>{t('Defaults to 5 minutes. Changes during the interval are merged into the latest snapshot and uploaded when it is due. Manual uploads are not limited. Failed attempts are retried automatically with a longer delay.')}</div>
                </div>
                <div className='retention-row'>
                  <Label htmlFor='cloud-enabled'>{t('Enable cloud sync')}</Label>
                  <Switch
                    id='cloud-enabled'
                    checked={form.enabled}
                    disabled={disabled}
                    onCheckedChange={checked =>
                      patch({ enabled: checked, autoSync: checked ? form.autoSync : false })
                    }
                  />
                  <div className='hint'>{t('When off, manual upload/download is rejected too.')}</div>
                </div>
              </div>

              {dirty ? (
                <div className='hint'>{t('Unsaved changes: save first to sync (testing the connection is unaffected).')}</div>
              ) : null}

              <div className='head-actions mt-3 flex flex-wrap gap-2'>
                <Button variant='outline' disabled={disabled} onClick={() => void test()}>
                  {busy === 'test' ? t('Testing…') : t('Test connection')}
                </Button>
                <Button variant='outline' disabled={disabled || !dirty} onClick={() => void save()}>
                  {busy === 'save' ? t('Saving…') : t('Save')}
                </Button>
                <Button
                  disabled={disabled || dirty || !view?.enabled}
                  onClick={() => void beginUpload()}
                >
                  {busy === 'upload' ? t('Uploading…') : busy === 'remote' ? t('Reading remote…') : t('Upload to cloud')}
                </Button>
                <Button
                  variant='secondary'
                  disabled={disabled || dirty || !view?.enabled}
                  onClick={() => void beginDownload()}
                >
                  {busy === 'download' ? t('Downloading…') : t('Restore from cloud')}
                </Button>
              </div>

              <div className='hint'>
                {lastSync
                  ? t('Last sync: {time} ({action})', {
                      time: lastSync,
                      action: status.lastAction === 'download' ? t('Download') : t('Upload'),
                    })
                  : t('Not synced yet.')}
                {view?.remotePath ? ` ${t('Remote path: {path}', { path: view.remotePath })}` : ''}
              </div>

              {showAutoError ? (
                <Alert variant='destructive' className='danger-zone' id='cloud-sync-auto-error'>
                  <strong>{t('Last auto sync failed')}</strong>
                  <br />
                  {status.lastError}
                  <br />
                  {t('Your changes remain saved locally. The system will retry automatically with a longer delay. After fixing the network or credentials, you can also select "Upload to cloud" manually.')}
                  {status.retryAt ? <div>{t('Next automatic retry: {time}', { time: formatTime(status.retryAt) })}</div> : null}
                </Alert>
              ) : null}

              <Alert variant='destructive' className='danger-zone'>
                <strong>{t('The upload contains encrypted account credentials and gateway configuration')}</strong>
                {t(
                  ', plus any proxy credentials that may exist in the proxy pool / subscriptions. Use private storage only you can access; the account key is encrypted separately with the sync password, so leaking the storage alone does not expose your accounts.'
                )}
              </Alert>
            </>
          )}
        </CardContent>
      </Card>

      {/* ─── 上传确认 ─── */}
      <Dialog open={dialog === 'upload'} onOpenChange={next => { if (!next) setDialog(null) }}>
        <DialogContent className='w-[min(460px,calc(100vw-48px))]'>
          <DialogHeader>
            <DialogTitle>{t('Upload to cloud')}</DialogTitle>
          </DialogHeader>
          <DialogBody>
            <div>
              {t('This machine\'s business data (accounts, configuration, Agent profiles, proxy pool / subscriptions, check-in and balance records) will be uploaded and')}
              <strong>{t('overwrite')}</strong>
              {t('the old snapshot at the same remote path. Event logs, request details and debug payloads are not uploaded.')}
            </div>
            <div className='hint mt-2'>
              {t('Target path: {path}', { path: view?.remotePath || '' })}
            </div>
            {remote ? (
              <Alert variant='destructive' className='danger-zone mt-3'>
                <strong>{t('A snapshot already exists on the remote and will be overwritten')}</strong>
                <br />
                {t('From {device}, created at {time}', {
                  device: remote.deviceName || t('Unknown device'),
                  time: formatDateText(remote.createdAt),
                })}
              </Alert>
            ) : (
              <div className='hint mt-3'>{t('No snapshot exists at that remote path yet; this is the first upload.')}</div>
            )}
          </DialogBody>
          <DialogFooter>
            <Button variant='outline' onClick={() => setDialog(null)}>
              {t('Cancel')}
            </Button>
            <Button onClick={() => void confirmUpload()}>{t('Confirm upload')}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ─── 下载确认 ─── */}
      <Dialog open={dialog === 'download'} onOpenChange={next => { if (!next) setDialog(null) }}>
        <DialogContent className='w-[min(480px,calc(100vw-48px))]'>
          <DialogHeader>
            <DialogTitle>{t('Restore from cloud')}</DialogTitle>
          </DialogHeader>
          <DialogBody>
            <Alert variant='destructive' className='danger-zone'>
              {t('This machine\'s accounts, gateway configuration, Agent profiles, proxy pool / subscriptions, and check-in and balance records will be replaced by the remote snapshot')}
              <strong>{t('wholesale-replaced')}</strong>
              {t('. Event logs, request details, debug payloads and panel login state stay untouched. The current business data is backed up to backups/ under the config directory before replacing.')}
            </Alert>
            <div className='mt-3 text-[12.5px]'>
              <div>{t('Source device: {device}', { device: remote?.deviceName || t('Unknown device') })}</div>
              <div>{t('Created at: {time}', { time: formatDateText(remote?.createdAt) })}</div>
              <div>{t('App version: {version}', { version: remote?.appVersion || '—' })}</div>
              <div>{t('Includes: {items}', { items: (remote?.artifacts || []).join(', ') || '—' })}</div>
            </div>
          </DialogBody>
          <DialogFooter>
            <Button variant='outline' onClick={() => setDialog(null)}>
              {t('Cancel')}
            </Button>
            <Button variant='destructive' onClick={() => void confirmDownload()}>
              {t('Replace and restore')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ─── 同步密码（远端密钥与本机不一致时） ─── */}
      <Dialog open={dialog === 'key'} onOpenChange={next => { if (!next) setDialog(null) }}>
        <DialogContent className='w-[min(440px,calc(100vw-48px))]'>
          <DialogHeader>
            <DialogTitle>{t('Enter sync password')}</DialogTitle>
          </DialogHeader>
          <DialogBody>
            <div>
              {t(
                'The remote snapshot uses a different account key than this machine. Enter the sync password set on the uploading machine to decrypt the key and install it here, then continue the download.'
              )}
            </div>
            <div className='hint mt-2'>
              {t('The local account.key is renamed to a backup (account.key.bak-<timestamp>) before importing; it is never deleted.')}
            </div>
            <Label htmlFor='cloud-sync-key-password' className='mt-3 mb-[5px] block text-[12.5px]'>
              {t('Sync password')}
            </Label>
            <Input
              id='cloud-sync-key-password'
              type='password'
              value={keyPassword}
              autoComplete='off'
              onChange={event => setKeyPassword(event.target.value)}
              onKeyDown={event => {
                if (event.key === 'Enter') void importKey()
              }}
            />
          </DialogBody>
          <DialogFooter>
            <Button variant='outline' onClick={() => setDialog(null)}>
              {t('Cancel')}
            </Button>
            <Button disabled={busy === 'key'} onClick={() => void importKey()}>
              {busy === 'key' ? t('Importing…') : t('Import key and continue')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
