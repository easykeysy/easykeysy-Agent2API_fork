import { Progress as UiProgress } from '@ui'
import { useEffect, useRef, useState } from 'react'
import { SectionHead } from './ui'
import { Icon } from './Icon'
import { store } from '../state/store'
import {
  APP_VERSION,
  autoCheckEnabled,
  autoCheckIntervalMs,
  canInstallApk,
  canSelfUpdate,
  cancelDownload,
  checkApkUpdate,
  downloadApk,
  hasDownloadedApk,
  installApk,
  openInstallPermission,
  setAutoCheckEnabled,
  type ApkUpdateInfo,
  type DownloadProgress,
} from '../lib/appUpdate'
import { GithubAccessSettingsPanel } from './GithubAccessSettings'
import { AppUpdateSettings } from './AppUpdateSettings'
import { openExternal } from '../lib/community'
import { t } from '../i18n'
import { githubErrorMessage, assetMissingReason } from '../../../ui-shared/updates'
import { Card, Button } from '@ui'
import { Switch as UiSwitch } from '@ui'

/**
 * use update（Android / iOS use ， as by ）。
 *
 * ── ──────────────────────────────────────────────────────
 *   idle → checking → (up-to-date | available) → downloading → downloaded
 * 1 failure to idle and to error。
 *
 * ── iOS Why has 「 down 」 ────────────────────────────────────
 * not App down and can ，IPA can by （App Store /
 * TestFlight / ）。 iOS up this to 「 new 」 as ，
 * by GitHub Release ， by use —— not down 
 * and 「」 by ， to 1 failure 。
 *
 * ── check ────────────────────────────────────────────────────
 * when by config check（ `/api/scheduled-tasks` 
 * `updateCheck` ， lib/appUpdate）。 not can when —— and 
 * 1 can then ， up not this in after 。
 *
 * ── Why「 already down 」 need 1 ──────────────────────────────────
 * use down 、、 when will new —— if by ，
 * will to 「 has new 」 down 1 （ MB）。 when 1 
 * `downloaded()`， has 「」。
 *
 * ── ────────────────────────────────────────────────────
 * down need ： down not need need ， will set —— need 
 * down ， use when down no ， is 。
 */
type Phase = 'idle' | 'checking' | 'latest' | 'available' | 'downloading' | 'downloaded'

export function AppUpdatePanel() {
  const [phase, setPhase] = useState<Phase>('idle')
  const [info, setInfo] = useState<ApkUpdateInfo | null>(null)
  const [progress, setProgress] = useState<DownloadProgress | null>(null)
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)
  const [auto, setAuto] = useState(autoCheckEnabled)
  const [settingsRevision, setSettingsRevision] = useState(0)
  const channelRevision = useRef(0)
  const checkInFlight = useRef(false)
  /** check （0 = this ） */
  const [interval, setIntervalMs] = useState(0)
  /** current can not can ： has Android can （ lib/appUpdate） */
  const selfUpdate = canSelfUpdate()

  // when has no has up down （iOS up as false）
  useEffect(() => {
    let cancelled = false
    void hasDownloadedApk().then(result => {
      if (!cancelled && result.exists) setPhase('downloaded')
    })
    return () => {
      cancelled = true
    }
  }, [])

  /**
   * check：config， not can when 。
   *
   * Why interval ： use in ， down this 
   * by new —— when new 1 can （set will ）。
   */
  useEffect(() => {
    let disposed = false
    let effectiveInterval = 0
    let timer: number | null = null

    const clear = () => {
      if (timer !== null) {
        window.clearInterval(timer)
        timer = null
      }
    }
    const start = (ms: number) => {
      clear()
      if (disposed || ms <= 0) return
      timer = window.setInterval(() => {
        if (document.visibilityState === 'visible') void runCheck(true)
      }, ms)
    }

    void (async () => {
      const ms = await autoCheckIntervalMs()
      if (disposed) return
      setIntervalMs(ms)
      effectiveInterval = ms
      if (!auto || ms <= 0) return
      await runCheck(true)
      start(ms)
    })()

    const onVisibility = () => {
      if (auto && document.visibilityState === 'visible' && effectiveInterval > 0) void runCheck(true)
    }
    document.addEventListener('visibilitychange', onVisibility)

    return () => {
      disposed = true
      clear()
      document.removeEventListener('visibilitychange', onVisibility)
    }
    // set after check； is 。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auto, settingsRevision])

  // check use in down / already down 
  const phaseRef = useRef(phase)
  phaseRef.current = phase

  /** `silent` = check：failure not error、 not current */
  const runCheck = async (silent = false) => {
    const current = phaseRef.current
    if (current === 'downloading' || current === 'downloaded' || checkInFlight.current) return
    const revision = channelRevision.current
    checkInFlight.current = true
    if (!silent) {
      setBusy(true)
      setPhase('checking')
    }
    if (!silent) setError('')
    try {
      const result = await checkApkUpdate()
      if (revision !== channelRevision.current) return
      setInfo(result)
      // hasUpdate as null = none（ up tag not is semver）；
      // this when need APK update， not is in 「unknown」 up 
      const canUpdate = result.hasUpdate === true || (result.hasUpdate === null && result.asset !== null)
      setPhase(prev => (prev === 'downloading' || prev === 'downloaded' ? prev : canUpdate ? 'available' : 'latest'))
    } catch (failure) {
      if (!silent && revision === channelRevision.current) {
        setError(githubErrorMessage(failure, t, 'update'))
        setPhase('idle')
      }
    } finally {
      checkInFlight.current = false
      if (!silent) setBusy(false)
    }
  }

  const download = async () => {
    const url = info?.asset?.url
    if (!url) {
      // need is `.apk`（ after by after ， pick_android_apk）——
      // Reason and 1 ， is 「 need 」 this 1 apk
      setError(t('这一版没有提供 Android 安装包：{reason}', {
        reason: assetMissingReason(info?.assetProbe, { fallbackSuffix: '.apk' }, t),
      }))
      return
    }
    setBusy(true)
    setError('')
    setProgress({ received: 0, total: 0, percent: -1 })
    setPhase('downloading')
    try {
      await downloadApk(url, setProgress)
      setPhase('downloaded')
    } catch (failure) {
      setError(githubErrorMessage(failure, t, 'update'))
      setPhase('available')
    } finally {
      setBusy(false)
      setProgress(null)
    }
  }

  const install = async () => {
    setBusy(true)
    setError('')
    try {
      // ： no has to set， after use 1 
      if (!(await canInstallApk())) {
        store.toast(t('请允许安装未知来源应用后重试'))
        await openInstallPermission()
        return
      }
      await installApk()
      store.toast(t('已启动安装器'))
    } catch (failure) {
      setError(githubErrorMessage(failure, t, 'update'))
    } finally {
      setBusy(false)
    }
  }

  const stop = async () => {
    await cancelDownload()
    setPhase('available')
    setProgress(null)
    setBusy(false)
  }

  /**
   * iOS 「update」 can ： GitHub Release ， by use 
   * （TestFlight / App Store / ）。`pageUrl` is from GitHub API to 
   * release ， no has when **** releases —— 
   * （） not is ， not can 。
   */
  const openReleasePage = () => {
    const target = info?.pageUrl || 'https://github.com/aimod-cc/Agent2API/releases'
    openExternal(target)
  }

  const toggleAuto = () => {
    const next = !auto
    setAutoCheckEnabled(next)
    setAuto(next)
    store.toast(next ? t('已开启自动检查') : t('已关闭自动检查'))
  }

  const percent = progress && progress.percent >= 0 ? progress.percent : null
  const sizeText = (bytes?: number) => {
    if (!bytes) return ''
    const mb = bytes / 1024 / 1024
    return `${mb >= 100 ? mb.toFixed(0) : mb.toFixed(1)} MB`
  }

  return (
    <div className='m-section'>
      <SectionHead title={t('软件更新')} />
      {info?.forceUpdate?.required ? <div className='mb-2 text-sm text-warning'>{t('需要强制更新')} · {info.forceUpdate.message}</div> : null}
      <Card className='m-card'>
        <div className='m-row' style={{ alignItems: 'flex-start' }}>
          <span className='m-logo' style={{ background: 'var(--m-primary-soft)', borderColor: 'transparent', color: 'var(--m-primary)' }}>
            <Icon name='up' />
          </span>
          <div className='m-grow' style={{ minWidth: 0 }}>
            <div className='m-t1'>
              {phase === 'latest'
                ? t('已是最新版本')
                : phase === 'available'
                  ? t('发现新版本 {version}', { version: info?.latestVersion || '' })
                  : phase === 'downloading'
                    ? progress?.phase === 'selecting' ? t('正在选择下载通道…') : t('正在下载…')
                    : phase === 'downloaded'
                      ? t('已下载，待安装')
                      : phase === 'checking'
                        ? t('正在检查…')
                        : t('当前版本 {version}', { version: APP_VERSION })}
            </div>
            <div className='m-t2' style={{ wordBreak: 'break-word' }}>
              {phase === 'available' && !selfUpdate
                ? t('请在 Release 页面下载后通过 TestFlight 或 App Store 安装')
                : phase === 'available' && info?.asset
                  ? t('{size} · 下载后可直接安装，无需跳转浏览器', { size: sizeText(info.asset.size) })
                  : phase === 'downloaded'
                    ? t('安装包已就绪，点「安装」即可完成更新')
                    : phase === 'downloading' && percent !== null
                      ? t('已完成 {percent}%', { percent })
                    : phase === 'latest'
                      ? t('当前版本 {version}', { version: APP_VERSION })
                      : selfUpdate
                        ? t('版本 {version} · 按 GitHub 访问设置下载安装包', { version: APP_VERSION })
                        : t('版本 {version} · 更新包需从 Release 页面获取', { version: APP_VERSION })}
            </div>

            {phase === 'downloading' && progress?.source ? <div className='m-t2'>
              {t('当前下载通道：{source}', { source: progress.source })}
            </div> : null}

            {/* ：totalunknown（percent = null） when not ， already down */}
            {phase === 'downloading' ? (
              <div style={{ marginTop: 8 }}>
                <UiProgress value={percent} aria-label={t('下载进度')} />
                {progress ? (
                  <div style={{ marginTop: 4, fontSize: 11, color: 'var(--m-text-3)' }}>
                    {sizeText(progress.received)}
                    {progress.total > 0 ? ` / ${sizeText(progress.total)}` : ''}
                  </div>
                ) : null}
              </div>
            ) : null}

            {error ? (
              <div style={{ marginTop: 6, fontSize: 11.5, color: 'var(--m-danger)', wordBreak: 'break-word' }}>
                {error}
              </div>
            ) : null}
          </div>

          {/* by ： by to 1 ， 1 can */}
          {phase === 'downloading' ? (
            <Button size='touch-sm' variant='outline' type='button' className='m-btn m-ghost m-sm' onClick={() => void stop()}>
              {t('取消')}
            </Button>
          ) : phase === 'downloaded' ? (
            <Button size='touch-sm' variant='default' type='button' className='m-btn m-sm' disabled={busy} onClick={() => void install()}>
              {t('安装')}
            </Button>
          ) : phase === 'available' && !selfUpdate ? (
            // iOS： can ， by 
            <Button size='touch-sm' variant='default' type='button' className='m-btn m-sm' onClick={openReleasePage}>
              {t('去下载')}
            </Button>
          ) : phase === 'available' ? (
            <Button size='touch-sm' variant='default' type='button' className='m-btn m-sm' disabled={busy} onClick={() => void download()}>
              {t('立即更新')}
            </Button>
          ) : (
            <Button size='touch-sm' variant='outline'
              type='button'
              className='m-btn m-ghost m-sm'
              disabled={busy}
              onClick={() => void runCheck()}
            >
              {phase === 'checking' ? t('检查中…') : t('检查更新')}
            </Button>
          )}
        </div>

        {/* update： has ，empty */}
        {info?.notes && (phase === 'available' || phase === 'downloaded') ? (
          <div
            style={{
              padding: '10px 13px',
              borderTop: '1px solid var(--m-line)',
              fontSize: 11.5,
              lineHeight: 1.6,
              color: 'var(--m-text-2)',
              whiteSpace: 'pre-wrap',
              maxHeight: 180,
              overflowY: 'auto',
              wordBreak: 'break-word',
            }}
          >
            {info.notes}
          </div>
        ) : null}

        {/* check： 1 ， this 「 this not 」 */}
        <div className='m-row' style={{ borderTop: '1px solid var(--m-line)' }}>
          <span className='m-logo'>
            <Icon name='refresh' />
          </span>
          <div className='m-grow'>
            <div className='m-t1'>{t('允许本机自动检查')}</div>
            <div className='m-t2'>
              {interval > 0
                ? t('与电脑端一致：每 {n} 分钟', { n: Math.round(interval / 60_000) })
                : t('网关已关闭自动检查，此处只做手动检查')}
            </div>
          </div>
          <UiSwitch size='touch' variant='success' checked={auto} aria-label={t('允许本机自动检查')}  onCheckedChange={() => toggleAuto()} />
        </div>
        <AppUpdateSettings onChanged={channelsChanged => {
          if (channelsChanged) channelRevision.current += 1
          if (channelsChanged && phaseRef.current !== 'downloading' && phaseRef.current !== 'downloaded') {
            setInfo(null)
            setPhase('idle')
            setError('')
          }
          setSettingsRevision(revision => revision + 1)
        }} />
        <GithubAccessSettingsPanel />
      </Card>
    </div>
  )
}
