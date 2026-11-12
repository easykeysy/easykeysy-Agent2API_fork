import React, { useEffect } from 'react'
import { AppFrame } from '@ui'
import { store, useStore, bootstrapTheme } from './state/store'
import { TabBar, Toast, AuthGate, ThemeSync } from './components/Shell'
import { PwaNotice } from './components/PwaNotice'
import { usePwa } from './lib/usePwa'
import { HomePage } from './pages/HomePage'
import { ReportPage } from './pages/ReportPage'
import { AccountsPage } from './pages/AccountsPage'
import { AssistantPage } from './pages/AssistantPage'
import { MorePage } from './pages/MorePage'
import { ModelsPage } from './pages/ModelsPage'
import { KeysPage } from './pages/KeysPage'
import { ProxiesPage } from './pages/ProxiesPage'
import { TasksPage } from './pages/TasksPage'
import { RequestsPage } from './pages/RequestsPage'
import { LogsPage } from './pages/LogsPage'
import { SettingsPage } from './pages/SettingsPage'
import { DocsPage } from './pages/DocsPage'
import { AgentPage } from './pages/AgentPage'
import { ToolsPage } from './pages/ToolsPage'
import { ServersPage } from './pages/ServersPage'
import { CheckinPage } from './pages/CheckinPage'
import { PluginsPage } from './pages/PluginsPage'
import type { ScreenId } from './lib/routes'
import { NavBar } from './components/NavBar'
import { t } from './i18n'
import { RequiredUpdateGate } from './components/RequiredUpdateGate'

/**
 * by 。
 *
 * 1 （`primary: true`）；return。
 * and not ： 13 1 in up 5 ， its 
 * 「」 —— has ， not has 。
 */
type ScreenDef = {
  title: string
  component: () => React.ReactElement
  primary?: boolean
}

const SCREENS: Record<ScreenId, ScreenDef> = {
  home: { title: t('首页'), component: HomePage, primary: true },
  report: { title: t('报表'), component: ReportPage, primary: true },
  accounts: { title: t('账号'), component: AccountsPage, primary: true },
  assistant: { title: t('AI 助手'), component: AssistantPage, primary: true },
  more: { title: t('更多'), component: MorePage, primary: true },
  models: { title: t('模型管理'), component: ModelsPage },
  keys: { title: t('网关 Key'), component: KeysPage },
  proxies: { title: t('网络代理'), component: ProxiesPage },
  tasks: { title: t('定时任务'), component: TasksPage },
  requests: { title: t('请求日志'), component: RequestsPage },
  logs: { title: t('运行日志'), component: LogsPage },
  settings: { title: t('设置'), component: SettingsPage },
  docs: { title: t('接口文档'), component: DocsPage },
  agent: { title: t('Agent 配置'), component: AgentPage },
  tools: { title: t('工具服务'), component: ToolsPage },
  servers: { title: t('服务器'), component: ServersPage },
  checkin: { title: t('签到中心'), component: CheckinPage },
  plugins: { title: t('网关插件'), component: PluginsPage },
}

export function App() {
  const state = useStore()
  const pwa = usePwa()
  const def = SCREENS[state.route.screen]
  const Page = def.component
  const isPrimary = Boolean(def.primary)
  // for ， not create 。
  const fillHeight = state.route.screen === 'assistant'
  /**
   * APK 、 1 no ：** no has not to **，
   * （ not —— to is empty ）。 1 after 
   * this 。
   */
  const needsServer = state.packaged && !state.server

  useEffect(() => {
    bootstrapTheme()
    store.restoreFromHash()
    // no has available when not need request： that will 「none」 1 error，
    // Reason is 「 no 」。
    if (!state.packaged || state.server) {
      void store.refresh(true)
    }
    // before / after （PWA and APK WebView will ）。 store 
    // handlePopState not is restoreFromHash： before will ，
    // after hash， will 「return」「current」。
    const onPop = () => store.handlePopState()
    window.addEventListener('popstate', onPop)
    return () => window.removeEventListener('popstate', onPop)
    // in when 1 ： after refresh by and 
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  /**
   * return。
   *
   * not Capacitor default as is finish Activity —— use in 
   * by 1 down will to ， not is up 1 。
   *
   * ： already （Sheet / Dialog ），
   * to store 。 and none when use —— this is 。
   */
  useEffect(() => {
    if (!state.packaged) return
    let dispose: (() => void) | undefined
    let cancelled = false
    void import('@capacitor/app')
      .then(({ App: CapApp }) => {
        if (cancelled) return
        void CapApp.addListener('backButton', () => {
          // when 。Base UI data-open， and by 
          // data-slot='sheet-close' / 'dialog-close'； etc use by 。
          const popup = document.querySelector<HTMLElement>(
            '[data-slot="sheet-content"][data-open], [data-slot="dialog-content"][data-open]',
          )
          if (popup) {
            const closer = popup.querySelector<HTMLElement>('[data-slot="sheet-close"], [data-slot="dialog-close"]')
            closer?.click()
            return
          }
          if (!store.handleHardwareBack()) void CapApp.exitApp()
        }).then(handle => {
          if (cancelled) void handle.remove()
          else dispose = () => void handle.remove()
        })
      })
      .catch(() => {
        /* （e.g.） when not ： has return */
      })
    return () => {
      cancelled = true
      dispose?.()
    }
    // packaged in use not 
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    // current（PWA ）
    document.title = needsServer ? `${t('服务器')} · Agent2API` : `${def.title} · Agent2API`
  }, [def.title, needsServer])

  if (needsServer) {
    return (
      <AppFrame className='m-app'>
        <ThemeSync />
        <div className='m-scroll' data-screen='servers'>
          <ServersPage key={state.serverRevision} />
        </div>
        <Toast />
      </AppFrame>
    )
  }

  return (
    <AppFrame className='m-app' data-offline={pwa.offline ? 'true' : undefined}>
      <ThemeSync />
      <PwaNotice />
      {isPrimary ? null : <NavBar title={def.title} onBack={() => store.back()} />}
      <div
        className='m-scroll'
        style={fillHeight ? { overflow: 'hidden' } : undefined}
        data-screen={state.route.screen}
      >
        <Page key={state.serverRevision} />
      </div>
      <TabBar />
      <Toast />
      <RequiredUpdateGate />
      <AuthGate key={state.serverRevision} />
    </AppFrame>
  )
}
