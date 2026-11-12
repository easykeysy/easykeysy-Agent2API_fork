import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { Badge, Button, Input, Switch , Label, Card, CardHeader, CardContent, CardFooter } from '@ui'
import { t } from '../i18n'
import { Empty } from '@ui'
// 「 check to update」 and app.js 1 ， in update 

/**
 * when （）—— 1 ****。
 *
 * is ui/tasks-panel.js（ that use innerHTML .task-item / .badge that 、
 * ）。 for and ** 1 **：`window.wbTasksPanel.load()`，
 * use has ui/app.js to when that 1 ， use 1 not use 。
 *
 * ── and ： index.html has ──────────
 * is to body； to `<section class="page" data-page="tasks">`，
 * empty after React root in ** this section up **， by file JSX 
 * （.panel / .panel-head / .panel-body / .panel-foot） 1 。
 * · Why not 1 div： CSS `.page[data-page="tasks"] .panel` that 
 * （ ui/css/layout.css） by 「.panel is .page 」 —— need 
 * 、 in .task-list up 。 in 1 will this 。
 * · Why React is ：app.js showPage in this section up 
 * class（active）， from not ， not will 1 DOM。
 *
 * ── ： old ， ────────────────────────
 * `.panel` / `.panel-head` / `.head-actions` / `.task-list` / `.task-grid` /
 * `.task-item` / `.task-main` / `.task-title` / `.task-state` / `.task-actions` /
 * `.task-interval` / `.task-providers` / `.log-empty` （ in 
 * ui/css/page-tasks.css）—— not is 「」， Tailwind will this 1 and its 
 * not 1 。 **** 1 @ui：button → Button、.badge → Badge、
 * → Switch、 → Checkbox、 / when → Input。
 *
 * ── this 1 （ has 1 ）──────────────────────────────
 * · ****（ / directory new /
 * new / request new / new ）
 * —— 1 ：`{enabled, interval}`， /api/scheduled-tasks。
 * 「check」 is 、 1 ， config
 * 「updateset」（update-settings.tsx）—— that 1 in （
 * HIDDEN_TASK_IDS）， after when not 。
 * · `runner: "frontend"` （ / request / new ） is 
 * ****， no has 「 up / down 」， not to 「」 by ——
 * by no has can ；config by file to that （ pushAutoRefresh）。
 * · ** to is 1 **， already to 「 to in 」（checkin-page.tsx）——
 * to 、、 and set from has that 1 ； not 
 * （ vs ）， after /api/scheduled-tasks 1 。
 *
 * ── old ： no when update ──────────
 * will in 、 in 。React 1 id
 * by key 、DOM not ，；**** has ：
 * will use to 1 。 old 「update 」
 * ， this use etc ： in by （intervalDrafts） value，
 * in as empty（= use no in ） when —— is 「 or 
 * 」， this when then （ submitInterval）。
 */

/* ─── ─────────────────────────────────── */

/**
 * 1 （GET /api/scheduled-tasks tasks[]）。
 * and after task_json 1 1 for ； / up down / default by after down ， before not 1 。
 */
type IntervalTask = {
  id: string
  label: string
  /** （ tooltip ）， after down */
  description: string
  /** 'seconds' | 'minutes' */
  unit: string
  /** 'backend'（ after ）| 'frontend'（ when ） */
  runner: string
  enabled: boolean
  interval: number
  min: number
  max: number
  defaultInterval: number
  running: boolean
  lastRunAt: number | null
  lastResult: string | null
  /** failure to when （only after ） */
  retryAt: number | null
  nextRunAt: number | null
  /** can not can 「」： after by runner ， before not */
  canRun: boolean
}

/**
 * use to after （ 1 ； to /api/auto-checkin already that 
 * to to in ， checkin-page.tsx）
 */
type TasksBridge = {
  /** GET /api/scheduled-tasks */
  getScheduledTasks(): Promise<{ tasks?: IntervalTask[] } | null | undefined>
  /** PATCH /api/scheduled-tasks/{id}，response is that */
  saveScheduledTask(id: string, patch: Record<string, unknown>): Promise<IntervalTask>
  /** POST /api/scheduled-tasks/{id}/run，response new after that */
  runScheduledTask(
    id: string,
  ): Promise<{ task?: IntervalTask; summary?: string } | null | undefined>
}

/**
 * window up by its / its 。
 *
 * use 「 + 」 not is declare global Window up ：
 * workbuddyDesktop / wbApp / wbLogsPanel this is use ， if 
 * declare 1 ， and will not 1 TS2717 —— and when 
 * 。file wbTasksPanel（file）。
 */
type SharedWindow = {
  workbuddyDesktop?: TasksBridge
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    /** new （ to will ， 1 ） */
    refresh?: () => Promise<void> | void
    /** ： is that ， up */
    renderTopbarStatus?: () => void
    /** current： in 「 use 」 when after */
    readonly currentPage?: string
  }
  wbLogsPanel?: {
    /** ： and to */
    showCategory?: (category: string) => unknown
    applyAutoRefresh?: (task: IntervalTask | null) => unknown
  }
  wbRequestsPanel?: { applyAutoRefresh?: (task: IntervalTask | null) => unknown }
  wbReport?: { applyAutoRefresh?: (task: IntervalTask | null) => unknown }
  wbAccountsView?: {
    syncBalancesSnapshot?: () => unknown
  }
}

function shared(): SharedWindow {
  return window as unknown as SharedWindow
}

/* ─── ─────────────────────────────────── */

/**
 * ** not ** id：check（ after `config::KEY_UPDATE_CHECK`）。
 *
 * in after （ to GitHub、 and old ），
 * is and config「update」 「updateset」 ——
 * and set in 1 。 in load and sync 1 ，
 * 、、 not 。
 */
const HIDDEN_TASK_IDS: ReadonlySet<string> = new Set(['updateCheck'])

/** 1 ： not not （ / / not ） */
function filterVisibleTasks(list: { tasks?: IntervalTask[] } | null | undefined): IntervalTask[] {
  const tasks = Array.isArray(list?.tasks) ? list.tasks : []
  return tasks.filter(task => !HIDDEN_TASK_IDS.has(task.id))
}

/** can when ： and app.js ， not can when not */
const SYNC_MS = 20_000

/**
 * ：（ 1 ）。
 * its `.task-grid` （ for ， page-tasks.css ）。
 * to 1 ， to to in after this 。
 */
const LEAD_CARDS = 1

/** （empty + ）， and old 1 */
const SEP = '　·　'

/* ─── ─────────────────────────────────── */

/** / toast 1 （ wbApp， not in ） */
function toast(message: string, kind?: 'err' | 'ok') {
  shared().wbApp?.toast?.(message, kind)
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** ： by after to （'seconds' | 'minutes'）， before not */
function unitLabel(unit: string): string {
  return unit === 'seconds' ? t('秒') : t('分钟')
}

/** ms when 「 when ::seconds」， up / down */
function clockOf(value: unknown): string {
  const ms = Number(value) || 0
  if (!ms) return ''
  return new Date(ms).toLocaleTimeString('zh-CN', { hour12: false })
}

/** 「 down 」 for ： to 1 when has use （ use is has ） */
function describeNext(value: unknown): string {
  const ms = Number(value) || 0
  if (!ms) return ''
  const diff = ms - Date.now()
  if (diff <= 0) return t('即将执行')
  const seconds = Math.round(diff / 1000)
  if (seconds < 60) return t('{n} 秒后', { n: seconds })
  const minutes = Math.round(seconds / 60)
  if (minutes < 60) return t('{n} 分钟后', { n: minutes })
  const hours = Math.round(minutes / 60)
  if (hours < 24) return t('{n} 小时后', { n: hours })
  return t('{n} 天后', { n: Math.round(hours / 24) })
}

/**
 * 1 。
 *
 * ── is old `updateTask()` that 1 ， not is `taskCard()` that 1 ──
 * in old file not 1 ：「 after 」，update「 no has 」，
 * and up （ after ），「 after 」 is when ——
 * updateTask this 。 this 1 after （ use 20 seconds after will to that ），
 * and up 「failure」（ that is use 1 can 「Why 1 no 」 ）。
 */
function taskStateText(task: IntervalTask): string {
  const backend = task.runner === 'backend'
  const lines: string[] = []
  if (task.lastRunAt) {
    const at = clockOf(task.lastRunAt)
    // up is after ， {result} 、 not 
    lines.push(task.lastResult
      ? t('上次执行 {time}（{result}）', { time: at, result: task.lastResult })
      : t('上次执行 {time}', { time: at }))
  } else {
    lines.push(backend ? t('还没有执行记录') : t('由页面按间隔自动刷新'))
  }
  if (backend && task.enabled && task.nextRunAt) {
    lines.push(t('下次执行 {time}（{relative}）', {
      time: clockOf(task.nextRunAt),
      relative: describeNext(task.nextRunAt),
    }))
  }
  if (backend && task.retryAt && task.retryAt > Date.now()) {
    lines.push(t('冷却中，{relative}重试', { relative: describeNext(task.retryAt) }))
  }
  return lines.join(SEP)
}

/**
 * before ： and after 1 （ by after down ， not will ）。
 * old in before 1 —— 「」 in that 1 ， not is etc 1 400。
 */
function parseInterval(
  task: IntervalTask,
  raw: string,
): { ok: true; value: number } | { ok: false; message: string } {
  const text = String(raw ?? '').trim()
  const unit = unitLabel(task.unit)
  if (!text) {
    return { ok: false, message: t('间隔不能为空（可填 {min}–{max} {unit}）', { min: task.min, max: task.max, unit }) }
  }
  if (!/^\d+$/.test(text)) return { ok: false, message: t('间隔必须是整数') }
  const value = Number(text)
  if (value < task.min || value > task.max) {
    return {
      ok: false,
      message: t('间隔必须在 {min}–{max} {unit} 之间（当前填的是 {value}）', {
        min: task.min,
        max: task.max,
        unit,
        value: text,
      }),
    }
  }
  return { ok: true, value }
}

/**
 * ** before ** config to （ / request / when ）。
 *
 * has this need need ： after by after config（ down 1 ）， before 
 * when in ， has 。
 *
 * optional： that can can no （ up in after ，；
 * 1 failure， this not this ）。 in when will 1 
 * config，failure not will down not 1 。
 */
function pushAutoRefresh(task: IntervalTask) {
  const bridge = shared()
  if (task.id === 'logsAutoRefresh') bridge.wbLogsPanel?.applyAutoRefresh?.(task)
  else if (task.id === 'requestsAutoRefresh') bridge.wbRequestsPanel?.applyAutoRefresh?.(task)
  else if (task.id === 'reportAutoRefresh') bridge.wbReport?.applyAutoRefresh?.(task)
}

/* ─── （ and ）────────── */

/**
 * 「 1 1 」 （ old panelBusy）。
 *
 * in not is state：must in **** to —— 1 
 * down 、 and ，。state update is ， 1 will 
 * 。 has 1 ， not will 。
 */
let panelBusy = false

/** after ：window.wbTasksPanel.load and */
let runLoad: (() => Promise<void>) | null = null
/** before use load()： 1 ， after （file） */
let loadBeforeMount = false

/** for ：app.js to when load() */
async function load() {
  if (!runLoad) {
    loadBeforeMount = true
    return
  }
  await runLoad()
}

/* ─── ───────────────────────────────── */

function TasksPanel() {
  /** 1 to */
  const [tasks, setTasks] = React.useState<IntervalTask[]>([])
  /**
   * is already success to 。
   * use 「 in 」 and 「failure」—— is tasks empty，
   * will 「 after unavailable」 not 「 in …」。
   */
  const [loaded, setLoaded] = React.useState(false)
  /** in 「」 that id（ by 「 in …」； when can can has 1 ） */
  const [runningId, setRunningId] = React.useState<string | null>(null)
  /**
   * ： has = use in ，value as （ not ）。
   * （ / ） after ， is new after —— and old 
   * 「document.activeElement !== interval when 」 etc 。
   */
  const [intervalDrafts, setIntervalDrafts] = React.useState<Record<string, string | undefined>>({})

  function clearIntervalDraft(id: string) {
    setIntervalDrafts(prev => {
      if (!(id in prev)) return prev
      const next = { ...prev }
      delete next[id]
      return next
    })
  }

  /* ─── and ─────────────────────────── */

  /**
   * 1 （ and update）。
   *
   * catch is this 1 failure： loaded = true after 「 after return」
   * failure， not is in in 。
   */
  const loadPanel = React.useCallback(async () => {
    const api = shared().workbuddyDesktop
    try {
      if (!api) throw new Error(t('后端桥不可用'))
      const list = await api.getScheduledTasks()
      setTasks(filterVisibleTasks(list))
      setLoaded(true)
    } catch (error) {
      console.warn('读取定时任务失败:', errorMessage(error))
      setTasks([])
      setLoaded(true) // 标记「尝试过了」，于是空态显示成失败说明而不是加载中
    }
  }, [])

  /**
   * ：update， not （ by React key ）。
   *
   * 20 seconds 1 ， app.js ； not can 、 use not in 、
   * or has 1 / in when not request（ old 1 ）。
   * failure 1 ： down 1 will ， not in 。
   */
  const sync = React.useCallback(async () => {
    if (panelBusy || document.hidden || shared().wbApp?.currentPage !== 'tasks') return
    const api = shared().workbuddyDesktop
    if (!api) return
    try {
      const list = await api.getScheduledTasks()
      setTasks(filterVisibleTasks(list))
    } catch (error) {
      console.warn('同步定时任务状态失败:', errorMessage(error))
    }
  }, [])

  /** ：， when （ down effect） */
  React.useEffect(() => {
    const timer = window.setInterval(() => {
      void sync()
    }, SYNC_MS)
    return () => {
      window.clearInterval(timer)
    }
  }, [sync])

  /** that is （app.js renderTopbarStatus #tasks-badge）：
   * 1 update up ，otherwise need etc down 1 （20 seconds）。 */
  React.useEffect(() => {
    shared().wbApp?.renderTopbarStatus?.()
  }, [tasks, loaded])

  /** to load()；「 before that 」 */
  React.useEffect(() => {
    runLoad = loadPanel
    if (loadBeforeMount) {
      loadBeforeMount = false
      void loadPanel()
    }
    return () => {
      runLoad = null
    }
  }, [loadPanel])

  /* ─── ： ───────────────────── */

  /**
   * 1 。
   *
   * return as （ after will to and in need when default）， not can 
   * 「 by before 」—— that will in after when already 。
   * failure when to after ： can can is use 。
   *
   * `label` is toast use ， use and already t()（ is after down ）。
   */
  async function saveTask(id: string, patch: Record<string, unknown>, label: string) {
    if (panelBusy) return
    const api = shared().workbuddyDesktop
    if (!api) return
    panelBusy = true
    try {
      const saved = await api.saveScheduledTask(id, patch)
      setTasks(prev => prev.map(task => (task.id === id ? saved : task)))
      pushAutoRefresh(saved)
      toast(t('✅ 已更新「{label}」', { label }))
    } catch (error) {
      toast(t('保存失败：{reason}', { reason: errorMessage(error) }), 'err')
      await loadPanel()
    } finally {
      panelBusy = false
    }
  }

  /**
   * （ / ）。
   *
   * and after 1 not request： （ 010） will to this 。
   * not when = after （ old is input.value = task.interval）。
   */
  async function submitInterval(task: IntervalTask, raw: string) {
    const parsed = parseInterval(task, raw)
    if (!parsed.ok) {
      toast(parsed.message, 'err')
      clearIntervalDraft(task.id)
      return
    }
    if (parsed.value === task.interval) {
      clearIntervalDraft(task.id)
      return
    }
    await saveTask(task.id, { interval: parsed.value }, t('{name}间隔', { name: t(task.label) }))
    // success when after return as ；failure when saveTask already loadPanel() —— 
    clearIntervalDraft(task.id)
  }

  /** 1 ** after **（ before by not ， canRun） */
  async function runTask(task: IntervalTask) {
    if (panelBusy) return
    const api = shared().workbuddyDesktop
    if (!api) return
    panelBusy = true
    setRunningId(task.id)
    try {
      const result = await api.runScheduledTask(task.id)
      if (result?.task) {
        const refreshed = result.task
        setTasks(prev => prev.map(item => (item.id === task.id ? refreshed : item)))
      }
      toast(t('✅ {task}：{summary}', {
        task: t(task.label),
        summary: result?.summary || t('已执行'),
      }))
      // new will valid / ， new 
      if (task.id === 'credentialMaintenance') await shared().wbApp?.refresh?.()
    } catch (error) {
      toast(t('执行失败：{reason}', { reason: errorMessage(error) }), 'err')
      await loadPanel()
    } finally {
      panelBusy = false
      setRunningId(null)
    }
  }

  /* ─── ───────────────────────────────── */

  /**
   * 1 。
   *
   * and old 1 1 for （.task-item / .task-main / .task-title / .task-state /
   * .task-actions / .task-interval …）， ； is 
   * `.tip-q` + `data-tip`（desktop-tooltip.tsx use MutationObserver new ，
   * React will ）。
   *
   * ** not ** `data-island-input`： that is 1 （） ，
   * when 1 will （）。 not need need esc()：
   * old is innerHTML， this to React ， not 。
   */
  function taskCard(task: IntervalTask) {
    const unit = unitLabel(task.unit)
    const running = runningId === task.id
    return (
      <div className='task-item' data-task={task.id} key={task.id}>
        <div className='task-main'>
          <div className='task-title'>
            {/* and in 1 label （ and old ）： Switch is 
                ， DOM 1 checkbox，label as 
                 to —— can ， and old 1 。 */}
            <Label className='switch'>
              <Switch
                checked={task.enabled === true}
                onCheckedChange={next => void saveTask(task.id, { enabled: next }, t('{name}开关', { name: t(task.label) }))}
              />
              {/* / by after down （ in ）， t() can */}
              <span className='task-name'>{t(task.label)}</span>
            </Label>
            <span className='tip-q' data-tip={t(task.description)}></span>
            <Badge className='task-badge' variant={task.enabled ? 'success' : 'outline'}>
              {task.enabled ? t('已开启') : t('已关闭')}
            </Badge>
            {task.running ? (
              <Badge className='task-badge' variant='warning'>
                {t('执行中…')}
              </Badge>
            ) : null}
          </div>
          <div className='task-state'>{taskStateText(task)}</div>
        </div>
        <div className='task-actions'>
          <span className='task-interval'>
            <span className='task-interval-label'>{t('每')}</span>

            <Input
              type='number'
              inputMode='numeric'
              min={task.min}
              max={task.max}
              step={1}
              className='w-[74px] max-w-[74px] text-center tabular-nums'
              value={intervalDrafts[task.id] ?? String(task.interval)}
              title={t('可填 {min}–{max} {unit}（默认 {def} {unit}）', {
                min: task.min,
                max: task.max,
                unit,
                def: task.defaultInterval,
              })}
              // onChange 、 not request： to blur / （ submitInterval）
              onChange={event => {
                const value = event.currentTarget.value
                setIntervalDrafts(prev => ({ ...prev, [task.id]: value }))
              }}
              onBlur={event => void submitInterval(task, event.currentTarget.value)}
              // etc 「」： blur 1 path 1 （ old ）
              onKeyDown={event => {
                if (event.key === 'Enter') event.currentTarget.blur()
              }}
            />
            <span className='task-interval-unit'>{unit}</span>
          </span>
          {task.canRun === true ? (
            <Button
              size='sm'
              variant='outline'
              disabled={running || task.running === true}
              onClick={() => void runTask(task)}
            >
              {running ? t('执行中…') : t('立即执行')}
            </Button>
          ) : null}
        </div>
      </div>
    )
  }

  /** 1 no has ：「 in 」 and 「failure / after no return」 */
  const empty = tasks.length === 0
  const enabledCount = tasks.filter(task => task.enabled).length
  const badgeText = empty
    ? loaded
      ? t('不可用')
      : '—'
    : t('{enabled} / {total} 个已开启', { enabled: enabledCount, total: tasks.length })
  /** ：none → none； not to → bad； → has ok */
  const badgeVariant: 'destructive' | 'outline' | 'success' = empty
    ? loaded
      ? 'destructive'
      : 'outline'
    : enabledCount
      ? 'success'
      : 'outline'
  /** use （app.js mirror ， not by className Tailwind ） */
  const badgeTone = badgeVariant === 'destructive' ? 'bad' : badgeVariant === 'success' ? 'ok' : ''

  // ： by after to （ 1 「」， LEAD_CARDS）。
  // its .task-grid （ for ）， is ， not will 。
  const cards = tasks.map(task => taskCard(task))
  const rest = cards.slice(LEAD_CARDS)

  return (
    <Card render={<section />} className='panel'>
      <CardHeader className='panel-head'>
        <h2>{t('任务清单')}</h2>
        {/* id ：app.js renderTopbarStatus will by id this and 
            （ data-tone ， badgeTone） */}
        <Badge id='tasks-badge' variant={badgeVariant} data-tone={badgeTone}>
          {badgeText}
        </Badge>
        <span className='panel-sub'>{t('状态每 20 秒自动同步')}</span>
        <div className='head-actions'>
          {/* old is load().then(() => toast(' when already new '))： not ， 1 already new */}
          <Button variant='outline' onClick={() => void loadPanel().then(() => toast(t('定时任务已刷新')))}>
            {t('刷新')}
          </Button>
        </div>
      </CardHeader>
      <CardContent className='panel-body'>
        <div className='task-list'>
          {empty ? (
            <Empty className='log-empty'>
              {loaded
                ? t('没有可显示的定时任务。后端未返回任务清单，请确认网关正在运行。')
                : t('正在加载定时任务…')}
            </Empty>
          ) : (
            <>
              {cards.slice(0, LEAD_CARDS)}
              {rest.length ? <div className='task-grid'>{rest}</div> : null}
            </>
          )}
        </div>
      </CardContent>
      <CardFooter className='panel-foot'>
        {/* path / is config、 not ； in t()（ and docs-page 1 ） */}
        <span>
          {t('任务配置保存在 ')}<code>~/.agent2api/config.json</code>{t(' 的 ')}<code>scheduledTasks</code>{t(' 字段')}
        </span>
        <span>{t('改动立即生效，不需要重启程序')}</span>
      </CardFooter>
    </Card>
  )
}

/* ─── ： index.html has ─────── */

const PAGE_SELECTOR = '.page[data-page="tasks"]'

let mounted = false

/**
 * React root in up 。
 *
 * （.panel / .panel-head / .task-list …）： this by 
 * down JSX by new ， will and React 。
 */
function mount() {
  if (mounted) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  mounted = true
  section.replaceChildren()
  createRoot(section).render(<TasksPanel />)
}

// to when （ island-lazy）： not in ， not 。
mountOnPage('tasks', PAGE_SELECTOR, mount)

declare global {
  interface Window {
    /** when （ ui/tasks-panel.js， and 1 ） */
    wbTasksPanel?: { load(): Promise<void> }
  }
}

window.wbTasksPanel = { load }

// ：app.js showPage in before （ that when window.wbTasksPanel
// not in ， that use empty）， use up if in ， this 1 —— otherwise 1 in 
// 「 in when …」。 render、 no ，load() will this request in 
// loadBeforeMount up ， by effect 。
if (shared().wbApp?.currentPage === 'tasks') void load()
