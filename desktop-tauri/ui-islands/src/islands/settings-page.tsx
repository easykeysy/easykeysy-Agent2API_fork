import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { providerPromptRows } from './settings-prompt-rows'
import { flushSync } from 'react-dom'
import { createRoot } from 'react-dom/client'
import {
  Badge,
  Button,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  InputGroup,
  InputGroupInput,
  Label,
  NavList,
  NavItem,
  SegmentedControl,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  CardHeader, Card, CardContent } from '@ui'
import {
  CATEGORIES,
  CAPTCHA_POOL_FIELDS,
  IMAGE_COMPRESS_FIELDS,
  modelsDevStateText,
  NOTES,
  PROMPT_MODES,
  QUEUE_FIELDS,
  RETENTION_FIELDS,
  RETRY_FIELDS,
  STATES,
  THEME_EVENT,
  THEME_MODES,
  TIPS,
  TIMEOUT_FIELDS,
  ZOOM_EVENT,
  ZOOM_PERCENTS,
  formatBytes,
  formatCount,
  openExternal,
  readThemeMode,
  readZoomPercent,
  shared,
  type GatewayBlocks,
  type NumberField,
  type ThemeMode,
} from './settings-model'
import {
  addProviderPrompt,
  addRetryCode,
  applyUnits,
  cancelLanRegister,
  clearDegrade,
  closePasswordDialog,
  dropLastRetryCode,
  exportAccounts,
  importAccounts,
  load,
  openPasswordDialog,
  panelLogout,
  refreshDebug,
  refreshDeveloperRoleCompat,
  refreshImageCompress,
  refreshClineHeaders,
  refreshZcodeVersion,
  refreshCors,
  refreshCaptchaPool,
  refreshModelsDev,
  refreshPrompt,
  refreshQueue,
  refreshRetention,
  refreshRetry,
  refreshStorage,
  refreshTimeouts,
  removeProviderPrompt,
  removeRetryCode,
  renderDebug,
  renderCors,
  renderPrompt,
  renderRetention,
  renderRetry,
  renderSanitize,
  renderSettings,
  renderStorage,
  resolveLanConfirm,
  resolveRetentionConfirm,
  restoreCategory,
  saveCaptcha,
  saveClineHeaders,
  saveZcodeVersion,
  saveCors,
  saveCaptchaPoolField,
  saveDebug,
  saveImageCompressEnabled,
  saveImageCompressField,
  saveModelsDev,
  savePromptFile,
  savePromptMode,
  saveProviderGatewayPrompt,
  saveProviderPromptFile,
  saveProviderPromptMode,
  saveQueueField,
  saveRetentionField,
  saveRetryField,
  saveDeveloperRoleCompat,
  saveTimeoutField,
  saveToggle,
  selectCategory,
  showCategory,
  submitLanRegister,
  submitPasswordChange,
  toggleLan,
  toggleLanPanel,
  useSettings,
  type ClineHeadersState,
  type DebugState,
  type LanConfirm,
  type LanRegister,
  type LoadStatus,
  type NumericState,
  type PromptState,
  type ProviderPromptOption,
  type ProviderPromptState,
  type SettingsSnapshot,
  type StorageState,
} from './settings-state'
// and use 1 store：set、，
// must 1 （ in this localStorage.setItem 1 ）
import {
  readGroupTotalFiltered, readNamesHidden, setGroupTotalFiltered, setNamesHidden, subscribe,
} from './accounts-store'
import {
  GatewayTextButton,
  ProviderPromptTextButton,
  PromptTextButton,
  gatewayEditedText,
} from './settings-prompt-editor'
// （set → ）：file、 1 （/api/cloud-sync），
// and set store no has ， this file module header
import { CloudSyncPanel } from './settings-cloud-sync'
import { DbRestoreDialog } from './settings-db-restore'
import { DatabaseBackupDialogs, DatabaseBackupPanel } from './settings-database-backup'
import { SanitizePanel } from './settings-sanitize'
import { ClientIdentificationPanel } from '../../../ui-shared/client-identification-panel'
import { clientIdentificationApi } from './settings-client-identification'
// （set → ）： / / 。 in 
// ui/sidebar-layout.js（， and this use 1 ）， this file module header
import { SidebarPane } from './sidebar-pane'
import { t } from '../i18n'
import { Alert } from '@ui'

/**
 * Agent2API · set（React ）。
 *
 * ui/settings-panel.js（ that 、 by id DOM、 use innerHTML failure 
 * ）。 for and ** 1 **（file）： use 1 not use ——
 * app.js:135 load()、upgrade-panel.js:74 load()、islands/update-panel.tsx:972 showCategory('about')。
 *
 * ── file ──────────────────────────
 * settings-page.tsx（file）= + + for ；settings-state.ts = store and 
 * ；settings-model.ts = / / 。 after is .ts， not will 
 * `islands/*.tsx` glob —— set has file 1 。
 *
 * ── ⚠ 「update」（「」） not file ───────────
 * `.settings-pane[data-cat="about"]` is ** 1 **（update-panel.tsx） ： in 
 * `document.querySelector('.settings-pane[data-cat="about"]')`， to React root
 * up 。 this must to ：
 * ① 1 **empty ** `<div class="settings-pane" data-cat="about" />` —— 1 
 * not can （ will that ），React not can in after DOM 。React for 
 * 「 no has children 」 not will ， need **** this div、
 * not to children、 not in in ， 1 is that ；
 * ② in ** in DOM ** —— mount() （flushSync）。
 * （`.active`） React className： to 「update」 when this div will to active，
 * that ， and old class etc 。
 *
 * ── index.html， ────────────
 * （`.settings-layout` / `.settings-nav` / `.settings-nav-item` /
 * `.settings-panes` / `.settings-pane` / `.panel` / `.panel-head` / `.panel-body` /
 * `.head-actions` / `.settings-switches` / `.settings-state` / `.retention-list` /
 * `.retention-row` / `.retention-input` / `.unit` / `.hint` / `.retention-note` /
 * `.prompt-input` / `.tag-input` / `.tag-chip` / `.tag-x` / `.storage-line` /
 * `.storage-path` / `.storage-counts` / `.storage-count` / `.danger-zone` / `.io-result`），
 * is this 1 not is 「」（ in ui/css/page-settings.css）。
 * ： input → Input、 checkbox → Switch、 select → Select 1 、
 * button → Button、`.badge` → Badge、`#retention-modal` → Dialog。
 * ****（icons.js that set，17px and 
 * ）—— not 「」limit： 1 、。
 *
 * `[data-tip]` by Tooltip；only。
 * ， not 。
 */

/* ─── ───────────────────────────────── */

/**
 * （icons.js SVG ）： use 1 ， this 。
 * in set（） already in icons.js； not to when returnempty（empty，
 * not and —— by not to is error， not this ）。
 */
function iconHtml(name: string, size: number): string {
  return shared().wbIcons?.icon?.(name, size) || ''
}

/** ：`.badge`（ in …）/ `.badge.ok` / `.badge.bad` → Badge variant */
function StatusBadge({ tone, children }: { tone: 'idle' | 'ok' | 'bad'; children: React.ReactNode }) {
  const variant = tone === 'ok' ? 'success' : tone === 'bad' ? 'destructive' : 'outline'
  return <Badge variant={variant}>{children}</Badge>
}

type PanelHeadProps = {
  title: string
  /** （ use `[data-tip]`） */
  tip?: string
  badge?: React.ReactNode
  actions?: React.ReactNode
}

/** ： + + + （DOM and 1 ） */
function PanelHead({ title, tip, badge, actions }: PanelHeadProps) {
  return (
    <CardHeader className='panel-head'>
      <h2>{title}</h2>
      {tip ? <span className='tip-q' data-tip={tip}></span> : null}
      {badge}
      {actions ? <div className='head-actions'>{actions}</div> : null}
    </CardHeader>
  )
}

/** not ， set。 */
function CollapsiblePanel({ title, tip, badge, actions, children }: PanelHeadProps & { children: React.ReactNode }) {
  return <Collapsible render={<Card />} className='panel panel-fold group/fold'>
    <CardHeader className='panel-head'>
      <CollapsibleTrigger className='h-auto flex-1 justify-start gap-2 px-0 text-left'>
        <span className='transition-transform group-data-open/fold:rotate-90' aria-hidden dangerouslySetInnerHTML={{ __html: iconHtml('caretRight', 14) }} />
        <h2>{title}</h2>
      </CollapsibleTrigger>
      {tip ? <span className='tip-q' data-tip={tip} /> : null}
      {badge}
      {actions ? <div className='head-actions'>{actions}</div> : null}
    </CardHeader>
    <CollapsibleContent keepMounted>{children}</CollapsibleContent>
  </Collapsible>
}

type SwitchRowProps = {
  id: string
  label: string
  checked: boolean
  disabled?: boolean
  /** （ and PanelHead `[data-tip]`，desktop-tooltip.tsx 1 ） */
  tip?: string
  onCheckedChange: (next: boolean) => void
}

/**
 * 1 。 and in 1 `<label class="switch">` （ and old ）：Base UI 
 * Switch will 1 checkbox，label as to —— can 。
 */
function SwitchRow({ id, label, checked, disabled, tip, onCheckedChange }: SwitchRowProps) {
  return (
    <Label className='switch'>
      <Switch id={id} checked={checked} disabled={disabled} onCheckedChange={next => onCheckedChange(next)} />
      <span>{label}</span>
      {tip ? <span className='tip-q' data-tip={tip}></span> : null}
    </Label>
  )
}

type NumberRowProps = {
  field: NumberField
  /** ；null = no to （ use ， old ） */
  value: number | null
  /** （ in ）： use ， and old DOM disabled etc */
  disabled: boolean
  onCommit: (raw: string) => Promise<void>
}

/**
 * 1 set（ / / when use ）。
 *
 * is ** **： when ， / ，
 * （none） 、 to —— this is old that 「 in that 1 not +
 * failure + after by after 」 etc ， is in this 1 。
 */
function NumberRow({ field, value, disabled, onCommit }: NumberRowProps) {
  const [draft, setDraft] = React.useState<string | null>(null)
  const text = draft !== null ? draft : value === null ? '' : String(value)

  async function commit(): Promise<void> {
    const raw = draft
    if (raw === null) return
    try { await onCommit(raw) } finally { setDraft(null) }
  }

  return (
    <div className='retention-row'>
      <Label htmlFor={field.id}>{field.label}</Label>
      <span className='retention-input'>
        <Input
          id={field.id}
          type='number'
          min={field.min}
          max={field.max}
          step={1}
          inputMode='numeric'
          className={field.max > 99999
            ? 'w-[132px] max-w-[132px] text-center tabular-nums'
            : 'w-[92px] max-w-[92px] text-center tabular-nums'}
          value={text}
          disabled={disabled || value === null}
          onChange={event => setDraft(event.target.value)}
          onFocus={() => setDraft(String(value ?? ''))}
          onBlur={() => void commit()}
          // etc 「」： not Enter is change and not 1 ，
          // this blur 1 1 「 already 」 this 1 path
          onKeyDown={event => { if (event.key === 'Enter') event.currentTarget.blur() }}
        />
        <span className='unit'>{field.unit}</span>
      </span>
      <div className='hint'>{field.hint}</div>
    </div>
  )
}

/** ：「 in …」、 to 「 already 」、 not to 「unavailable」 */
function numericBadge(state: NumericState): React.ReactNode {
  if (state.status === 'ready') return <StatusBadge tone='ok'>{t('已生效')}</StatusBadge>
  if (state.status === 'unavailable') return <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
  return <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>
}

/** new by （none button → Button outline ， and 1 ） */
function RefreshButton({ id, onClick }: { id: string; onClick: () => void }) {
  return <Button id={id} variant='outline' onClick={onClick}>{t('刷新')}</Button>
}

/* ─── use ─────────────────────────────── */

/** 「」 ： when to this API */
function lanStateText(snap: SettingsSnapshot): string {
  const { lanAccess, lanPanel, lanIp, port } = snap.app
  if (!lanAccess) return t('未开启：网关只监听 127.0.0.1，仅本机可以访问。')
  const base = `http://${lanIp || t('<本机IP>')}${port ? `:${port}` : ''}`
  const panel = lanPanel ? t('；网页管理面板已一并开放（浏览器打开同一地址）') : ''
  return t('已开启：其他设备把 API 地址指向 {url}{panel}。', { url: `${base}/v1`, panel })
}

function GeneralPane({ snap }: { snap: SettingsSnapshot }) {
  const app = snap.app
  // in 「 to 」 up （ when no has can ），
  // by → → 
  const appState = app.status === 'unavailable'
    ? STATES.appUnavailable
    : app.status === 'loading'
      ? STATES.appLoading
      : !app.closeToTray
        ? STATES.appTrayOff
        : app.lightweightMode ? STATES.appLightOn : STATES.appTrayOn

  return (
    <>
      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('启动与托盘')}
          tip={TIPS.tray}
          badge={app.status === 'ready'
            ? <StatusBadge tone='ok'>{t('已应用')}</StatusBadge>
            : app.status === 'unavailable'
              ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
              : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
        />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            {/* no returnset when can （ old ）： patch，
                success to 「 already use 」， not use */}
            <SwitchRow
              id='settings-close-to-tray'
              label={t('关闭窗口时最小化到托盘')}
              checked={app.closeToTray}
              disabled={snap.busy === 'app'}
              onCheckedChange={next => void saveToggle('tray', next)}
            />
            {/* 「 to 」： when this none， and */}
            <SwitchRow
              id='settings-lightweight'
              label={t('轻量模式（关窗释放界面内存）')}
              tip={TIPS.lightweight}
              checked={app.lightweightMode}
              disabled={snap.busy === 'app' || !app.closeToTray}
              onCheckedChange={next => void saveToggle('lightweight', next)}
            />
            <SwitchRow
              id='settings-autostart'
              label={t('开机自动启动')}
              checked={app.autostart}
              disabled={snap.busy === 'app'}
              onCheckedChange={next => void saveToggle('autostart', next)}
            />
          </div>
          <div className='settings-state'>{appState}</div>
        </CardContent>
      </Card>

      {!snap.panelLogin && (
        <Card render={<section />} className='panel'>
          <PanelHead title={t('局域网访问')} tip={TIPS.lan} />
          <CardContent className='panel-body'>
            <div className='settings-switches'>
              {/* / not ：（→ need need when → set and ），
                   and in settings-state toggleLan / resolveLanConfirm */}
              <SwitchRow
                id='settings-lan-access'
                label={t('允许局域网内的设备访问网关')}
                checked={snap.app.lanAccess}
                disabled={snap.busy === 'lan'}
                onCheckedChange={toggleLan}
              />
              <SwitchRow
                id='settings-lan-panel'
                label={t('同时开放网页管理面板')}
                checked={snap.app.lanPanel}
                disabled={snap.busy === 'lan' || !snap.app.lanAccess}
                onCheckedChange={toggleLanPanel}
              />
            </div>
            <div className='settings-state'>{lanStateText(snap)}</div>
          </CardContent>
        </Card>
      )}

      <Card render={<section />} className='panel'>
        <PanelHead title={t('计量单位')} tip={TIPS.units} />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            <SwitchRow
              id='settings-chinese-units'
              label={t('使用中文单位（亿 / 万）')}
              checked={snap.unitsChinese}
              onCheckedChange={applyUnits}
            />
          </div>
          <div className='settings-state'>
            {snap.unitsChinese ? STATES.unitsOn : STATES.unitsOff}
          </div>
        </CardContent>
      </Card>
    </>
  )
}

/* ─── ─────────────────────────────── */

/** （ in settings-model STATES ） */
function themeStateText(mode: ThemeMode): string {
  if (mode === 'light') return STATES.themeLight
  if (mode === 'dark') return STATES.themeDark
  return STATES.themeSystem
}

/**
 * ： / / 。
 *
 * is ** before ** —— and after confignone， not and settings-state load
 * （ that 1 and after set）， this 。
 *
 * and ** 1 use in app.js**（ settings-model 1 ）：
 * this wbApp.applyTheme / applyZoom， 'wb:theme' / 'wb:zoom' 
 * —— is 1 set 1 ，must；
 * not for ， new 。
 *
 * ：available（data-theme is CSS ， in shim is empty）；
 * use —— Ctrl +/- 。
 */
function DisplayPane() {
  const [theme, setTheme] = React.useState<ThemeMode>(readThemeMode)
  const [zoom, setZoom] = React.useState<number>(readZoomPercent)
  // 「 this 」： and 。
  // and / before （localStorage）， not settings-state 。
  const [groupTotalFiltered, setGroupTotalFilteredState] = React.useState<boolean>(readGroupTotalFiltered)
  const [namesHidden, setNamesHiddenState] = React.useState<boolean>(readNamesHidden)
  // （platform） not is navigator： and set its 1 
  const isWeb = shared().workbuddyDesktop?.platform === 'web'

  // （、index.html use 、 or ）：
  // new ，， not localStorage
  React.useEffect(() => {
    const onTheme = (event: Event) => setTheme(String((event as CustomEvent).detail || 'system') as ThemeMode)
    const onZoom = (event: Event) => setZoom(Number((event as CustomEvent).detail) || 100)
    window.addEventListener(THEME_EVENT, onTheme)
    window.addEventListener(ZOOM_EVENT, onZoom)
    return () => {
      window.removeEventListener(THEME_EVENT, onTheme)
      window.removeEventListener(ZOOM_EVENT, onZoom)
    }
  }, [])

  // ：set、 1 store。 this when ——
  // · store ： that （toggleNamesHidden）/ 1 when ，
  // need （ localStorage not storage ，
  // that 1 will ）；
  // · storage ： 1 / 1 。
  React.useEffect(() => {
    const sync = () => {
      setGroupTotalFilteredState(readGroupTotalFiltered())
      setNamesHiddenState(readNamesHidden())
    }
    window.addEventListener('storage', sync)
    const off = subscribe(sync)
    return () => {
      window.removeEventListener('storage', sync)
      off()
    }
  }, [])

  /** ： + （ 1 store， 1 ） */
  function applyGroupTotalFiltered(next: boolean): void {
    setGroupTotalFiltered(next)
    setGroupTotalFilteredState(next)
  }

  /** ： and that 1 */
  function applyNamesHidden(next: boolean): void {
    setNamesHidden(next)
    setNamesHiddenState(next)
  }

  const zoomLabel = zoom === 100 ? t('100%（默认）') : `${zoom}%`

  return (
    <>
      <Card render={<section />} className='panel'>
        <PanelHead title={t('显示模式')} tip={TIPS.displayTheme} />
        <CardContent className='panel-body'>
          <div>
            <SegmentedControl<ThemeMode>
              aria-label={t('显示模式')}
              options={THEME_MODES}
              value={theme}
              onValueChange={next => shared().wbApp?.applyTheme?.(next)}
            />
          </div>
          <div className='settings-state'>{themeStateText(theme)}</div>
        </CardContent>
      </Card>

      <Card render={<section />} className='panel'>
        <PanelHead title={t('界面缩放')} tip={TIPS.displayZoom} />
        <CardContent className='panel-body'>
          <div className='retention-list'>
            <div className='retention-row'>
              <Label htmlFor='settings-zoom'>{t('缩放比例')}</Label>
              <span className='prompt-input'>
                {/* is 80%–130% 11 ， use down not is ： 1 need can 
                    （ use 「 in 」 when is ）， and before no has Slider。
                     to SelectValue， not value 。 */}
                <Select
                  value={String(zoom)}
                  onValueChange={next => shared().wbApp?.applyZoom?.(Number(next))}
                >
                  <SelectTrigger
                    id='settings-zoom'
                    className='w-[140px]'
                    disabled={isWeb}
                    aria-label={t('界面缩放比例')}
                  >
                    <SelectValue>{zoomLabel}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {ZOOM_PERCENTS.map(percent => (
                      <SelectItem key={percent} value={String(percent)}>
                        {percent === 100 ? t('100%（默认）') : `${percent}%`}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </span>
              <div className='hint'>
                {t('放大或缩小整个界面（文字与控件一起变），效果与浏览器 Ctrl +/- 相同：共 11 档，立即生效并记住。')}
              </div>
            </div>
          </div>
          <div className='settings-state'>
            {isWeb ? STATES.zoomWeb : zoom === 100 ? STATES.zoomDefault : t('当前按 {n}% 显示。', { n: zoom })}
          </div>
        </CardContent>
      </Card>

      <Card render={<section />} className='panel'>
        <PanelHead title={t('语言设置')} tip={TIPS.displayLanguage} />
        <CardContent className='panel-body'>
          <LanguageRow />
          <div className='settings-state'>{STATES.languageHint}</div>
        </CardContent>
      </Card>

      {/* ： / 。 and up 「 this 」 before 
          （localStorage， accounts-store GROUP_TOTAL_FILTERED_KEY and 
          NAMES_HIDDEN_KEY）—— not 、 not ， not settings-state
           after 。 has this （ that 、），
           is 1 。 */}
      <Card render={<section />} className='panel'>
        <PanelHead title={t('账号列表')} tip={TIPS.accountList} />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            <SwitchRow
              id='settings-group-total-filtered'
              label={t('积分合计只统计当前筛选出来的账号')}
              checked={groupTotalFiltered}
              onCheckedChange={applyGroupTotalFiltered}
            />
            <SwitchRow
              id='settings-names-hidden'
              label={t('隐藏账号名（显示成星号）')}
              checked={namesHidden}
              onCheckedChange={applyNamesHidden}
            />
          </div>
          <div className='settings-state'>
            {groupTotalFiltered ? STATES.groupTotalFilteredOn : STATES.groupTotalFilteredOff}
          </div>
        </CardContent>
      </Card>

      {/* ： / / 。 in （kv sidebarLayout
          ， /api/sidebar-layout ），localStorage is ——
           、 not settings-state ， in 「」。
           1 in ui/sidebar-layout.js that ——
          app.js when is 1 。 */}
      <SidebarPane />
    </>
  )
}

/**
 * 「」 1 ： down auto or ， in new 。
 *
 * ── Why not settings-state after ──────────────────────
 * ： not 、 not ，「 / / 」 before 
 * 。 localStorage（ workbuddy-desktop-locale， ui/i18n.js），
 * and after desktop-settings.json none， this not need need state —— will 
 * location.reload()， after by new new 。
 *
 * ── Why not t() ─────────────────────────────────
 * 'auto' is in （）， need ； its label is ** **
 * （English / / 한국어 …）， before this ， wbI18n.LOCALES， not 。
 */
function LanguageRow() {
  const api = window.wbI18n
  const value = api?.rawLocale?.() ?? 'auto'
  const locales = api?.LOCALES ?? []
  const labelOf = (code: string): string => (
    code === 'auto' ? t('跟随系统') : (locales.find(item => item.code === code)?.label ?? code)
  )

  return (
    <div className='retention-list'>
      <div className='retention-row'>
        <Label htmlFor='settings-language'>{t('界面语言')}</Label>
        <span className='prompt-input'>
          {/* and 「」 1 Select ： to SelectValue， not value 。
               onValueChange can can to null（ empty）， this in to empty when */}
          <Select value={value} onValueChange={next => { if (next) api?.setLocale(next) }}>
            <SelectTrigger id='settings-language' className='w-[200px]' aria-label={t('界面语言')}>
              <SelectValue>{labelOf(value)}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='auto'>{t('跟随系统')}</SelectItem>
              {locales.map(item => (
                <SelectItem key={item.code} value={item.code}>{item.label}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </span>
      </div>
    </div>
  )
}

/* ─── ─────────────────────────────── */

/** 「error」 （GitHub Topics ： + ） */
function RetryCodesField({ codes, status, busy }: {
  codes: number[] | null
  status: LoadStatus
  busy: boolean
}) {
  const [text, setText] = React.useState('')
  const inputRef = React.useRef<HTMLInputElement | null>(null)
  // not to set when use （ old ：field.disabled + .disabled + 「—」）
  const locked = status === 'unavailable'

  return (
    <div className='retention-row'>
      <Label htmlFor='settings-retry-no-codes-input'>{t('指定错误码直接换号')}</Label>
      <div className='retention-input'>
        {/* empty = （ is 1 ， old ） */}
        <InputGroup
          className={`h-auto w-[320px] max-w-full flex-wrap gap-1.5 px-2 py-1${locked ? ' opacity-55' : ''}`}
          onClick={() => inputRef.current?.focus()}
        >
          {(codes || []).map(code => (
            <Badge variant='brand' className='tag-chip' key={code}>
              <span className='v'>{code}</span>
              {/* up ✕：， —— use + ，
                  「 can 」 and old 1 */}
              <Button
                type='button'
                variant='ghost'
                size='icon-2xs'
                className='tag-x text-muted-foreground hover:bg-destructive-soft hover:text-destructive'
                title={t('删除 {code}', { code })}
                aria-label={t('删除状态码 {code}', { code })}
                onClick={event => { event.stopPropagation(); void removeRetryCode(code) }}
              >
                ✕
              </Button>
            </Badge>
          ))}
          <InputGroupInput
            ref={inputRef}
            id='settings-retry-no-codes-input'
            type='text'
            className='h-6 min-w-[120px] flex-1 px-0'
            placeholder={locked ? '—' : t('输入状态码，回车添加')}
            inputMode='numeric'
            autoComplete='off'
            value={text}
            disabled={locked || busy}
            onChange={event => setText(event.target.value)}
            onKeyDown={event => {
              if (event.key === 'Enter') {
                event.preventDefault()
                void addRetryCode(text)
                setText('') // 提交后立即清空输入框（旧实现同）
                return
              }
              // and GitHub Topics 1 ： as empty when after 1 
              if (event.key === 'Backspace' && !text) void dropLastRetryCode()
            }}
          />
        </InputGroup>
      </div>
      <div className='hint'>{NOTES.retryCodes}</div>
    </div>
  )
}

/** ：filepath（ and ， / ） */
function PromptFileRow({ prompt, locked, busy }: {
  prompt: PromptState
  locked: boolean
  busy: boolean
}) {
  const [draft, setDraft] = React.useState<string | null>(null)

  async function commit(): Promise<void> {
    const raw = draft
    if (raw === null) return
    try { await savePromptFile(raw) } finally { setDraft(null) }
  }

  return (
    <div className='retention-row'>
      <Label htmlFor='settings-prompt-file'>{t('提示词文件')}</Label>
      <span className='prompt-input'>
        <Input
          id='settings-prompt-file'
          type='text'
          placeholder={t('留空 = 用内置默认提示词')}
          value={draft !== null ? draft : prompt.file}
          disabled={locked || busy}
          onChange={event => setDraft(event.target.value)}
          onFocus={() => setDraft(prompt.file)}
          onBlur={() => void commit()}
          onKeyDown={event => { if (event.key === 'Enter') event.currentTarget.blur() }}
        />
      </span>
      <div className='hint'>{NOTES.promptFile}</div>
    </div>
  )
}

/** ： + and + file */
function promptStateText(prompt: PromptState): string {
  if (prompt.status === 'loading') return STATES.appLoading
  if (prompt.status === 'unavailable') return STATES.promptUnavailable
  const source = promptSourceText(prompt.source)
  const parts: string[] = []
  if (prompt.mode === 'passthrough') {
    parts.push(t('客户端 system 原样出站（只靠指纹脱敏改写模板句）。'))
  } else {
    parts.push(
      t('{mode}生效：上游收到的 system 来自{source}{lines}。', {
        mode: prompt.mode === 'custom' ? t('替换') : t('追加'),
        source,
        lines: prompt.lines ? t('（{n} 行）', { n: prompt.lines }) : '',
      }),
    )
  }
  if (prompt.fileError) parts.push(`⚠️ ${prompt.fileError}`)
  return parts.join('')
}

/** in （ use ；'' = no has can ） */
function promptSourceText(source: string): string {
  if (source === 'inline') return t('界面里编辑的正文')
  if (source === 'file') return t('提示词文件')
  if (source === 'builtin') return t('内置默认提示词')
  return ''
}

/** ： in when （ when is 1 and use none ） */
function degradeHint(prompt: PromptState): string {
  return t('已自动切换到最小中性提示词（撞了上游内容拦截，多半是 system 指纹误报），到 {until} 自动解除。', {
    until: prompt.degradeUntilText || STATES.degradeUntilFallback,
  })
    + t('期间本模式自己的提示词不会发出；把提示词改好后可以立即解除。')
}

/**
 * file（ and up that ， by id to for that 1 ）
 */
function ProviderFileRow({ item, label, locked, busy }: {
  item: ProviderPromptState
  label: string
  locked: boolean
  busy: boolean
}) {
  const [draft, setDraft] = React.useState<string | null>(null)

  async function commit(): Promise<void> {
    const raw = draft
    if (raw === null) return
    try {
      // in and 「」 not when ： this 1 can can is 「」 （ is 
      // that ）， not this empty 1 
      if (raw !== item.file) await saveProviderPromptFile(item, raw)
    } finally { setDraft(null) }
  }

  return (
    <span className='prompt-input'>
      <Input
        id={`settings-prompt-file-${item.id}`}
        type='text'
        placeholder={t('留空 = 用内置默认提示词')}
        aria-label={t('{label} 提示词文件', { label })}
        value={draft !== null ? draft : item.file}
        disabled={locked || busy}
        onChange={event => setDraft(event.target.value)}
        onFocus={() => setDraft(item.file)}
        onBlur={() => void commit()}
        onKeyDown={event => { if (event.key === 'Enter') event.currentTarget.blur() }}
      />
    </span>
  )
}

/** 1 ： + and + file（ and that 1 ） */
function providerStateText(item: ProviderPromptState, configured: boolean): string {
  // no （ in 「 has 」 when will ） that ——
  // not this 1 ， use will as this 1 is 「 this set」
  if (!configured) return t('未单独配置：模式、提示词文件与正文都跟随上面的全局配置（当前「{mode}」）。', {
    mode: PROMPT_MODES.find(option => option.value === item.mode)?.toastLabel ?? item.mode,
  })
  if (item.mode === 'passthrough') return t('客户端 system 原样出站。')
  const head = t('{mode}生效：上游收到的 system 来自{source}{lines}。', {
    mode: item.mode === 'custom' ? t('替换') : t('追加'),
    source: promptSourceText(item.source),
    lines: item.lines ? t('（{n} 行）', { n: item.lines }) : '',
  })
  return item.fileError ? `${head} ⚠️ ${item.fileError}` : head
}

/**
 * 1 **** that 1 （ has registry `gatewayNote` has ）。
 *
 * this not client、 not file， is up （ZCode 
 * ）。default、 can ，** can ** —— after can not can 
 * up current ， when this 1 ****（`tone` as ok
 * + ）， not is 1 none 。
 */
function ProviderGatewayRow({ item, option, over, locked, busy }: {
  item: ProviderPromptState
  option: ProviderPromptOption
  /** this 1 already （ ； = is ） */
  over: GatewayBlocks | undefined
  locked: boolean
  busy: boolean
}) {
  const on = item.gateway
  const chars = option.gatewayChars
  const size = chars ? t('约 {n} 字符', { n: chars.toLocaleString('zh-CN') }) : t('一段内置装配')
  const edited = gatewayEditedText(over)
  return (
    <div className='retention-row'>
      <Label className='prompt-gateway-label'>{t('网关自带')}</Label>
      <span className='prompt-input'>
        <SwitchRow
          id={`settings-prompt-gateway-${item.id}`}
          label={t('装上{size}的官方身份提示词', { size })}
          checked={on}
          disabled={locked || busy}
          onCheckedChange={next => void saveProviderGatewayPrompt(item.id, next)}
        />
        {/* and and ：（ not / ） in 1 can ，
             ， 1 not will 1 （ settings-state ） */}
        <GatewayTextButton item={item} option={option} over={over} locked={locked} busy={busy} />
      </span>
      <div className={on ? 'hint' : 'hint prompt-gateway-off'}>
        {on ? option.gatewayNote : t('已关闭。{note}', { note: option.gatewayNote })}
        {edited ? t(' 正文已改：{edited}（其余段用官方原文）。', { edited }) : ''}
      </div>
    </div>
  )
}

/** already 、 or ；not configured from down 。 */
function ProviderPromptRows({ prompt, locked, busy }: {
  prompt: PromptState
  locked: boolean
  busy: boolean
}) {
  const options = prompt.options
  const rows = providerPromptRows(prompt)
  const listed = new Set(rows.map(row => row.id))
  const candidates = options.filter(item => !listed.has(item.id))
  const [pending, setPending] = React.useState('')
  const labelOf = (id: string) => options.find(item => item.id === id)?.label || id
  const optionOf = (id: string) => options.find(item => item.id === id)
  const noteOf = (id: string) => optionOf(id)?.gatewayNote || ''

  return (
    <>
      {rows.map(item => {
        const label = labelOf(item.id)
        const modeLabel = PROMPT_MODES.find(option => option.value === item.mode)?.optionLabel ?? item.mode
        const note = noteOf(item.id)
        const option = optionOf(item.id)
        return (
          <React.Fragment key={item.id}>
            <div className='retention-row'>
              <Label htmlFor={`settings-prompt-mode-${item.id}`}>{label}</Label>
              <span className='prompt-input'>
                <Select
                  value={item.mode}
                  onValueChange={next => {
                    // up ：null = no （empty / ）， not is 1 "null" 。
                    // and file 1 ： this 1 can can in 「」， will 
                    // file from up pathdefault（ saveProviderPromptMode）
                    if (next != null && String(next)) {
                      void saveProviderPromptMode(item, String(next))
                    }
                  }}
                >
                  <SelectTrigger
                    id={`settings-prompt-mode-${item.id}`}
                    className='w-[240px]'
                    disabled={locked || busy}
                    aria-label={t('{label} 的提示词模式', { label })}
                  >
                    <SelectValue>{modeLabel}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {PROMPT_MODES.map(option => (
                      <SelectItem key={option.value} value={option.value}>{option.optionLabel}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <ProviderFileRow item={item} label={label} locked={locked} busy={busy} />
                <ProviderPromptTextButton item={item} label={label} locked={locked} busy={busy} />
                {/* 「default」 in **** up ： no not 
                    （ after when no has this 1 ）， 1 invalid by will as */}
                {item.configured ? (
                  <Button
                    variant='outline'
                    disabled={locked || busy}
                    onClick={() => void removeProviderPrompt(item.id)}
                  >
                    {t('跟随默认')}
                  </Button>
                ) : null}
              </span>
              <div className='hint'>{providerStateText(item, item.configured)}</div>
            </div>
            {/* that （ has registry note has ）—— 1 ，
                 and up that not is 1 ： that client system、 this */}
            {option && note ? (
              <ProviderGatewayRow
                item={item}
                option={option}
                over={prompt.gatewayText[item.id]}
                locked={locked}
                busy={busy}
              />
            ) : null}
          </React.Fragment>
        )
      })}

      <div className='retention-row'>
        <Label htmlFor='settings-prompt-provider-add'>{t('添加提供商')}</Label>
        <span className='prompt-input'>
          {/* Select unsupported（must has 1 value）， 1 is 
              「 need config 」 this ；。
              `next` can can is null（Base UI in 「empty / 」 when is ，
               @base-ui `onValueChange` ）： `String(next)` will 
               "null"， after will 1 「unknown id：null」——
               1 「 no 」 not this 1 failure 。 */}
          <Select
            value={pending}
            onValueChange={next => {
              setPending('')
              const id = next == null ? '' : String(next)
              if (id) void addProviderPrompt(id)
            }}
          >
            <SelectTrigger
              id='settings-prompt-provider-add'
              className='w-[240px]'
              disabled={locked || busy || candidates.length === 0}
              aria-label={t('添加要单独配置的提供商')}
            >
              <SelectValue>
                {pending ? labelOf(pending) : (candidates.length ? t('添加提供商…') : t('全部已配置'))}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {candidates.map(item => (
                <SelectItem key={item.id} value={item.id}>{item.label}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </span>
        <div className='hint'>{NOTES.promptProviders}</div>
      </div>
    </>
  )
}

function PromptPanel({ snap }: { snap: SettingsSnapshot }) {
  const prompt = snap.prompt
  const locked = prompt.status !== 'ready'
  const busy = snap.busy === 'prompt'
  const modeLabel = PROMPT_MODES.find(item => item.value === prompt.mode)?.optionLabel ?? prompt.mode

  return (
    <Card render={<section />} className='panel'>
      <PanelHead
        title={t('系统提示词')}
        tip={TIPS.prompt}
        badge={prompt.status === 'ready'
          ? <StatusBadge tone='ok'>{prompt.mode === 'passthrough' ? t('透传') : t('已接管')}</StatusBadge>
          : prompt.status === 'unavailable'
            ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
            : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
        actions={<RefreshButton id='btn-prompt-refresh' onClick={() => void refreshPrompt()} />}
      />
      <CardContent className='panel-body'>
        <div className='retention-list'>
          {/* 「default」 and 「 」： up that is allconfig 
               use default， down that is —— not when is 1 ，
               use not 、 1 （ page-settings.css .prompt-group）。 */}
          <div className='prompt-group'>
            {t('全局配置')}<span className='note'>{t('所有未单独配置的提供商都用这一份')}</span>
          </div>

          <div className='retention-row'>
            <Label htmlFor='settings-prompt-mode'>{t('模式')}</Label>
            <span className='prompt-input'>
              {/* old is <select>（select.js + wbSelect.sync）； this 
                  Select： is by ，page-settings.css `.prompt-input select{width:240px}`
                   not in ， use （otherwise by 1 ）。
                   to SelectValue， not value 。 */}
              <Select
                value={prompt.mode}
                // 「」 that ：null = no ， not 1 （ that ）
                onValueChange={next => { if (next != null && String(next)) void savePromptMode(String(next)) }}
              >
                <SelectTrigger
                  id='settings-prompt-mode'
                  className='w-[240px]'
                  disabled={locked || busy}
                  aria-label={t('系统提示词模式')}
                >
                  <SelectValue>{modeLabel}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {PROMPT_MODES.map(item => (
                    <SelectItem key={item.value} value={item.value}>{item.optionLabel}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </span>
            <div className='hint'>{NOTES.promptMode}</div>
          </div>

          <PromptFileRow prompt={prompt} locked={locked} busy={busy} />

          {/* ： and file is 1 （），file that 1 。
               by and —— not no */}
          <div className='retention-row'>
            <Label htmlFor='btn-prompt-edit-body'>{t('提示词正文')}</Label>
            <span className='prompt-input'>
              <PromptTextButton prompt={prompt} locked={locked} busy={busy} />
            </span>
            <div className='hint'>
              {NOTES.promptText}
              {prompt.source === 'inline'
                ? t('（当前生效的就是这一份，{n} 行）', { n: prompt.lines })
                : prompt.text.trim()
                  ? t('（已存一份正文，{n} 行；切成「替换 / 追加」后生效）', { n: prompt.lines })
                  : ''}
            </div>
          </div>

          <div className='prompt-group'>
            {t('按提供商配置')}<span className='note'>{t('只列单独配置过的家，其余沿用上面的全局配置')}</span>
          </div>

          <ProviderPromptRows prompt={prompt} locked={locked} busy={busy} />

          {/* in when （ old is hidden） */}
          {prompt.status === 'ready' && prompt.degradeActive ? (
            <div className='retention-row'>
              <Label>{t('内容拦截降级')}</Label>
              <span className='prompt-input'>
                <Button
                  id='btn-prompt-clear-degrade'
                  variant='outline'
                  disabled={busy}
                  onClick={() => void clearDegrade()}
                >
                  {t('立即解除')}
                </Button>
              </span>
              <div className='hint'>{degradeHint(prompt)}</div>
            </div>
          ) : null}
        </div>
        <div className='settings-state'>{promptStateText(prompt)}</div>
        <div className='hint retention-note'>{NOTES.prompt}</div>
      </CardContent>
    </Card>
  )
}

/** ： in after to when */
function debugStateText(debug: DebugState): string {
  if (debug.status === 'loading') return STATES.appLoading
  if (debug.status === 'unavailable') return STATES.debugUnavailable
  const stored = debug.count !== null && debug.limit !== null
    ? t('已保存 {count} / {limit} 条报文（超出后丢弃最旧的）。', { count: debug.count, limit: debug.limit })
    : ''
  return debug.on ? `${STATES.debugOn}${stored}${t('凭据类请求头已脱敏。')}` : STATES.debugOff
}

/* ─── / when （ from 「」 ）── */

/** request when （ by settings-model CATEGORIES ） */
function TimeoutPane({ snap }: { snap: SettingsSnapshot }) {
  return (
    <Card render={<section />} className='panel'>
      <PanelHead
        title={t('请求超时')}
        tip={TIPS.timeouts}
        badge={numericBadge(snap.timeouts)}
        actions={<RefreshButton id='btn-timeouts-refresh' onClick={() => void refreshTimeouts()} />}
      />
      <CardContent className='panel-body'>
        <div className='retention-list'>
          {TIMEOUT_FIELDS.map(field => (
            <NumberRow
              key={field.key}
              field={field}
              value={snap.timeouts.values?.[field.key] ?? null}
              disabled={snap.busy === 'timeouts'}
              onCommit={raw => saveTimeoutField(field, raw)}
            />
          ))}
        </div>
        <div className='hint retention-note'>{NOTES.timeouts}</div>
      </CardContent>
    </Card>
  )
}

/** request：parameter + 「error」（ use 1 ） */
function RetryPane({ snap }: { snap: SettingsSnapshot }) {
  return (
    <Card render={<section />} className='panel'>
      <PanelHead
        title={t('请求重试')}
        tip={TIPS.retry}
        badge={numericBadge(snap.retry)}
        actions={<RefreshButton id='btn-retry-refresh' onClick={() => void refreshRetry()} />}
      />
      <CardContent className='panel-body'>
        <div className='retention-list'>
          {RETRY_FIELDS.map(field => (
            <NumberRow
              key={field.key}
              field={field}
              value={snap.retry.values?.[field.key] ?? null}
              disabled={snap.busy === 'retry'}
              onCommit={raw => saveRetryField(field, raw)}
            />
          ))}
          <RetryCodesField
            codes={snap.retryCodes}
            status={snap.retry.status}
            busy={snap.busy === 'codes' || snap.busy === 'retry'}
          />
        </div>
        <div className='hint retention-note'>{NOTES.retry}</div>
      </CardContent>
    </Card>
  )
}

/* ─── ─────────────────────────────── */

function GatewayPane({ snap, panels }: { snap: SettingsSnapshot; panels: Set<string> }) {
  return (
    <>
      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('排队等待')}
          tip={TIPS.queue}
          badge={numericBadge(snap.queue)}
          actions={<RefreshButton id='btn-queue-refresh' onClick={() => void refreshQueue()} />}
        />
        <CardContent className='panel-body'>
          <div className='retention-list'>
            {QUEUE_FIELDS.map(field => (
              <NumberRow
                key={field.key}
                field={field}
                value={snap.queue.values?.[field.key] ?? null}
                disabled={snap.busy === 'queue'}
                onCommit={raw => saveQueueField(field, raw)}
              />
            ))}
          </div>
          <div className='hint retention-note'>{NOTES.queue}</div>
        </CardContent>
      </Card>

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('图片压缩')}
          tip={TIPS.imageCompress}
          badge={snap.imageCompress.status === 'unavailable'
            ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
            : <StatusBadge tone={snap.imageCompress.enabled ? 'ok' : 'idle'}>
                {snap.imageCompress.enabled ? t('已开启') : t('已关闭')}
              </StatusBadge>}
          actions={<RefreshButton id='btn-image-compress-refresh' onClick={() => void refreshImageCompress()} />}
        />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            <SwitchRow
              id='settings-image-compress'
              label={t('转发前压缩对话里内嵌的大图（只改发给上游的副本，生图与图片编辑不受影响）')}
              checked={snap.imageCompress.enabled}
              // to after before not （ and developerRoleCompat 1 ）
              disabled={snap.imageCompress.status !== 'ready' || snap.busy === 'imageCompress'}
              onCheckedChange={next => void saveImageCompressEnabled(next)}
            />
          </div>
          <div className='retention-list'>
            {IMAGE_COMPRESS_FIELDS.map(field => (
              <NumberRow
                key={field.key}
                field={field}
                value={snap.imageCompress.values?.[field.key] ?? null}
                disabled={snap.busy === 'imageCompress'}
                onCommit={raw => saveImageCompressField(field, raw)}
              />
            ))}
          </div>
          <div className='hint retention-note'>{NOTES.imageCompress}</div>
        </CardContent>
      </Card>

      {/* ： in ZCode ， has set。
           no （ no / no ZCode） when this not 。 */}
      {panels.has('captcha_pool') ? (
        <Card render={<section />} className='panel'>
          <PanelHead
            title={t('验证码令牌池')}
            tip={TIPS.captchaPool}
            badge={numericBadge(snap.captchaPool)}
            actions={
              <RefreshButton id='btn-captcha-pool-refresh' onClick={() => void refreshCaptchaPool()} />
            }
          />
          <CardContent className='panel-body'>
            <div className='retention-list'>
              {CAPTCHA_POOL_FIELDS.map(field => (
                <NumberRow
                  key={field.key}
                  field={field}
                  value={snap.captchaPool.values?.[field.key] ?? null}
                  disabled={snap.busy === 'captchaPool'}
                  onCommit={raw => saveCaptchaPoolField(field, raw)}
                />
              ))}
            </div>
            <div className='hint retention-note'>{NOTES.captchaPool}</div>
          </CardContent>
        </Card>
      ) : null}

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('models.dev 模型元数据')}
          tip={TIPS.modelsDev}
          badge={snap.modelsDev.status === 'ready'
            ? <StatusBadge tone='ok'>{t('{count} 条', { count: Number(snap.modelsDev.count).toLocaleString('zh-CN') })}</StatusBadge>
            : snap.modelsDev.status === 'unavailable'
              ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
              : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
          actions={
            <RefreshButton id='btn-modelsdev-refresh' onClick={() => void refreshModelsDev()} />
          }
        />
        <CardContent className='panel-body'>
          <SwitchRow
            id='settings-modelsdev'
            label={t('用 models.dev 补齐上游没声明的模型能力与思考档位（联网刷新，内嵌快照始终生效）')}
            checked={snap.modelsDev.enabled}
            disabled={snap.modelsDev.status !== 'ready' || snap.busy === 'modelsDev'}
            onCheckedChange={next => void saveModelsDev(next)}
          />
          <div className='settings-state'>{modelsDevStateText(snap.modelsDev)}</div>
          <div className='hint retention-note'>{NOTES.modelsDev}</div>
        </CardContent>
      </Card>

      <SanitizePanel snap={snap} />

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('开发者角色兼容')}
          tip={TIPS.developerRoleCompat}
          badge={snap.developerRoleCompat.status === 'ready'
            ? <StatusBadge tone='ok'>{t('已生效')}</StatusBadge>
            : snap.developerRoleCompat.status === 'unavailable'
              ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
              : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
          actions={
            <RefreshButton id='btn-role-compat-refresh' onClick={() => void refreshDeveloperRoleCompat()} />
          }
        />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            <SwitchRow
              id='settings-developer-role-compat'
              label={t('转发前将 developer 角色消息转换为 system，兼容不支持 developer 角色的 OpenAI 兼容上游')}
              checked={snap.developerRoleCompat.on}
              // to after before not （otherwise will 「 not after is 」， no ）
              disabled={snap.developerRoleCompat.status !== 'ready' || snap.busy === 'developerRoleCompat'}
              onCheckedChange={next => void saveDeveloperRoleCompat(next)}
            />
          </div>
          <div className='settings-state'>
            {snap.developerRoleCompat.status === 'loading'
              ? STATES.appLoading
              : snap.developerRoleCompat.status === 'unavailable'
                ? STATES.roleCompatUnavailable
                : snap.developerRoleCompat.on ? STATES.roleCompatOn : STATES.roleCompatOff}
          </div>
          <div className='hint retention-note'>{NOTES.developerRoleCompat}</div>
        </CardContent>
      </Card>

      {/* Cline ： and default in ， no not 。
           in 「」 down is ， up 。 */}
      {panels.has('upstream_headers') ? <ClineHeadersPanel snap={snap} /> : null}

      {/* ZCode client： up by down not down 。 and up that not ，
           this 1 ** not ** —— is config（
          api::zcode_version module header）， no need can 。 */}
      <ZcodeVersionPanel snap={snap} />

      <PromptPanel snap={snap} />

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('调试模式')}
          tip={TIPS.debug}
          badge={snap.debug.status === 'ready'
            ? <StatusBadge tone='ok'>{t('已生效')}</StatusBadge>
            : snap.debug.status === 'unavailable'
              ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
              : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
          actions={<RefreshButton id='btn-debug-refresh' onClick={() => void refreshDebug()} />}
        />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            <SwitchRow
              id='settings-debug-mode'
              label={t('保存上游原始报文（请求头、请求体、响应头、响应体）')}
              checked={snap.debug.on}
              disabled={snap.debug.status !== 'ready' || snap.busy === 'debug'}
              onCheckedChange={next => void saveDebug(next)}
            />
          </div>
          <div className='settings-state'>{debugStateText(snap.debug)}</div>
          <div className='hint retention-note'>{NOTES.debug}</div>
        </CardContent>
      </Card>
    </>
  )
}

/* ─── Cline （）───────────────── */

/** can ：default ， can */
type ClineHeaderRow = { key: string; value: string; isDefault: boolean }

/**
 * from can ：default in before （ after to ），
 * in after 。default 「」（empty = use 
 * this ），。
 */
function clineRowsFrom(state: ClineHeadersState): ClineHeaderRow[] {
  const rows: ClineHeaderRow[] = Object.entries(state.defaults).map(([key, value]) => ({
    key,
    value: key in state.overrides ? state.overrides[key] : value,
    isDefault: true,
  }))
  for (const [key, value] of Object.entries(state.overrides)) {
    if (!(key in state.defaults)) rows.push({ key, value, isDefault: false })
  }
  return rows
}

/** is etc （ + ， and none） */
function sameOverrides(a: Record<string, string>, b: Record<string, string>): boolean {
  const keys = Object.keys(a)
  if (keys.length !== Object.keys(b).length) return false
  return keys.every(key => a[key] === b[key])
}

/**
 * Cline ： 1 「 → 」 。
 *
 * in ** **（ and NumberRow 1 to ， is 1 ）；
 * after 1 （ / success / new / failure）。「」 
 * to —— defaultdefault not （ after is 「 
 * 」， not is config）、defaultempty = this not 、must has has 。
 */
function ClineHeadersPanel({ snap }: { snap: SettingsSnapshot }) {
  const state = snap.clineHeaders
  const [rows, setRows] = React.useState<ClineHeaderRow[]>(() => clineRowsFrom(state))
  React.useEffect(() => { setRows(clineRowsFrom(state)) }, [state])

  const busy = snap.busy === 'clineHeaders'
  const ready = state.status === 'ready'

  function updateRow(index: number, patch: Partial<ClineHeaderRow>): void {
    setRows(current => current.map((row, i) => (i === index ? { ...row, ...patch } : row)))
  }

  function buildOverrides(): Record<string, string> {
    const out: Record<string, string> = {}
    for (const row of rows) {
      const key = row.key.trim()
      if (!key) continue
      if (row.isDefault) {
        if (row.value !== (state.defaults[key] ?? '')) out[key] = row.value
      } else if (row.value !== '') {
        out[key] = row.value
      }
    }
    return out
  }

  // and after 「 is 」，** not for **： by （default
  // in before 、 in after ）， after BTreeMap is —— will 
  // 「 default + 1 in before 」 this in success after 
  // as 「 has 」， by 1 。
  const dirty = ready && !sameOverrides(buildOverrides(), state.overrides)

  return (
    <CollapsiblePanel
      title={t('Cline 伪装头')}
      tip={TIPS.clineHeaders}
      badge={state.status === 'ready'
        ? <StatusBadge tone='ok'>{t('已生效')}</StatusBadge>
        : state.status === 'unavailable'
          ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
          : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
      actions={<RefreshButton id='btn-cline-headers-refresh' onClick={() => void refreshClineHeaders()} />}
    >
      <CardContent className='panel-body'>
        <div className='retention-list'>
          {ready
            ? rows.map((row, index) => (
              <div key={String(index)} className='flex items-center gap-2'>
                <Input
                  className='w-[190px] shrink-0 font-mono text-[13px]'
                  value={row.key}
                  readOnly={row.isDefault}
                  disabled={busy}
                  placeholder='X-Custom-Header'
                  onChange={event => updateRow(index, { key: event.target.value })}
                />
                <Input
                  className='flex-1 font-mono text-[13px]'
                  value={row.value}
                  disabled={busy}
                  placeholder={row.isDefault ? t('值（留空 = 不发送）') : t('值')}
                  onChange={event => updateRow(index, { value: event.target.value })}
                />
                <Button
                  variant='ghost'
                  disabled={busy}
                  onClick={() => {
                    if (row.isDefault) {
                      updateRow(index, { value: state.defaults[row.key] ?? '' })
                    } else {
                      setRows(current => current.filter((_, i) => i !== index))
                    }
                  }}
                >
                  {row.isDefault ? t('还原') : t('删除')}
                </Button>
              </div>
            ))
            : null}
          <div className='flex items-center gap-2'>
            <Button
              variant='outline'
              disabled={busy || !ready}
              onClick={() => setRows(current => [...current, { key: '', value: '', isDefault: false }])}
            >
              {t('添加自定义头')}
            </Button>
            <div className='flex-1' />
            <Button
              variant='ghost'
              disabled={busy || !ready || Object.keys(state.overrides).length === 0}
              onClick={() => void saveClineHeaders({})}
            >
              {t('恢复默认')}
            </Button>
            <Button disabled={busy || !ready || !dirty} onClick={() => void saveClineHeaders(buildOverrides())}>
              {t('保存')}
            </Button>
          </div>
        </div>
        <div className='settings-state'>
          {state.status === 'loading'
            ? STATES.appLoading
            : state.status === 'unavailable'
              ? t('未能读取 Cline 伪装头设置，请稍后重试')
              : dirty
                ? t('有未保存的修改')
                : t('{n} 个头将随每个 Cline 请求发送', { n: Object.keys(state.effective).length })
                  + (Object.keys(state.overrides).length > 0
                    ? t('（{n} 项被覆盖）', { n: Object.keys(state.overrides).length })
                    : t('（全部为默认值）'))}
        </div>
        <div className='hint retention-note'>{NOTES.clineHeaders}</div>
      </CardContent>
    </CollapsiblePanel>
  )
}

/**
 * ZCode client（ up by down not down ）。
 *
 * ── Why need need use ────────────────────────────────────
 * in `app_version` to when return；
 * after when up is 1 **empty**（`code: 0`， not is error）。 is is 
 * 「 in ， up 1 empty」—— up no this ， is client
 * old （2026-10 ：3.16.0 and down empty，3.16.1 has 1024 ）。
 *
 * that default is ， need 、。 this 1 is 
 * that ： use in this 1 update can ， not etc 。
 */
function ZcodeVersionPanel({ snap }: { snap: SettingsSnapshot }) {
  const state = snap.zcodeVersion
  const [draft, setDraft] = React.useState(state.version)
  React.useEffect(() => { setDraft(state.version) }, [state.version])

  const busy = snap.busy === 'zcodeVersion'
  const ready = state.status === 'ready'
  const dirty = ready && draft.trim() !== state.version
  // （ after module header）： when up not ，
  // must，otherwise use will for 1 「 no 」 。
  const envShadowed = state.envOverride !== ''

  return (
    <CollapsiblePanel
      title={t('ZCode 客户端版本号')}
      tip={TIPS.zcodeVersion}
      badge={state.status === 'ready'
        ? <StatusBadge tone={envShadowed ? 'idle' : 'ok'}>
          {envShadowed ? t('被环境变量覆盖') : t('已生效')}
        </StatusBadge>
        : state.status === 'unavailable'
          ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
          : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
      actions={<RefreshButton id='btn-zcode-version-refresh' onClick={() => void refreshZcodeVersion()} />}
    >
      <CardContent className='panel-body'>
        <div className='retention-list'>
          <div className='flex items-center gap-2'>
            <Input
              id='settings-zcode-version'
              className='w-[190px] shrink-0 font-mono text-[13px]'
              value={draft}
              disabled={busy || !ready}
              placeholder='3.16.2'
              onChange={event => setDraft(event.target.value)}
            />
            <div className='flex-1' />
            <Button
              variant='ghost'
              disabled={busy || !ready || draft === ''}
              onClick={() => void saveZcodeVersion('')}
            >
              {t('清空（用内置默认值）')}
            </Button>
            <Button disabled={busy || !ready || !dirty} onClick={() => void saveZcodeVersion(draft)}>
              {t('保存')}
            </Button>
          </div>
        </div>
        <div className='settings-state'>
          {state.status === 'loading'
            ? STATES.appLoading
            : state.status === 'unavailable'
              ? t('未能读取 ZCode 版本号设置，请稍后重试')
              : envShadowed
                ? t('环境变量 ZCODE_APP_VERSION={value} 正在生效，上面填的值暂不生效', { value: state.envOverride })
                : draft.trim()
                  ? t('探测与领取时会用 {version} 作为客户端版本号', { version: draft.trim() })
                  : t('未设置，使用插件内置的默认版本号')}
        </div>
        <div className='hint retention-note'>{NOTES.zcodeVersion}</div>
      </CardContent>
    </CollapsiblePanel>
  )
}

/* ─── ─────────────────────────────── */
function SecurityPane({ snap }: { snap: SettingsSnapshot }) {
  // 「」success after ， that when already not in ； has failure need need by 
  const [loggingOut, setLoggingOut] = React.useState(false)

  async function onLogout(): Promise<void> {
    setLoggingOut(true)
    const ok = await panelLogout()
    if (!ok) setLoggingOut(false)
  }

  return (
    <>
      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('网关跨域访问（CORS）')}
          tip={TIPS.cors}
          badge={snap.cors.status === 'ready'
            // and ： this 「」 is not ，
            // when to （bad）， is ok
            ? (snap.cors.on ? <StatusBadge tone='bad'>{t('已开启')}</StatusBadge> : <StatusBadge tone='ok'>{t('已关闭')}</StatusBadge>)
            : snap.cors.status === 'unavailable'
              ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
              : <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>}
          actions={<RefreshButton id='btn-cors-refresh' onClick={() => void refreshCors()} />}
        />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            <SwitchRow
              id='settings-cors'
              label={t('允许浏览器里的页面跨来源调用网关（/v1/*）')}
              checked={snap.cors.on}
              // to after before not （： not after is ）
              disabled={snap.cors.status !== 'ready' || snap.busy === 'cors'}
              onCheckedChange={next => void saveCors(next)}
            />
          </div>
          <div className='settings-state'>
            {snap.cors.status === 'loading'
              ? STATES.appLoading
              : snap.cors.status === 'unavailable'
                ? STATES.corsUnavailable
                : snap.cors.on ? STATES.corsOn : STATES.corsOff}
          </div>
          <div className='hint retention-note'>{NOTES.cors}</div>
        </CardContent>
      </Card>

      {/* to 「」（ and 、）。
           this 「」—— 「current this 1 will 」， and 
           its （、） not is 1 。 */}
      {/* ：only（ use ， no has 「」 ） */}
      {snap.panelLogin ? (
        <Card render={<section />} className='panel' id='panel-login-section'>
          <PanelHead title={t('面板登录')} />
          <CardContent className='panel-body'>
            <div className='hint'>{NOTES.panelLogin}</div>
            <div className='mt-2.5'>
              <Button
                id='btn-panel-logout'
                variant='outline'
                disabled={loggingOut}
                onClick={() => void onLogout()}
              >
                {t('退出登录')}
              </Button>
            </div>
          </CardContent>
        </Card>
      ) : null}
    </>
  )
}

/* ─── ─────────────────────────────── */

/** 1 （ in up 、 in down ） */
function StorageCount({ label, value }: { label: string; value: string }) {
  return (
    <div className='storage-count'>
      <span className='k'>{label}</span>
      <span className='v'>{value}</span>
    </div>
  )
}

function storageBadge(state: StorageState): React.ReactNode {
  if (state.status === 'loading') return <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>
  if (state.status === 'unavailable') return <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
  // 「unavailable」 is to not ， and 「 not to 」 not is 1 （ old ）
  return <StatusBadge tone={state.available ? 'ok' : 'bad'}>
    {state.available ? t('已生效') : t('数据库不可用')}
  </StatusBadge>
}

/**
 * 「」：（）、（ 1 ）、。
 *
 * ── Why ────────────────────────────────────
 * 「 is need 」 by 「 has no has 」（ `access::panel_auth_enabled`），
 * not is 1 。 can ， use is in delete —— that and 
 * 「」 is 1 ， is ， will as 「 is when 」。
 * this ， need by down ，、 after can 。
 */
function AccountPane({ snap }: { snap: SettingsSnapshot }) {
  const account = snap.account
  const captcha = snap.captcha
  const switchDisabled = !captcha.available || snap.busy === 'captcha'
  // is 1 ：used for to 「 in use 」
  const here = captcha.localPanel ? 'desktop' : 'web'

  return (
    <>
      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('管理员账号')}
          badge={
            account.status === 'loading'
              ? <StatusBadge tone='idle'>{t('检测中…')}</StatusBadge>
              : account.status === 'unavailable'
                ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
                : account.registered
                  ? <StatusBadge tone='ok'>{t('已启用登录')}</StatusBadge>
                  : <StatusBadge tone='idle'>{t('未注册')}</StatusBadge>
          }
          actions={
            account.registered ? (
              <Button
                id='btn-account-change-password'
                variant='outline'
                onClick={() => openPasswordDialog()}
              >
                {t('修改密码')}
              </Button>
            ) : null
          }
        />
        <CardContent className='panel-body'>
          <div className='settings-state' id='account-username'>
            {account.status === 'unavailable'
              ? t('未能读取账户信息，请稍后重试')
              : account.registered
                ? t('当前管理员：{name}', { name: account.username || '—' })
                : t('尚未注册管理员：管理接口不需要登录。开启「允许局域网访问」时会引导注册，也可以从登录页注册。')}
          </div>
          {account.registered ? (
            <div className='hint'>
              {t('修改密码需要输入当前密码；修改成功后，其他设备 / 浏览器上的登录会立即失效，当前这条会话保留。')}
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('机器人校验')}
          tip={TIPS.captchaSides}
          badge={!captcha.available
            ? <StatusBadge tone='bad'>{t('不可用')}</StatusBadge>
            : null}
        />
        <CardContent className='panel-body'>
          <div className='settings-switches'>
            <SwitchRow
              id='settings-captcha-desktop'
              label={here === 'desktop'
                ? t('桌面端：登录 / 注册需要 ALTCHA 人机验证（当前正在使用）')
                : t('桌面端：登录 / 注册需要 ALTCHA 人机验证')}
              checked={captcha.desktop}
              disabled={switchDisabled}
              onCheckedChange={next => void saveCaptcha('desktop', next)}
            />
            <SwitchRow
              id='settings-captcha-web'
              label={here === 'web'
                ? t('网页端：登录 / 注册需要 ALTCHA 人机验证（当前正在使用）')
                : t('网页端：登录 / 注册需要 ALTCHA 人机验证')}
              checked={captcha.web}
              disabled={switchDisabled}
              onCheckedChange={next => void saveCaptcha('web', next)}
            />
          </div>
          <div className='hint'>{NOTES.captcha}</div>
        </CardContent>
      </Card>

      <Card render={<section />} className='panel'>
        <PanelHead title={t('重置管理员')} />
        <CardContent className='panel-body'>
          <div className='hint'>
            {t('忘记密码时无法在界面里找回：需要停掉程序后，从数据库删除管理员记录（kv 表的 panelAdmin 键），重启后按首次使用重新注册。具体命令与各形态的库文件路径见 docs 目录下的「重置管理员」文档。')}
          </div>
          <Alert variant='destructive' className='danger-zone'>
            <strong>{t('重置会清空管理员账号与密码，面板将回到「未注册」状态。')}</strong>
          </Alert>
        </CardContent>
      </Card>
    </>
  )
}

function DataPane({ snap }: { snap: SettingsSnapshot }) {
  const io = snap.busy
  const storage = snap.storage
  // not when is after 0， and 「 no has 」 in up none ——
  // 「—」， not use as ；path（file is already ）
  const ready = storage.status === 'ready'
  const counts = ready && storage.available
  const path = ready && storage.file ? storage.file : '—'
  const count = (value: number | null): string => (counts && value !== null ? formatCount(value) : '—')

  return (
    <>
      {/* （ for cc-switch WebDAV / S3 ）：file、
          （ /api/cloud-sync， and set its no has ）。
           in 「 / 」 before —— is 「」 ，
           use ， up 。 */}
      <CloudSyncPanel />

      <DatabaseBackupPanel />

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('账号导入 / 导出')}
          tip={TIPS.io}
          actions={
            <>
              {/* and old guard() ： down that by use and ，
                   1 can will down */}
              <Button
                id='btn-settings-export'
                variant='outline'
                disabled={io === 'export'}
                onClick={() => void exportAccounts()}
              >
                {io === 'export' ? t('导出中…') : t('导出账号')}
              </Button>
              <Button
                id='btn-settings-import'
                variant='default'
                disabled={io === 'import'}
                onClick={() => void importAccounts()}
              >
                {io === 'import' ? t('导入中…') : t('导入账号')}
              </Button>
            </>
          }
        />
        <CardContent className='panel-body'>
          <Alert variant='destructive' className='danger-zone'>
            <strong>{t('导出文件内含 accessToken / refreshToken / apiKey 等凭证与自定义提供商定义')}</strong>
            {t('，可直接用于登录。请妥善保管，不要外传或上传到公共位置。')}
          </Alert>
          {/* failure（ old innerHTML and display:none empty， this ） */}
          {snap.ioFailure ? (
            <div className='io-result'>
              <span className='text-destructive'>
                {t('失败 {n} 个：{detail}', { n: snap.ioFailure.failed, detail: snap.ioFailure.detail })}
                {snap.ioFailure.more ? t(' 等') : ''}
              </span>
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('数据保留')}
          tip={TIPS.retention}
          badge={numericBadge(snap.retention)}
          actions={<RefreshButton id='btn-retention-refresh' onClick={() => void refreshRetention()} />}
        />
        <CardContent className='panel-body'>
          <div className='retention-list'>
            {RETENTION_FIELDS.map(field => (
              <NumberRow
                key={field.key}
                field={field}
                value={snap.retention.values?.[field.key] ?? null}
                disabled={snap.busy === 'retention'}
                onCommit={raw => saveRetentionField(field, raw)}
              />
            ))}
          </div>
          <div className='hint retention-note'>{NOTES.retention}</div>
        </CardContent>
      </Card>

      <Card render={<section />} className='panel'>
        <PanelHead
          title={t('数据存储')}
          tip={TIPS.storage}
          badge={storageBadge(storage)}
          actions={<RefreshButton id='btn-storage-refresh' onClick={() => void refreshStorage()} />}
        />
        <CardContent className='panel-body'>
          <div className='retention-list'>
            <div className='retention-row'>
              <Label htmlFor='storage-db-path'>{t('数据库文件')}</Label>
              <span className='storage-line'>
                {/* path（ up is ，path will ） */}
                <span className='storage-path' id='storage-db-path' title={path}>{path}</span>
              </span>
              <div className='hint'>{NOTES.storageFile}</div>
            </div>
            <div className='retention-row'>
              <Label htmlFor='storage-db-size'>{t('占用大小')}</Label>
              <span className='storage-line'>
                <span className='storage-path' id='storage-db-size'>
                  {counts ? formatBytes(storage.bytes) : '—'}
                </span>
              </span>
              <div className='hint'>{NOTES.storageSize}</div>
            </div>
          </div>
          <div className='retention-list storage-counts'>
            <StorageCount label={t('账号')} value={count(storage.accounts)} />
            <StorageCount label={t('事件日志')} value={count(storage.logs)} />
            <StorageCount label={t('请求记录')} value={count(storage.requests)} />
            <StorageCount label={t('报表天数')} value={count(storage.dailyDays)} />
            <StorageCount label={t('上游调试报文')} value={count(storage.upstreamRaw)} />
            <StorageCount label={t('下游正文')} value={count(storage.downstreamRaw)} />
          </div>
          <div className='hint retention-note'>{NOTES.storage}</div>
        </CardContent>
      </Card>
    </>
  )
}

/* ─── and need ───────────────────────── */

/**
 * ， to GitHub issue 。
 * use default（`openExternal` → `open_release_page`，
 * http(s)）： use webview will ， and issue need need use GitHub 。
 */
const FEEDBACK_LINKS = [
  {
    title: t('问题反馈'),
    desc: t('遇到 Bug、报错或异常行为'),
    cta: t('去反馈'),
    url: 'https://github.com/aimod-cc/Agent2API/issues/new?template=bug_report.yml',
  },
  {
    title: t('功能建议'),
    desc: t('想要的新功能或改进想法'),
    cta: t('提建议'),
    url: 'https://github.com/aimod-cc/Agent2API/issues/new?template=feature_request.yml',
  },
  {
    title: t('请求提供商 / 模型支持'),
    desc: t('希望接入新的提供商或模型'),
    cta: t('去申请'),
    url: 'https://github.com/aimod-cc/Agent2API/issues/new?template=provider_request.yml',
  },
] as const

function FeedbackPane() {
  return (
    <Card render={<section />} className='panel'>
      <PanelHead title={t('反馈与需求')} />
      <CardContent className='panel-body'>
        <div className='hint'>
          {t('点下面的入口会用系统默认浏览器打开 GitHub 的对应表单（需要 GitHub 账号，模板已预设好，填完直接提交即可）。')}
        </div>
        <div className='mt-3 flex flex-col gap-2'>
          {FEEDBACK_LINKS.map(item => (
            <div
              key={item.url}
              className='flex items-center justify-between gap-3 rounded-md border border-hairline bg-surface-2 px-3 py-2.5'
            >
              <div className='min-w-0'>
                <div className='text-[12.5px] text-foreground'>{item.title}</div>
                <div className='mt-0.5 text-[12px] text-subtle'>{item.desc}</div>
              </div>
              <Button variant='outline' size='sm' onClick={() => void openExternal(item.url)}>
                {item.cta}
              </Button>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}

/* ─── ───────────────────── */

/**
 * （ old is index.html `#retention-modal` + 1 class ）。
 *
 * use Dialog not is AlertDialog： old （ / / up ✕ / / Esc）
 * has 「」， is ** can ** ， not is AlertDialog that must 。
 * in 「」 not is ： this is not can delete， not this etc delete。
 */
function RetentionConfirmDialog({ confirm }: { confirm: { head: string } | null }) {
  const cancelRef = React.useRef<HTMLButtonElement | null>(null)

  return (
    <Dialog open={confirm !== null} onOpenChange={next => { if (!next) resolveRetentionConfirm(false) }}>
      {/* old .modal-confirm to 440px（ this has 1 by ，620px ） */}
      <DialogContent className='w-[min(440px,calc(100vw-48px))]' initialFocus={cancelRef}>
        <DialogHeader>
          <DialogTitle>{t('确认缩减数据保留')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <Alert variant='destructive' className='danger-zone'>
            {confirm?.head}
            <br />
            <strong>{t('超出的历史数据会被立即删除，且不可恢复。')}</strong>{t('确定继续？')}</Alert>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button ref={cancelRef} variant='outline' onClick={() => resolveRetentionConfirm(false)}>
            {t('取消')}
          </Button>
          <Button variant='destructive' onClick={() => resolveRetentionConfirm(true)}>
            {t('继续并清理')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/* ─── / ─────────────── */

/** （ / / ） 、 and */
const LAN_CONFIRM_COPY: Record<
  NonNullable<LanConfirm>['mode'],
  { title: string; body: (panel: boolean) => React.ReactNode; label: string }
> = {
  enable: {
    title: t('开启局域网访问'),
    body: () => (
      <>
        {t('为了安全，开启前需要先注册一个')}<b>{t('面板管理员账号')}</b>{t('（已注册过会跳过这一步，直接生效）。')}<br />
        {t('开启后网关将监听所有网卡，同一局域网内的设备即可把 API 地址指向本机 IP 一起使用；管理接口从此要求管理员会话或网关 Key，转发接口在没有一把启用的 Key 时也会拒绝服务（届时会自动创建一把名为「默认」的 Key）。')}<br />
        <strong>{t('保存后应用将重启以生效。')}</strong>{t('确定继续？')}
      </>
    ),
    label: t('继续'),
  },
  disable: {
    title: t('关闭局域网访问'),
    body: () => (
      <>
        {t('关闭后网关回到只监听 127.0.0.1，局域网内的设备将无法继续访问；已配置的网关 Key 与账号都不受影响。')}<br />
        <strong>{t('保存后应用将重启以生效。')}</strong>{t('确定继续？')}
      </>
    ),
    label: t('关闭并重启'),
  },
  panel: {
    title: t('变更网页管理面板'),
    body: panel =>
      panel ? (
        <>
          {t('开放后，局域网内其他设备的浏览器打开本机 IP 即可进入管理面板（需管理员账号登录）。')}<br />
          <strong>{t('保存后应用将重启以生效。')}</strong>{t('确定继续？')}
        </>
      ) : (
        <>
          {t('关闭后，管理界面不再从局域网提供，只有本机的桌面程序可以管理；已开启的 API 转发不受影响。')}<br />
          <strong>{t('保存后应用将重启以生效。')}</strong>{t('确定继续？')}
        </>
      ),
    label: t('保存并重启'),
  },
}

/**
 * 「」：current + new + new 。
 *
 * is **** ref ？ not —— this use state： need need in before 
 * 「 new is 1 」， that 1 in that 1 ， ref and state etc ，
 * can 「 not 1 」 in when （errorempty）。
 * in 「」： not this as （ and 1 to ）。
 */
function PasswordDialog({ password }: { password: { busy: boolean; error: string } | null }) {
  const cancelRef = React.useRef<HTMLButtonElement | null>(null)
  const [oldValue, setOldValue] = React.useState('')
  const [newValue, setNewValue] = React.useState('')
  const [repeat, setRepeat] = React.useState('')
  const open = password !== null
  const busy = password?.busy === true

  // empty ： not this in state etc down 1 
  React.useEffect(() => {
    if (!open) {
      setOldValue('')
      setNewValue('')
      setRepeat('')
    }
  }, [open])

  const mismatch = repeat.length > 0 && newValue !== repeat

  function submit(): void {
    if (busy) return
    if (mismatch) return
    void submitPasswordChange(oldValue, newValue)
  }

  return (
    <Dialog open={open} onOpenChange={next => { if (!next) closePasswordDialog() }}>
      <DialogContent className='w-[min(440px,calc(100vw-48px))]' initialFocus={cancelRef}>
        <DialogHeader>
          <DialogTitle>{t('修改密码')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          {/* form can 「 old → new 」 this 1 */}
          <form
            onSubmit={event => { event.preventDefault(); submit() }}
            autoComplete='on'
          >
            <Label htmlFor='account-old-password' className='mb-[5px] block text-[12.5px]'>
              {t('当前密码')}
            </Label>
            <Input
              id='account-old-password'
              type='password'
              autoComplete='current-password'
              placeholder={t('输入当前登录用的密码')}
              value={oldValue}
              disabled={busy}
              onChange={event => setOldValue(event.target.value)}
            />
            <Label htmlFor='account-new-password' className='mt-3.5 mb-[5px] block text-[12.5px]'>
              {t('新密码')}
            </Label>
            <Input
              id='account-new-password'
              type='password'
              autoComplete='new-password'
              placeholder={t('至少 8 位')}
              value={newValue}
              disabled={busy}
              onChange={event => setNewValue(event.target.value)}
            />
            <Label htmlFor='account-repeat-password' className='mt-3.5 mb-[5px] block text-[12.5px]'>
              {t('确认新密码')}
            </Label>
            <Input
              id='account-repeat-password'
              type='password'
              autoComplete='new-password'
              placeholder={t('再输入一次新密码')}
              value={repeat}
              disabled={busy}
              onChange={event => setRepeat(event.target.value)}
            />
          </form>
          <div className='mt-3 min-h-5 text-[13px] whitespace-pre-wrap text-destructive'>
            {mismatch ? t('两次输入的新密码不一致') : (password?.error || '')}
          </div>
        </DialogBody>
        <DialogFooter>
          <div className='mr-auto' />
          <Button ref={cancelRef} variant='outline' onClick={() => closePasswordDialog()} disabled={busy}>
            {t('取消')}
          </Button>
          <Button onClick={submit} disabled={busy || mismatch || !oldValue || !newValue || !repeat}>
            {busy ? t('正在修改…') : t('修改密码')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/**
 * （ RetentionConfirmDialog）：
 * · （`confirm` empty）： / / after and 「 need 」；
 * · （`register` empty）： no has when ——
 * is 「」 that 1 ，success by 
 * （`submitLanRegister`）， use not need need 1 。
 * / up ✕ / / Esc 「 not 」：（）。
 * in ： has ， this not is 。
 */
function LanDialog({ confirm, register }: { confirm: LanConfirm; register: LanRegister }) {
  const cancelRef = React.useRef<HTMLButtonElement | null>(null)
  const nameRef = React.useRef<HTMLInputElement | null>(null)
  const passwordRef = React.useRef<HTMLInputElement | null>(null)

  const open = confirm !== null || register !== null
  const close = () => {
    if (register !== null) cancelLanRegister()
    else resolveLanConfirm(false)
  }
  const submitRegister = () => {
    submitLanRegister(nameRef.current?.value ?? '', passwordRef.current?.value ?? '')
  }

  return (
    <Dialog open={open} onOpenChange={next => { if (!next) close() }}>
      <DialogContent className='w-[min(480px,calc(100vw-48px))]' initialFocus={cancelRef}>
        {register !== null ? (
          <>
            <DialogHeader>
              <DialogTitle>{t('注册管理员账号')}</DialogTitle>
            </DialogHeader>
            <DialogBody>
              <div>
                {t('局域网开放后，管理接口要求登录。请设置管理员账号与密码，注册完成会直接继续开启流程。')}
              </div>
              {/* ：autoComplete and */}
              <form
                onSubmit={event => { event.preventDefault(); if (!register.busy) submitRegister() }}
              >
                <Label htmlFor='lan-admin-name' className='mt-3.5 mb-[5px] block text-[12.5px]'>{t('账号')}</Label>
                {/* when by ref （submitLanRegister），ref must up */}
                <Input ref={nameRef} id='lan-admin-name' autoComplete='username' placeholder={t('管理员账号')} disabled={register.busy} />
                <Label htmlFor='lan-admin-password' className='mt-3.5 mb-[5px] block text-[12.5px]'>{t('密码')}</Label>
                <Input
                  ref={passwordRef}
                  id='lan-admin-password'
                  type='password'
                  autoComplete='new-password'
                  placeholder={t('至少 8 位')}
                  disabled={register.busy}
                />
              </form>
              <div className='mt-3 min-h-5 text-[13px] whitespace-pre-wrap text-destructive'>
                {register.error}
              </div>
            </DialogBody>
            <DialogFooter>
              <div className='mr-auto' />
              <Button ref={cancelRef} variant='outline' onClick={cancelLanRegister} disabled={register.busy}>
                {t('取消')}
              </Button>
              <Button onClick={submitRegister} disabled={register.busy}>
                {register.busy ? t('正在注册…') : t('注册并开启')}
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>{confirm ? LAN_CONFIRM_COPY[confirm.mode].title : ''}</DialogTitle>
            </DialogHeader>
            <DialogBody>
              <div>{confirm ? LAN_CONFIRM_COPY[confirm.mode].body(confirm.panel) : null}</div>
            </DialogBody>
            <DialogFooter>
              <div className='mr-auto' />
              <Button ref={cancelRef} variant='outline' onClick={() => resolveLanConfirm(false)}>
                {t('取消')}
              </Button>
              <Button onClick={() => resolveLanConfirm(true)}>
                {confirm ? LAN_CONFIRM_COPY[confirm.mode].label : t('继续')}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

/* ─── ─────────────────────────────────── */

/**
 * current already 。
 *
 * ── Whyset need ────────────────────────────────────
 * this **** in （ `admin.action`），
 * use 。 no when no has default no has and ， is 1 
 * 「failure」。 by ： no not 。
 *
 * ── Why not use useSettings ──────────────────────────
 * that `getAppSettings`（ use set）， is ，
 * not 。 1 `/api/plugins` ， and failure when by 「 no has 」
 * —— can ， not need 1 not empty。
 *
 * `category`： to 「」 when 1 ， use 
 * can to new 。
 */
function useAdminPanels(category: string): Set<string> {
  const [panels, setPanels] = React.useState<Set<string>>(() => new Set())
  React.useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const internals = (window as unknown as {
          __TAURI_INTERNALS__?: {
            invoke?: (command: string, args?: unknown) => Promise<unknown>
          }
        }).__TAURI_INTERNALS__
        if (!internals?.invoke) return
        const result = await internals.invoke('api_request', {
          request: { method: 'GET', path: '/api/plugins', body: null },
        })
        const envelope = result as { data?: { adminPanels?: string[] } } | null
        const list = envelope?.data?.adminPanels
        if (!cancelled && Array.isArray(list)) setPanels(new Set(list))
      } catch {
        // not to by 「 no has 」： 1 empty
      }
    })()
    return () => { cancelled = true }
  }, [category])
  return panels
}

function SettingsPage() {
  const snap = useSettings()
  // （ useAdminPanels ）
  const adminPanels = useAdminPanels(snap.category)
  const panesRef = React.useRef<HTMLDivElement | null>(null)
  /**
   * 「 1 / 」 current。
   *
   * and （≥768px） is false：、 in ，
   * and before 1 —— this in CSS 767px use ，
   * by `.settings-layout.is-detail` then excluding， not 。
   *
   * up ：false = （ 1 ），true = set（）。
   * 、return 1 。 as false（set from ，
   * not use up —— up 「 up that 1 」 no has up down ，）。
   */
  const [mobileDetail, setMobileDetail] = React.useState(false)

  // after （ old is scrollTop）：otherwise up 1 will 
  // to new up ，「update」 in 。scrollReset showCategory ，
  // is 「 1 」（ when load → restoreCategory） will 。
  React.useEffect(() => {
    const panes = panesRef.current
    if (panes) panes.scrollTop = 0
  }, [snap.scrollReset])

  const paneClass = (cat: string): string => (snap.category === cat ? 'settings-pane active' : 'settings-pane')
  const activeCategory = CATEGORIES.find(item => item.id === snap.category) || CATEGORIES[0]

  return (
    <div className={mobileDetail ? 'settings-layout is-detail' : 'settings-layout'}>
      <NavList render={<nav />} className='settings-nav' id='settings-nav'>
        {CATEGORIES.map(item => (
          <NavItem
            key={item.id}
            active={snap.category === item.id}
            className={snap.category === item.id ? 'settings-nav-item active' : 'settings-nav-item'}
            data-cat={item.id}
            icon={<span className='ico' dangerouslySetInnerHTML={{ __html: iconHtml(item.icon, 17) }} />}
            onClick={() => {
              selectCategory(item.id)
              setMobileDetail(true)
            }}
          >
            {item.label}
          </NavItem>
        ))}
      </NavList>

      {/* return： in （≤767px） and already 1 when ，
           by CSS excluding。 current， use in 。
           use caretRight and （icons.js no has use back ，
           as this 1 by add not ）。 */}
      <div className='settings-back-bar'>
        <Button size='default' variant='ghost' type='button' className='settings-back-btn' onClick={() => setMobileDetail(false)}>
          <span className='ico' dangerouslySetInnerHTML={{ __html: iconHtml('caretRight', 17) }} />
          {t('返回')}
        </Button>
        <span className='settings-back-title'>{activeCategory.label}</span>
      </div>

      <div className='settings-panes' ref={panesRef}>
        <div className={paneClass('general')} data-cat='general'>
          <GeneralPane snap={snap} />
        </div>
        {/* AI config already to 「set」
            （assistant-settings-drawer.tsx）—— / is in will 
            ， in set need for 。 */}
        <div className={paneClass('display')} data-cat='display'>
          <DisplayPane />
        </div>
        <div className={paneClass('gateway')} data-cat='gateway'>
          <ClientIdentificationPanel api={clientIdentificationApi} t={t} active={snap.category === 'gateway'} />
          <GatewayPane snap={snap} panels={adminPanels} />
        </div>
        <div className={paneClass('retry')} data-cat='retry'>
          <RetryPane snap={snap} />
        </div>
        <div className={paneClass('timeout')} data-cat='timeout'>
          <TimeoutPane snap={snap} />
        </div>
        <div className={paneClass('security')} data-cat='security'>
          <SecurityPane snap={snap} />
        </div>
        <div className={paneClass('account')} data-cat='account'>
          <AccountPane snap={snap} />
        </div>
        <div className={paneClass('data')} data-cat='data'>
          <DataPane snap={snap} />
        </div>
        <div className={paneClass('feedback')} data-cat='feedback'>
          <FeedbackPane />
        </div>
        {/*
          「」 by （system-info-pane.tsx，
          `.settings-pane[data-cat="system"]`）。 and down 「update」 that 1 ：
          must is **empty **、 and 1 —— that in 
          React root in this div up ，React this need not ，。
           not is file：file already 2300 ， this and set
           no has （ `/api/system-info`）。
        */}
        <div className={paneClass('system')} data-cat='system' />
        {/*
          「update」 （「」，id is about） by 1 （update-panel.tsx）：
           by this ， to React root in this div up 。 this must is 
          **empty **、 and 1 （ not to children、 not in in 、 not ）——
          React for no has children not will DOM ， that 。
           old ：className up active by file by current。
        */}
        <div className={paneClass('about')} data-cat='about' />
      </div>

      <RetentionConfirmDialog confirm={snap.retentionConfirm} />
      <DatabaseBackupDialogs />
      {/* ：** not can **， to （ this file ） */}
      <DbRestoreDialog />
      <PasswordDialog password={snap.account.password} />
      <LanDialog confirm={snap.lanConfirm} register={snap.lanRegister} />
    </div>
  )
}

/* ─── ： index.html has set section ─── */

const PAGE_SELECTOR = '.page[data-page="settings"]'

let pageRoot: ReturnType<typeof createRoot> | null = null

/**
 * React root in `.page[data-page="settings"]` up （ not div： CSS use 
 * `.page[data-page="settings"]` and ）。
 *
 * ── Whymust flushSync ──────────────────────
 * `.settings-pane[data-cat="about"]` is update-panel.tsx ， in ** **
 * `document.querySelector` this ， to root。React 19 `createRoot().render()`
 * is and ， can can in down 1 —— that 1 this pane no DOM，update-panel will 
 * etc DOMContentLoaded， that when already ，「set → update」 will empty。
 * by glob file（settings-page.tsx in update-panel.tsx before ），
 * this after ， 1 can to 。
 *
 * replaceChildren()：React not ， will and 。
 */
function mount(): void {
  if (pageRoot) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  section.replaceChildren()
  pageRoot = createRoot(section)
  flushSync(() => { pageRoot?.render(<SettingsPage />) })
}

// by up （ old in 1 ）， not etc load() ；
// load() will 1 ，「 when 」 。
restoreCategory()

// to when （ island-lazy）： not in ， not 。
mountOnPage('settings', PAGE_SELECTOR, mount)

/* ─── ： for ─────────────────────────── */

declare global {
  interface Window {
    /**
     * set（ ui/settings-panel.js， and 1 ）。
     * use ：app.js:135 to set when load()；upgrade-panel.js:74 after load()；
     * update-panel.tsx:972 「update」showCategory('about')。
     */
    wbSettingsPanel?: {
      load(): Promise<void>
      /** and set（ old renderSettings， for is render） */
      render(data?: unknown): void
      renderRetention(data?: unknown): void
      renderRetry(data?: unknown): void
      renderDebug(data?: unknown): void
      renderSanitize(data?: unknown): void
      renderPrompt(data?: unknown): void
      renderStorage(data?: unknown): void
      showCategory(category?: string | null): void
    }
  }
}

window.wbSettingsPanel = {
  load,
  render: renderSettings,
  renderRetention,
  renderRetry,
  renderDebug,
  renderSanitize,
  renderPrompt,
  renderStorage,
  showCategory,
}

// ：app.js showPage in before already ， if up in set，
// this 1 ， 1 in 「 in …」
if (shared().wbApp?.currentPage === 'settings') void load()
