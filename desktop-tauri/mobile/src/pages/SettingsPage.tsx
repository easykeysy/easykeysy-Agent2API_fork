import { useState } from 'react'
import { PageIntro, SectionHead, IconButton } from '../components/ui'
import { Icon } from '../components/Icon'
import { ThemeSwitch } from './MorePage'
import { store, useStore } from '../state/store'
import { locale, localeOptions, setLocale } from '../i18n'
import { storeApiKey } from '../api/client'
import { usePwa } from '../lib/usePwa'
import { applyUpdate, promptInstall } from '../lib/pwa'
import { APP_VERSION } from '../lib/appUpdate'
import { AppUpdatePanel } from '../components/AppUpdatePanel'
import { CommunitySheet } from '../components/CommunitySheet'
import { SystemInfoSheet } from '../components/SystemInfoSheet'
import { apiBase, baseLabel, serverIdentity } from '../lib/servers'
import { usePullToRefresh, PullIndicator } from '../components/PullToRefresh'
import { t } from '../i18n'
import { Card, Button, Badge, BadgeDot, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Sheet, SheetContent, SheetHeader, SheetBody, SheetTitle } from '@ui'
import { ClientIdentificationPanel } from '../../../ui-shared/client-identification-panel'
import { clientIdentificationApi } from '../api/client-identification'
import { VisionCompatibilityPanel } from '../../../ui-shared/settings-vision-panel'
import { visionCompatApi } from '../api/vision-compat'

/**
 * 设置。
 *
 * ── 手机上只放「改了就生效、且不需要复杂表单」的项 ────────────
 * 桌面端设置页有 11 个分类、几十个字段。手机上保留：
 *   · 外观与语言（纯前端偏好）
 *   · 账号凭证（兼容模式的 API Key、退出登录）
 *   · 网关地址（复制给客户端）
 *   · 关于与安装（PWA 安装引导、版本）
 * 端口、路由、重试、超时、数据保留这些需要看文档才能填对的，一律留在桌面端
 * —— 摆出来只会让人误改。
 */
export function SettingsPage() {
  const { session, packaged, server, servers } = useStore()
  const pwa = usePwa()
  const [installing, setInstalling] = useState(false)
  const [community, setCommunity] = useState(false)
  const [systemInfo, setSystemInfo] = useState(false)
  const [clientIdentification, setClientIdentification] = useState(false)
  const [visionCompatibility, setVisionCompatibility] = useState(false)
  const currentLocale = locale()
  const options = localeOptions()

  // 本页没有自己的列表请求：下拉只需重拉会话（网关状态来自它）
  const pull = usePullToRefresh({ onRefresh: () => store.refresh(true) })

  const install = async () => {
    setInstalling(true)
    const outcome = await promptInstall()
    setInstalling(false)
    if (outcome === 'accepted') store.toast(t('Installing…'))
    else if (outcome === 'dismissed') store.toast(t('Installation cancelled'))
    // 'unavailable'：没有可用事件（iOS 或已安装），界面本来就展示手动指引
  }

  const signOut = async () => {
    const identity = serverIdentity()
    const { logout } = await import('../api/auth')
    const { clearPanelAuth } = await import('../api/client')
    await logout()
    if (serverIdentity() !== identity) return
    clearPanelAuth()
    store.setAuthLost('login')
    store.toast(t('Signed out'))
  }

  return (
    <div className='m-page' ref={pull.bindRef}>
      <PullIndicator pull={pull.pull} refreshing={pull.refreshing} />
      <PageIntro subtitle={t('Appearance · Access · About')}
        actions={<IconButton name='refresh' label={t('Refresh')} onClick={() => void store.refresh(true)} />}
      />

      {/* 服务器：APK 才有（浏览器里地址就是当前 origin，没什么可配的） */}
      {packaged ? (
        <div className='m-section' style={{ marginTop: 0 }}>
          <SectionHead title={t('Servers')} />
          <Card className='m-card'>
            <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => store.push('servers')}>
              <span className='m-logo' style={{ background: 'var(--m-primary-soft)', borderColor: 'transparent', color: 'var(--m-primary)' }}>
                <Icon name='server' />
              </span>
              <div className='m-grow'>
                <div className='m-t1'>{server?.name || baseLabel(server?.base || '')}</div>
                <div className='m-t2 m-mono' style={{ fontSize: 11.5 }}>
                  {servers.length > 1
                    ? t('{n} servers · tap to switch', { n: servers.length })
                    : t('Tap to add or switch servers')}
                </div>
              </div>
              <span className='m-chev'>
                <Icon name='chev' />
              </span>
            </Button>
          </Card>
        </div>
      ) : null}

      {/* 网关状态 */}
      <div className='m-section' style={packaged ? undefined : { marginTop: 0 }}>
        <Card className='m-card'>
          <div className='m-row'>
            <span
              className='m-logo'
              style={{ background: 'var(--m-ok-soft)', borderColor: 'transparent', color: 'var(--m-ok)' }}
            >
              <Icon name='bolt' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{session?.health?.upstreamConfigured ? t('Gateway running') : t('Upstream not configured')}</div>
              <div className='m-t2 m-mono' style={{ fontSize: 11.5 }}>
                {baseLabel(apiBase())}
              </div>
            </div>
            <Badge variant={session?.health?.upstreamConfigured ? 'success' : 'warning'} shape='tag'>
              <BadgeDot />{session?.health?.upstreamConfigured ? t('Normal') : t('Not configured')}
            </Badge>
          </div>
        </Card>
      </div>

      <div className='m-section'>
        <SectionHead title={t('Appearance')} />
        <Card className='m-card'>
          <div className='m-row'>
            <span className='m-logo'>
              <Icon name='moon' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Theme')}</div>
              <div className='m-t2'>{t('System · Light · Dark')}</div>
            </div>
            <ThemeSwitch />
          </div>
          <div className='m-row'>
            <span className='m-logo'>
              <Icon name='globe' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Interface Language')}</div>
              <div className='m-t2'>{options.find(item => item.code === currentLocale)?.label || currentLocale}</div>
            </div>
            <Select value={currentLocale} onValueChange={value => setLocale(String(value))}>
              <SelectTrigger aria-label={t('Interface Language')} className='w-auto shrink-0'>
                <SelectValue>{options.find(item => item.code === currentLocale)?.label || currentLocale}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                {options.map(item => <SelectItem key={item.code} value={item.code}>{item.label}</SelectItem>)}
              </SelectContent>
            </Select>
          </div>
        </Card>
      </div>

      <div className='m-section'>
        <SectionHead title={t('Access')} />
        <Card className='m-card'>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => setVisionCompatibility(true)}>
            <span className='m-logo'><Icon name='model' /></span>
            <div className='m-grow'><div className='m-t1'>{t('Image compatibility')}</div><div className='m-t2'>{t('Assisted reading or vision model switching, off by default')}</div></div>
            <span className='m-chev'><Icon name='chev' /></span>
          </Button>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => setClientIdentification(true)}>
            <span className='m-logo'><Icon name='doc' /></span>
            <div className='m-grow'><div className='m-t1'>{t('Client identification')}</div><div className='m-t2'>{t('Built-in identification, custom rules, and origin declarations')}</div></div>
            <span className='m-chev'><Icon name='chev' /></span>
          </Button>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => store.push('provider-settings')}>
            <span className='m-logo'><Icon name='settings' /></span>
            <div className='m-grow'><div className='m-t1'>{t('Provider settings')}</div><div className='m-t2'>{t('View and adjust provider-specific options')}</div></div>
            <span className='m-chev'><Icon name='chev' /></span>
          </Button>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => store.push('plugins')}>
            <span className='m-logo'><Icon name='plugin' /></span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Gateway plugins')}</div>
              <div className='m-t2'>{t('View, install and manage the plugins of the connected gateway')}</div>
            </div>
            <span className='m-chev'><Icon name='chev' /></span>
          </Button>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => store.push('docs')}>
            <span className='m-logo'>
              <Icon name='doc' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('API address')}</div>
              <div className='m-t2 m-mono' style={{ fontSize: 11.5 }}>
                {apiBase()}/v1
              </div>
            </div>
            <span className='m-chev'>
              <Icon name='chev' />
            </span>
          </Button>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => store.push('keys')}>
            <span className='m-logo'>
              <Icon name='key' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Gateway key')}</div>
              <div className='m-t2'>{t('Copy for your client')}</div>
            </div>
            <span className='m-chev'>
              <Icon name='chev' />
            </span>
          </Button>
        </Card>
      </div>

      <div className='m-section'>
        <SectionHead title={t('Accounts')} />
        <Card className='m-card'>
          {/* APK 有真正的登录态，给「退出登录」；浏览器端只有 API Key 可清 */}
          {packaged ? (
            <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => void signOut()}>
              <span className='m-logo'>
                <Icon name='logout' />
              </span>
              <div className='m-grow'>
                <div className='m-t1'>{t('Sign Out')}</div>
                <div className='m-t2'>{t('Sign out of this server only; others are unaffected')}</div>
              </div>
              <span className='m-chev'>
                <Icon name='chev' />
              </span>
            </Button>
          ) : (
            <Button size='default' variant='ghost'
              type='button'
              className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left'
              onClick={() => {
                storeApiKey('')
                store.setAuthLost(null)
                store.toast(t('Removed the API key stored in this browser'))
                void store.refresh(true)
              }}
            >
              <span className='m-logo'>
                <Icon name='logout' />
              </span>
              <div className='m-grow'>
                <div className='m-t1'>{t('Clear local credentials')}</div>
                <div className='m-t2'>{t('Only clears the key stored in this browser; the gateway is untouched')}</div>
              </div>
              <span className='m-chev'>
                <Icon name='chev' />
              </span>
            </Button>
          )}
        </Card>
      </div>

      {/* APK 不需要装到桌面（本身就是 App），这一段只给浏览器形态 */}
      {packaged ? null : (
      <div className='m-section'>
        <SectionHead title={t('Install as app')} />
        <Card className='m-card'>
          <div className='m-row'>
            <span className='m-logo'>
              <Icon name='install' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Running as')}</div>
              <div className='m-t2'>
                {pwa.standalone ? t('Standalone app') : t('Browser tab')}
              </div>
            </div>
            <Badge variant={pwa.standalone ? 'success' : 'secondary'} shape='tag'>
              {pwa.standalone ? t('Installed') : t('Not installed')}
            </Badge>
          </div>

          {pwa.standalone ? (
            <div className='m-card-pad' style={{ borderTop: '1px solid var(--m-line)', fontSize: 12.5, lineHeight: 1.7, color: 'var(--m-text-2)' }}>
              {t('Launch it from the home screen icon: full screen, no address bar. To uninstall, long-press the icon and remove it, or clear the browser\'s site data.')}
            </div>
          ) : pwa.canInstall ? (
            <div style={{ borderTop: '1px solid var(--m-line)', padding: '12px 15px 14px' }}>
              <div style={{ fontSize: 12.5, lineHeight: 1.7, color: 'var(--m-text-2)' }}>
                {t('Installed as an app it opens straight from the home screen icon: full screen, no address bar.')}
              </div>
              <Button size='touch' variant='default'
                type='button'
                className='m-btn w-full'
                style={{ marginTop: 10, height: 40 }}
                disabled={installing}
                onClick={() => void install()}
              >
                <Icon name='download' />
                {installing ? t('Installing…') : t('Add to Home Screen')}
              </Button>
            </div>
          ) : (
            <div style={{ borderTop: '1px solid var(--m-line)', padding: '12px 15px 14px' }}>
              <div style={{ fontSize: 12.5, lineHeight: 1.75, color: 'var(--m-text-2)' }}>
                {pwa.showIosHint
                  ? t('Safari has no install button: tap Share at the bottom, choose "Add to Home Screen", then tap "Add".')
                  : t('Use "Add to Home Screen" in the browser menu to install this page as an app: full screen, no address bar, just like a native app.')}
              </div>
              {pwa.showIosHint ? (
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    marginTop: 10,
                    padding: '9px 11px',
                    borderRadius: 11,
                    background: 'var(--m-surface-2)',
                    fontSize: 12,
                    color: 'var(--m-text-3)',
                  }}
                >
                  <Icon name='share' style={{ width: 16, height: 16, flex: 'none' }} />
                  <span>{t('The Share icon sits in the middle of Safari\'s bottom toolbar')}</span>
                </div>
              ) : null}
            </div>
          )}
        </Card>
      </div>
      )}

      {/* 只有真有事时才出现：平时不占版面 */}
      {pwa.updateReady || pwa.swState === 'unsupported' ? (
        <div className='m-section'>
          <SectionHead title={t('App status')} />
          <Card className='m-card'>
            {pwa.updateReady ? (
              <div className='m-row'>
                <span className='m-logo' style={{ background: 'var(--m-primary-soft)', borderColor: 'transparent', color: 'var(--m-primary)' }}>
                  <Icon name='up' />
                </span>
                <div className='m-grow'>
                  <div className='m-t1'>{t('New version')}</div>
                  <div className='m-t2'>{t('Takes effect after a restart; nothing you\'re doing is interrupted')}</div>
                </div>
                <Button size='touch-sm' variant='default' type='button' className='m-btn m-sm' onClick={() => void applyUpdate()}>
                  {t('Restart & update')}
                </Button>
              </div>
            ) : null}
            {pwa.swState === 'unsupported' ? (
              <div className='m-row'>
                <span className='m-logo'>
                  <Icon name='info' />
                </span>
                <div className='m-grow'>
                  <div className='m-t1'>{t('Offline cache is off')}</div>
                  <div className='m-t2'>{t('Browsers only offer offline caching over https or localhost. Accessing over plain http on the LAN still works normally.')}</div>
                </div>
              </div>
            ) : null}
          </Card>
        </div>
      ) : null}

      {/*
        APK 的更新走安装包（浏览器里没有这条路径：PWA 的更新是 Service Worker，
        见上面的「应用状态」）。两套互斥，不会同时出现。
      */}
      {packaged ? <AppUpdatePanel /> : null}

      <div className='m-section' style={{ paddingBottom: 8 }}>
        <Card className='m-card'>
          {/* 「Agent2API 手机端」这一行点开是版本信息抽屉：本机版本 + 所连网关
              的版本 / 形态 / 系统 / 路径（手机端出问题时第一句要问的就是这些，
              而用户手里只有手机，不该被要求跑回电脑前翻设置页）。
              版本号仍常驻在右侧 —— 一眼能读到的那个数最常用。 */}
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => setSystemInfo(true)}>
            <span className='m-logo'>
              <Icon name='info' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Agent2API Mobile')}</div>
              <div className='m-t2'>{t('Version info · connected gateway')}</div>
            </div>
            {/* 版本号常驻：排查问题时第一句就要问「你装的哪版」 */}
            <span className='m-mono' style={{ flex: 'none', fontSize: 11.5, color: 'var(--m-text-3)' }}>
              v{APP_VERSION}
            </span>
            <span className='m-chev'>
              <Icon name='chev' />
            </span>
          </Button>
          <Button size='default' variant='ghost'
            type='button'
            className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left'
            style={{ borderTop: '1px solid var(--m-line)' }}
            onClick={() => setCommunity(true)}
          >
            <span className='m-logo'>
              <Icon name='heart' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Community & support')}</div>
              <div className='m-t2'>{t('QQQ groups · Telegram')}</div>
            </div>
            <span className='m-chev'>
              <Icon name='chev' />
            </span>
          </Button>
        </Card>
      </div>

      <Sheet open={visionCompatibility} onOpenChange={setVisionCompatibility}>
        <SheetContent><SheetHeader><SheetTitle>{t('Image compatibility')}</SheetTitle></SheetHeader><SheetBody>
          {visionCompatibility ? <VisionCompatibilityPanel key={serverIdentity()} api={visionCompatApi} t={t} /> : null}
        </SheetBody></SheetContent>
      </Sheet>
      <Sheet open={clientIdentification} onOpenChange={setClientIdentification}>
        <SheetContent><SheetHeader><SheetTitle>{t('Client identification')}</SheetTitle></SheetHeader><SheetBody>
          {clientIdentification ? <ClientIdentificationPanel key={serverIdentity()} api={clientIdentificationApi} t={t} /> : null}
        </SheetBody></SheetContent>
      </Sheet>
      <CommunitySheet open={community} onClose={() => setCommunity(false)} />
      <SystemInfoSheet open={systemInfo} onClose={() => setSystemInfo(false)} />
    </div>
  )
}

