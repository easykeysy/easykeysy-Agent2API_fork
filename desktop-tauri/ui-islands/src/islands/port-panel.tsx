import * as React from 'react'
import { createRoot } from 'react-dom/client'
import {
  Button,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogSection,
  DialogTitle,
  Input,
  Label,
  StatusDot,
  type StatusDotProps,
} from '@ui'
import { t } from '../i18n'
import { API_ENDPOINTS } from '../../../ui-shared/docs-api-endpoints'

/**
 * Agent2API · （React ）。
 *
 * ui/port-panel.js。 for and ** 1 **：
 *   `window.wbPortPanel = { render, sync, portLabel, isReady }`
 * use in ui/app.js（sync 157 / 820 / 856、isReady 183、portLabel 186、
 * render 588）， 1 not use 。 in ： that by 
 * Button， Dialog 1 + Input / Label， from ui/css 
 * `.modal-*` / `.field-row` / `.detail` Tailwind 。
 *
 * ── this （ and app.js ）──────────────────────
 * · ：**** in not in 、 is ；
 * · when ： use / ；
 * · 「」 that （ id is api-*， in ，
 * by id ， not in ）。
 * · app.js render() and / will state；file** not state**。
 *
 * and `state.health` is ，： this is ，state.health is 
 * up （ has no has available）。
 *
 * ── ： + by need ，（file and its not ）──
 *
 * 1 `#sidebar-status` is **** ： in ， can ，
 * this in when 1 React root up 、 after 1 。
 * 「」 is ** by need ** ： in when ， conc-dialog /
 * request-clear-modal 「 when 、」， not index.html
 * #port-modal* （ that id 1 not ）。
 *
 * ── Why in .sidebar-status up 、 not ─────────
 * `.sidebar-status` by flex column （ layout.css），`.status-line`
 * and `.status-actions` 、 down this 。 in 1 
 * wrapper will 1 ，flex wrapper， then 。 this 
 * empty has after createRoot to —— is root ，
 * React not ， that title（） by by 。
 *
 * ── Why「 store」 not is state ─────────────
 * `sync()` will app.js from React use （20 seconds、、）， to 
 * after must to up 。 use useState can 「」 
 * ； this + useSyncExternalStore：sync() and 
 * ， up ， not will 。
 *
 * ── and ──────────────────────────────────────────
 * ** not when **： by app.js （ sync、 sync、20 seconds
 * 856 ）， old is this 。 1 is `backend:error`
 * （ etc ，「 in 1 」 ）， in effect 
 * 、 —— not 。
 */

/* ─── and ─────────────────────────────── */

/** 3065 is default； by getBackendStatus up */
const FALLBACK_PORT = 3065

/**
 * web （ by ） down ** is **。
 *
 * Why not can use 「 up 」：`getBackendStatus` to is ** **，
 * Docker 、 to 、 after ， from up not can 
 * —— `127.0.0.1:3065` ， to is 。
 * that 1 （`location.origin`） this ， is 1 。
 *
 * already this use ： URL is use `location.origin` 
 * （`web_shim::buildRaccoonWebAuthUrl`）。 from this 1 。
 *
 * **must**： that when origin is Tauri （`tauri://localhost`
 * ）， not is ， can 。 this in web return null，
 * by use to path。
 */
function webOrigin(): string | null {
  if (shared().workbuddyDesktop?.platform !== 'web') return null
  const origin = typeof window === 'undefined' ? '' : window.location?.origin
  // http(s) （file:// etc ）origin is "null"， that not is available
  return origin && origin !== 'null' ? origin : null
}

/** this ：web ， use up */
function gatewayBaseFor(port: number, current: string): string {
  return webOrigin() ?? (port ? `http://127.0.0.1:${port}` : current)
}

/**
 * `backend_status` return failure（Rust StartupFailure，
 * camelCase ）。`label` is that 1 use （「 use 」 this ），
 * by after to —— not ，otherwise、、 will 。
 */
type PortFailure = {
  /** to （ can can up ， title and ） */
  message?: string
  label?: string
  /** ；Reason（configfailure etc ） when as empty */
  conflict?: unknown
  /**
   * this failure can not can （「 use 」 to not to ）。
   *
   * Note: old is **** `backendStatus.canEndOccupant`，
   * `backend_status` return has `{ready, port, portFromEnv, failure}`
   * —— no has this ， is old up that by as hidden、 not 
   *（ and ）。 this by Rust （StartupFailure）
   * from failure ， that 「 can to this 」 。
   */
  canEndOccupant?: boolean
}

/** `getBackendStatus` return；null = no （ not is 「 not 」） */
type BackendStatus = {
  ready?: boolean
  port?: number
  portFromEnv?: boolean
  failure?: PortFailure | null
}

/** `getPortOccupant` return */
type PortOccupantInfo = {
  port?: number
  occupant?: { name?: string; pid?: number; path?: string } | null
}

/** use to ：、 use 、、（ bridge.rs） */
type PortBridge = {
  /** （'windows' / 'macos' / 'linux' / 'web'）。
   * is as 「」 and —— 
   * not （ webOrigin）。 */
  readonly platform?: string
  getBackendStatus(): Promise<BackendStatus | null | undefined>
  getPortOccupant(): Promise<PortOccupantInfo | null | undefined>
  endPortOccupant(): Promise<{ released?: boolean } | null | undefined>
  checkPort(port: number): Promise<{ ok?: boolean; message?: string; same?: boolean } | null | undefined>
  changePort(port: number): Promise<{ changed?: boolean } | null | undefined>
  restartApp(): Promise<unknown>
  /** failure；return（ bridge.rs on） */
  onBackendError?(callback: (payload: unknown) => void): (() => void) | void
  onBackendReady?(callback: () => void): (() => void) | void
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：
 * workbuddyDesktop / wbApp / wbConfirm is use ， declare 1 
 * will not 1 TS2717。file wbPortPanel。
 */
type SharedWindow = {
  workbuddyDesktop?: PortBridge
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    /** need is ， 1 （app.js ） */
    renderTopbarStatus?: () => void
  }
  wbConfirm?: {
    ask?: (options: {
      title?: string
      /** ，（confirm-dialog before ） */
      text?: string
      okText?: string
      /** danger = not can （） */
      okClass?: string
    }) => Promise<boolean>
  }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** / toast 1 （ and its 1 ） */
function toast(message: string, kind?: 'err' | 'ok'): void {
  shared().wbApp?.toast?.(message, kind)
}

/* ─── （ store）──────────────────── */

type PortSnapshot = {
  /** up after （getBackendStatus 1 ） */
  backend: BackendStatus | null
  /** current （ not /v1）：default， to after */
  gatewayBase: string
}

let snapshot: PortSnapshot = {
  backend: null,
  // web down to this already can to （web_shim in `<head>` after 、all
  // before ）， to —— not 1 down `127.0.0.1:3065` for 
  gatewayBase: gatewayBaseFor(0, `http://127.0.0.1:${FALLBACK_PORT}`),
}

/** （current has that 1 root） */
const subscribers = new Set<() => void>()

function getSnapshot(): PortSnapshot {
  return snapshot
}

export function useGatewayBase(): string {
  return React.useSyncExternalStore(subscribe, getSnapshot).gatewayBase
}

function subscribe(listener: () => void): () => void {
  subscribers.add(listener)
  return () => {
    subscribers.delete(listener)
  }
}

/** and ：useSyncExternalStore use */
function publish(next: PortSnapshot): void {
  snapshot = next
  for (const listener of subscribers) listener()
}

/** （ and use ； not ） */
function portLabel(): string {
  return snapshot.gatewayBase.replace(/^https?:\/\//, '')
}

/** is in （ use ；null = no ） */
function isReady(): boolean {
  return snapshot.backend?.ready === true
}

/**
 * ， and update use ，。
 *
 * for （Chat Completions / Responses / Anthropic Messages）：
 * clientsupports 1 ， use 1 and 。
 * （ not in 「」 / old DOM） when ， render 。
 */
function paintGatewayAddress(base: string): void {
  const paint = (id: string, text: string) => {
    const el = document.getElementById(id)
    if (el) el.textContent = text
  }
  for (const endpoint of API_ENDPOINTS) {
    paint(endpoint.id, `${endpoint.method ? `${endpoint.method} ` : ''}${base}${endpoint.path}`)
  }
}

/** and request（ and can can when ） */
let syncing = false

/**
 * 1 after and ：、、 from this update。
 *
 * can can use （「」 will use ），** not 1 **：
 * sync new 1 ， can new 。
 */
async function sync(): Promise<void> {
  if (syncing) return
  syncing = true
  try {
    const api = shared().workbuddyDesktop
    if (!api) throw new Error('后端桥不可用')
    const status = await api.getBackendStatus()
    const port = Number(status?.port) || 0
    const current = getSnapshot()
    const next: PortSnapshot = {
      // and 、failureReason can can in this 1 ； by 「 no 」
      backend: status ?? null,
      // web （ / / after ）；，
      // not to ：， not empty not 
      gatewayBase: gatewayBaseFor(port, current.gatewayBase),
    }
    if (next.gatewayBase !== current.gatewayBase) paintGatewayAddress(next.gatewayBase)
    publish(next)
    shared().wbApp?.renderTopbarStatus?.()
  } catch (error) {
    // unavailable（） or use failure： up 1 ， not empty
    // —— empty will none
    console.warn('读取后端状态失败:', errorMessage(error))
  } finally {
    syncing = false
  }
}

/** by app.js render() use ： and ， and up */
function render(): void {
  paintGatewayAddress(snapshot.gatewayBase)
  void sync()
}

/** use （ will new ）； by use to ， as not */
async function restartApp(message?: string): Promise<void> {
  toast(message || t('正在重启…'))
  try {
    await shared().workbuddyDesktop?.restartApp()
    // will ， new by ； this not after 
  } catch (error) {
    toast(t('重启失败：{reason}', { reason: errorMessage(error) }), 'err')
  }
}

/* ─── 1 ： ─────────────────── */

/** 「 use 」 ： by and use */
type EndPhase = 'idle' | 'querying' | 'ending'

const END_LABELS: Record<EndPhase, string> = {
  idle: t('结束占用进程'),
  querying: t('查询中…'),
  ending: t('结束中…'),
}

/**
 * ：， not to 「」 can is in 。
 *
 * ── this 1 （ available and ）──────────────
 * `backend.ready`（ is_ready ），** not can 
 * API**（ when API not ， that when state is null， has this 
 * can ）。 up this use /api/session upstreamConfigured
 * （= has no has available）， is 「 no has 」 「 no 」。
 *
 * props.container is that .sidebar-status （root ）：
 * title（） by this by 。
 */
function SidebarStatus({ container }: { container: HTMLElement }) {
  const { backend, gatewayBase } = React.useSyncExternalStore(subscribe, getSnapshot)
  const [phase, setPhase] = React.useState<EndPhase>('idle')

  const failure = backend?.failure || null
  const port = portLabel()
  const ready = backend?.ready

  // ── and that 1 ： + 「 no 」 ──
  // use portLabel()（ and 1 ）， by after to 
  let dotVariant: StatusDotProps['variant'] = 'muted'
  let text = t('正在检查… · {port}', { port })
  if (ready === true) {
    dotVariant = 'success'
    text = t('网关运行中 · {port}', { port })
  } else if (failure) {
    dotVariant = 'destructive'
    text = t('{label} · {port}', { label: failure.label || t('网关启动失败'), port })
  } else if (ready === false) {
    text = t('网关启动中… · {port}', { port })
  }

  /** （ can can up ） title and ： has 1 */
  const title = failure
    ? failure.message ?? ''
    : ready === true
      ? t('本地网关正在监听 {url}，可直接调用 OpenAI 兼容接口', { url: gatewayBase })
      : t('正在确认网关是否已就绪')

  // failure can can ， when current。
  React.useEffect(() => {
    void sync()
  }, [])

  /**
   * failure：`backend:error` in etc down by 。
   * is 「 in 1 」 —— getBackendStatus as 
   *（ can can in before ）。
   */
  React.useEffect(() => {
    const api = shared().workbuddyDesktop
    const subscriptions = [api?.onBackendError?.(() => { void sync() }), api?.onBackendReady?.(() => { void sync() })]
    return () => { for (const unsubscribe of subscriptions) if (typeof unsubscribe === 'function') unsubscribe() }
  }, [])

  // title in root up （React not ）； not 
  React.useEffect(() => {
    container.title = title
  }, [container, title])

  /**
   * 1 ： use 。
   *
   * before **must use need is **： this is 1 1 will its 
   * ，「 use 」 can can is up no old ， can can is use 
   * 。 and path， use has will 「 not 」。
   */
  async function handleEndOccupant(): Promise<void> {
    if (phase !== 'idle') return
    const api = shared().workbuddyDesktop
    if (!api) return
    setPhase('querying')
    try {
      const info = await api.getPortOccupant()
      const occupant = info?.occupant
      if (!occupant) {
        // no to ： is （ that no has can ）。
        // this not use ， to 。
        toast(t('端口 {port} 上没有查到监听进程，请改用「更换端口」', { port: info?.port ?? '' }), 'err')
        return
      }
      const lines = [
        t('即将结束以下进程：'),
        '',
        t('进程名：{name}', { name: String(occupant.name) }),
        t('PID：{pid}', { pid: String(occupant.pid) }),
        t('路径：{path}', { path: occupant.path || t('（未知）') }),
        '',
        t('结束它会强制退出该程序未保存的数据。确认继续？'),
      ]
      // confirm in Tauri WebView not 、（ etc no has ）——
      // （wbConfirm）；text will and 
      const ask = shared().wbConfirm?.ask
      if (!ask) return
      const confirmed = await ask({
        title: t('结束占用端口的进程'),
        text: lines.join('\n'),
        okText: t('结束进程'),
        okClass: 'danger',
      })
      if (!confirmed) return

      setPhase('ending')
      const result = await api.endPortOccupant()
      if (result?.released) {
        toast(t('已结束进程 {name}（PID {pid}），端口已释放', {
          name: String(occupant.name),
          pid: String(occupant.pid),
        }))
        // ， no （ when ， already ）。
        // 1 is in ， not is use 。
        if (
          await ask({
            title: t('重启程序'),
            text: t('端口已释放。现在重启程序让网关用这个端口启动？'),
            okText: t('重启'),
          })
        ) {
          await restartApp(t('重启中…'))
        } else {
          await sync()
        }
      } else {
        toast(t('进程已结束，但端口仍被占用，建议改用「更换端口」'), 'err')
        await sync()
      }
    } catch (error) {
      toast(t('结束失败：{reason}', { reason: errorMessage(error) }), 'err')
    } finally {
      setPhase('idle')
    }
  }

  return (
    <>
      <div className='status-line'>
        <StatusDot variant={dotVariant} pulse={ready === true} />
        <span className='txt'>{text}</span>
      </div>

      {failure?.conflict ? (
        <div className='status-actions'>
          {failure.canEndOccupant === true ? (
            <Button size='sm' disabled={phase !== 'idle'} onClick={() => void handleEndOccupant()}>
              {END_LABELS[phase]}
            </Button>
          ) : null}
          <Button size='sm' onClick={() => void openPortModal()}>
            {t('更换端口')}
          </Button>
        </div>
      ) : null}
    </>
  )
}

/** React root：（ not ）， 1 */
let sidebarRoot: ReturnType<typeof createRoot> | null = null
let sidebarContainer: HTMLElement | null = null

/**
 * to `#sidebar-status` up 。
 *
 * in this div up （ not ） by file。：
 * · createRoot not will ， replaceChildren()
 * ——otherwise old that will and React and ；（title） not 。
 * · not in （ / bundle use ）：
 * ， and not 。
 */
function mountSidebarStatus(): void {
  const container = document.getElementById('sidebar-status')
  if (sidebarContainer === container) return
  sidebarRoot?.unmount()
  sidebarRoot = null
  sidebarContainer = container
  if (!container) return
  container.replaceChildren()
  sidebarRoot = createRoot(container)
  sidebarRoot.render(<SidebarStatus container={container} />)
}
window.addEventListener('wb:shell-layout', mountSidebarStatus)

/* ─── ：「」（ by need 、）── */

/** ： by and use */
type SavePhase = 'idle' | 'checking' | 'saving'

const SAVE_LABELS: Record<SavePhase, string> = {
  idle: t('保存并重启'),
  checking: t('校验中…'),
  saving: t('保存中…'),
}

type PortModalProps = {
  /** that 1 after （ use 「current」 and ） */
  status: BackendStatus | null
  onClose: () => void
}

function PortModal({ status, onClose }: PortModalProps) {
  const currentPort = Number(status?.port) || FALLBACK_PORT
  // 1 available ：current +1（current that use ）；
  // 65535 already ， 1 
  const [value, setValue] = React.useState(() =>
    currentPort < 65535 ? String(currentPort + 1) : String(currentPort - 1)
  )
  const [phase, setPhase] = React.useState<SavePhase>('idle')
  /** （available / already ） */
  const [hint, setHint] = React.useState('')
  /** （ use failure）： and hint ， as and not */
  const [message, setMessage] = React.useState('')
  /** when 1 ， after use not this */
  const selectOnFocusRef = React.useRef(true)

  const preview = Number(value) || 0
  const statusText = status?.failure?.message || t('当前端口 {port}，网关运行正常。', { port: currentPort })

  /**
   * new and ：**、、**（ old ，）。
   *
   * before after 1 bind （`checkPort`）—— use can can 
   * or in ， that when + will not （setfile down 
   * 1 not ， down not ）。 use and when 1 ，
   * this available is available。changePort will 1 ， that is after 。
   *
   * is when （ old in ）： already must，
   * after restartApp； after setState in already root up will 
   * React ， no has use 。
   */
  async function handleSave(): Promise<void> {
    if (phase !== 'idle') return
    const api = shared().workbuddyDesktop
    if (!api) return
    // 1 ：number value can can is empty / 'abc'（Number → NaN）， 1 by 0
    const port = Number(value) || 0
    if (!port || port < 1024 || port > 65535) {
      setMessage(t('端口需在 1024-65535 之间'))
      return
    }

    setPhase('checking')
    setMessage('')
    setHint('')
    try {
      const check = await api.checkPort(port)
      if (!check?.ok) {
        // unavailable Reason by after to （「 」 ），
        setHint(check?.message || t('该端口不可用'))
        return
      }
      if (check.same) {
        setHint(check.message ?? '')
        return
      }
      setPhase('saving')
      const result = await api.changePort(port)
      if (result?.changed === false) {
        setHint(t('与当前端口相同，无需重启'))
        return
      }
      onClose()
      await restartApp(t('端口已改为 {port}，正在重启…', { port }))
    } catch (error) {
      setMessage(errorMessage(error))
    } finally {
      setPhase('idle')
    }
  }

  return (
    // open（ as true）： 1 by onClose 。Esc / / up ✕
    // by Base UI Dialog （ old document Escape）。
    <Dialog open onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('更换网关端口')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <DialogSection>
            <h3>{t('当前状态')}</h3>
            {/* by after to （ that not down ）， this */}
            <p>{statusText}</p>
          </DialogSection>

          <DialogSection>
            <h3>{t('新端口')}</h3>
            {/* is .field-row（ + remaining）， this use 
                ； by ui/css input[type=number] 130px */}
            <div className='flex flex-wrap items-center gap-2.5'>
              <Label htmlFor='port-input' className='text-[12.5px] whitespace-nowrap text-subtle'>
                {t('端口号')}
              </Label>
              <Input
                id='port-input'
                type='number'
                min={1024}
                max={65535}
                step={1}
                placeholder={t('例如 3066')}
                autoComplete='off'
                className='max-w-[130px]'
                value={value}
                onChange={event => { setValue(event.currentTarget.value) }}
                onKeyDown={event => { if (event.key === 'Enter') void handleSave() }}
                autoFocus
                // and old when input.select() etc ： by to ，
                // in this 1 
                onFocus={event => {
                  if (!selectOnFocusRef.current) return
                  selectOnFocusRef.current = false
                  event.currentTarget.select()
                }}
              />
            </div>
            {hint ? <p>{hint}</p> : null}
            <p>{t('端口需在 1024-65535 之间。1023 及以下是系统保留范围，普通程序无法监听。')}</p>
          </DialogSection>

          <DialogSection>
            <h3>{t('换端口后要改的地方')}</h3>
            {/* <code> is 、 not ； in t()
                （ and docs-page.tsx 1 ： 1 not ） */}
            <p>
              {t('网关地址会变成 ')}
              <code>{`http://127.0.0.1:${preview || '—'}`}</code>
              {t('。已经把这个地址填进其它工具（Claude Code、Cherry Studio 等）的，需要一并改成新地址，否则那些工具会连不上。')}
            </p>
          </DialogSection>

          {/* （ use failure） 1 in ：toast seconds after no 。
              min-h is 1 ， when not will 。 */}
          <p className='min-h-[18px] text-[11.5px] text-muted-foreground'>{message}</p>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button variant='outline' onClick={onClose}>
            {t('取消')}
          </Button>
          <Button variant='default' disabled={phase !== 'idle'} onClick={() => void handleSave()}>
            {SAVE_LABELS[phase]}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── （ and old 、 1 ）── */

let modalRoot: ReturnType<typeof createRoot> | null = null
let modalHost: HTMLElement | null = null

function unmountPortModal(): void {
  if (modalRoot) {
    modalRoot.unmount()
    modalRoot = null
  }
  if (modalHost) {
    modalHost.remove()
    modalHost = null
  }
}

/**
 * 「」： when and root，。
 *
 * use that （ when is ）， no has （、 or from no sync
 * success） 1 ； not to by ，available。
 * up 1 （ and conc-dialog 1 ）。
 */
async function openPortModal(): Promise<void> {
  let status = getSnapshot().backend
  if (!status) {
    status = (await shared().workbuddyDesktop?.getBackendStatus().catch(() => null)) ?? null
  }
  unmountPortModal()
  modalHost = document.createElement('div')
  document.body.append(modalHost)
  modalRoot = createRoot(modalHost)
  modalRoot.render(<PortModal status={status} onClose={unmountPortModal} />)
}

/* ─── ： + ─────────────────────── */

/**
 * when ： bundle in body （ index.html）， already 。
 * readyState is —— 1 will <script> <head>， not will as 
 * not to #sidebar-status 。
 */
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', mountSidebarStatus, { once: true })
} else {
  mountSidebarStatus()
}

window.wbPortPanel = { render, sync, portLabel, isReady, gatewayBase: () => snapshot.gatewayBase }

declare global {
  interface Window {
    /** （ ui/port-panel.js， and 1 ） */
    wbPortPanel?: {
      /** app.js render() ： and ， and */
      render(): void
      /** 1 after （ / / 20 seconds） */
      sync(): Promise<void>
      /** （ not ）， use */
      portLabel(): string
      /** is in ， use */
      isReady(): boolean
      /**
       * （）。 iframe use for —— 
       * `tauri://` ，`/plugins/...` this for path will to 
       * directory not is 。
       */
      gatewayBase(): string
    }
  }
}
