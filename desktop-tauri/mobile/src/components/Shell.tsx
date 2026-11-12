import { useEffect, useState } from 'react'
import { Icon, type IconName } from './Icon'
import { useStore, store } from '../state/store'
import { LoginPage } from '../pages/LoginPage'
import { isRateLimited } from '../lib/format'
import { t } from '../i18n'
import { BottomNavigation, BottomNavigationItem, Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Input, StatusDot, Toaster, createToastManager } from '@ui'

const feedbackManager = createToastManager()

/**
 * use ： + + + 。
 *
 * ── then ─────────────────────────────────────────
 * 1 ，（ to ）。 need ， 
 * （ pages/AssistantPage as-mode ）。
 */

export type TabDef = { id: string; label: string; icon: IconName }

const TABS: TabDef[] = [
  { id: 'home', label: t('首页'), icon: 'home' },
  { id: 'report', label: t('报表'), icon: 'chart' },
  { id: 'accounts', label: t('账号'), icon: 'users' },
  { id: 'assistant', label: t('助手'), icon: 'spark' },
  { id: 'more', label: t('更多'), icon: 'grid' },
]

/** 1 ： this screen */
export const PRIMARY_SCREENS = new Set(TABS.map(tab => tab.id))

/**
 * is 。
 *
 * ── Why need need this ────────────────────────────────────────
 * Android `adjustResize` will WebView 1 ， is flex 
 * after 1 ， is to up —— use to is 「 + 1 
 * + 」， need is 「 + 」。 use is 
 * when （WhatsApp / Telegram is this ）。
 *
 * ── ────────────────────────────────────────────────────
 * `visualViewport` `innerHeight` 1 （> 120px） is in 
 * 。 120 not is 0： / will 
 * ， that is ， not this 1 down 。
 *
 * no has `visualViewport` WebView return false —— ， as and 
 * before 1 ， not will as not to API 1 。
 */
export function useKeyboardOpen(): boolean {
  const [open, setOpen] = useState(false)

  useEffect(() => {
    const viewport = window.visualViewport
    if (!viewport) return
    const threshold = 120
    const check = () => {
      const covered = window.innerHeight - viewport.height - viewport.offsetTop
      setOpen(covered > threshold)
    }
    viewport.addEventListener('resize', check)
    viewport.addEventListener('scroll', check)
    check()
    return () => {
      viewport.removeEventListener('resize', check)
      viewport.removeEventListener('scroll', check)
    }
  }, [])

  return open
}

export function TabBar() {
  const state = useStore()
  const keyboardOpen = useKeyboardOpen()
  const active = state.stack.length === 0 ? state.route.screen : ''
  if (!PRIMARY_SCREENS.has(state.route.screen) || state.stack.length > 0) return null
  // when ： that 1 to ， not is 
  if (keyboardOpen) return null
  const limited = state.session?.accounts?.accounts?.filter(a => a.enabled && isRateLimited(a)).length || 0
  return (
    <BottomNavigation className='m-tabbar' aria-label={t('主导航')}>
      {TABS.map(tab => (
        <BottomNavigationItem
          key={tab.id}
          active={active === tab.id}
          className='m-tab'
          onClick={() => store.go(tab.id)}
        >
          <Icon name={tab.icon} />
          <span>{tab.label}</span>
          {tab.id === 'more' && limited > 0 ? <StatusDot variant='destructive' className='m-dot' /> : null}
        </BottomNavigationItem>
      ))}
    </BottomNavigation>
  )
}

/** */
export function Toast() {
  const { toast } = useStore()
  useEffect(() => {
    if (toast) feedbackManager.add({ id: 'mobile-feedback', title: toast.text, type: toast.tone === 'err' ? 'error' : toast.tone === 'warn' ? 'warning' : 'success', timeout: 3500 })
  }, [toast])
  return <Toaster manager={feedbackManager} position='bottom-center' />
}

/**
 * 。
 *
 * ， and not ：
 * · `login` —— already 、 will ：APK use ；
 * （PWA）server，。
 * · `key` —— （ no ） no Key：， localStorage
 *
 * Why APK not can `/login`： that by ， APK in ——
 * etc use 、 1 not 1 ， and after server 
 * cookie not （ lib/servers.ts ）。
 */
export function AuthGate() {
  const state = useStore()
  const [key, setKey] = useState('')
  const [busy, setBusy] = useState(false)
  if (!state.authLost) return null

  /**
   * APK is ， use after not to 「set → 」。
   * to 「 1 」， use —— this 1 must，
   * otherwise ， not to 。authLost not ： use return after 
   * will new up ， to new will by new new 。
   */
  if (state.packaged && state.route.screen === 'servers') return null

  // use （login / key ）
  if (state.packaged) return <LoginPage mode={state.authLost} />

  const submit = async () => {
    const value = key.trim()
    if (!value) return
    setBusy(true)
    const { storeApiKey } = await import('../api/client')
    storeApiKey(value)
    store.setAuthLost(null)
    await store.refresh(true)
    setBusy(false)
  }

  return <Dialog open disablePointerDismissal>
    <DialogContent className='w-[min(340px,calc(100vw-40px))]' showCloseButton={false}>
      <DialogHeader><DialogTitle>{state.authLost === 'login' ? t('需要重新登录') : t('需要网关 API Key')}</DialogTitle></DialogHeader>
      <DialogBody>
        <p className='text-[12.5px] leading-relaxed text-subtle'>{state.authLost === 'login'
          ? t('会话已过期。重新登录后会自动回到手机端。')
          : t('这个面板没有管理员，用一把已启用的 API Key 访问（在桌面端「网关 Key」页创建）。')}</p>
        {state.authLost === 'key' && <Input type='password' value={key} onChange={event => setKey(event.target.value)} placeholder='sk-…' autoComplete='off' />}
      </DialogBody>
      <DialogFooter>
        {state.authLost === 'login' ? <Button size='touch' className='w-full' onClick={() => { window.location.href = '/login?next=%2Fm' }}>{t('去登录')}</Button>
          : <Button size='touch' className='w-full' onClick={() => void submit()} disabled={busy || !key.trim()}>{busy ? t('验证中…') : t('保存并继续')}</Button>}
      </DialogFooter>
    </DialogContent>
  </Dialog>
}

/**
 * ： store to <html>。
 *
 * 1 not is in store when ， is as React and DOM 
 * 1 —— （） in up is can 。
 */
export function ThemeSync() {
  const { theme } = useStore()
  useEffect(() => {
    document.documentElement.dataset.theme = theme
  }, [theme])
  return null
}
