import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { SegmentedControl, Spinner , Button } from '@ui'
import {
  loadCenter, loadRecords, store, subscribe,
} from './checkin-store'
import { CenterTab } from './checkin-center-tab'
import { ScheduleTab } from './checkin-schedule-tab'
import { MakeupTab } from './checkin-makeup-tab'
import { t } from '../i18n'
import { RecordsTab } from './checkin-records-tab'
import { Badge } from '@ui'

/**
 * Agent2API · to in （）。
 *
 * ── this 1 Why in ， not is ──────────────────────
 * by HTML/CSS/JS， use iframe 。 that is ** before **：
 * iframe not to CSS 、 not to 、 use not tooltip and ，
 * is 1 1 （`/plugins/runtime/*` that by is as ）。
 * —— 、 、 ， 1 1 for not up 。
 *
 * in ：**「 to this 」**（、 、、、empty），
 * **「 to 」**（ 、 has 、 has no has ）。
 * after can ， 1 exe RPC， not UI。 is this 1 and 、
 * is 1 、 1 、 1 —— not will 。
 *
 * ── is ──────────────────────────────────────────
 * and 「」「」， in 。** not no ** ——
 * to is 1 can ， not is 。 no when empty，
 * use ， not is empty not （ that will as can ）。
 *
 * use ； when to and when set。
 * and by is 。
 *
 * ── ──────────────────────────────────────────────────
 * 1 `GET /api/checkin-center` （ `checkin-store.ts`）， use 。
 * not new request，。
 */

const TABS = ['center', 'schedule', 'makeup', 'records'] as const
type Tab = typeof TABS[number]

function tabOptions() {
  return [
    { value: 'center', label: t('签到中心') },
    { value: 'schedule', label: t('定时签到') },
    { value: 'makeup', label: t('定时补签') },
    { value: 'records', label: t('签到记录') },
  ]
}

/** current（ can ： to in need use ） */
let activeTab: Tab = 'center'

/* ─── store ───────────────────────────── */

/** store and in when （，） */
function useStore(): number {
  const [, bump] = React.useReducer((n: number) => n + 1, 0)
  React.useEffect(() => subscribe(() => bump()), [])
  return 0
}

/* ─── ─────────────────────────────────── */

function CheckinPage(): React.ReactElement {
  useStore()
  const [tab, setTab] = React.useState<Tab>(activeTab)

  // + （ need total， 1 ）
  React.useEffect(() => {
    if (!store.loaded) void loadCenter()
    if (!store.recordsLoaded) void loadRecords()
  }, [])

  // to in ： that is 1 「 after will 」 （ and old 1 ）
  React.useEffect(() => {
    const timer = window.setInterval(() => {
      const page = (window as unknown as { wbApp?: { currentPage?: string } }).wbApp?.currentPage
      if (document.hidden || page !== 'checkin') return
      if (store.center?.auto?.running || activeTab === 'schedule' && store.center?.auto?.enabled) void loadCenter()
    }, 3000)
    return () => window.clearInterval(timer)
  }, [])

  const switchTab = (next: string) => {
    const value = (TABS as readonly string[]).includes(next) ? next as Tab : 'center'
    activeTab = value
    setTab(value)
  }

  // total「 to 」 need to 。 not is switchTab
  // to ：（center-tab） not this has ，
  // that will page → tab → page 。
  React.useEffect(() => {
    const onSwitch = (event: Event) => {
      const next = (event as CustomEvent<{ tab?: string }>).detail?.tab
      if (typeof next === 'string') switchTab(next)
    }
    window.addEventListener('wb:checkin-tab', onSwitch)
    return () => window.removeEventListener('wb:checkin-tab', onSwitch)
  }, [])

  const center = store.center
  const error = store.loadError

  return (
    <div className={`ck-page${tab === 'records' ? ' ck-page-records' : ''}`}>
      <div className="ck-tabs">
        <SegmentedControl
          options={tabOptions()}
          value={tab}
          onValueChange={switchTab}
          aria-label="签到中心标签页"
        />
        <span className="ck-tabs-spacer" />
        <AutoBadge />
        <Button size='default' variant='ghost' type="button" className="ck-refresh" onClick={() => { void loadCenter(); void loadRecords() }}>
          刷新
        </Button>
      </div>

      {error ? (
        <div className="ck-page-error">
          <span>{error}</span>
          <Button size='default' variant='ghost' type="button" className="ck-error-retry" onClick={() => { void loadCenter(); void loadRecords() }}>
            重试
          </Button>
        </div>
      ) : null}

      {!store.loaded ? (
        <div className="ck-loading">
          <Spinner className="size-5" />
          <span>正在读取签到数据…</span>
        </div>
      ) : (
        <>
          {tab === 'center' && center ? <CenterTab center={center} /> : null}
          {tab === 'schedule' && center ? <ScheduleTab center={center} /> : null}
          {tab === 'makeup' && center ? <MakeupTab center={center} /> : null}
          {tab === 'records' ? <RecordsTab /> : null}
        </>
      )}
    </div>
  )
}

/**
 * to 。
 *
 * ： in / already / already （ is already ）。 by Badge 1 。
 */
function AutoBadge(): React.ReactElement | null {
  const auto = store.center?.auto
  if (!auto) return null
  if (auto.running) return <Badge variant='warning' shape='tag'>{t('执行中…')}</Badge>
  if (!auto.enabled) return <Badge variant='outline' shape='tag'>{t('自动执行已暂停')}</Badge>
  const rows = (auto.schedules ?? []).filter(entry => entry.enabled).flatMap(entry => entry.providerRows ?? [])
  const pending = rows.filter(row => !row.firedToday).length
  return (
    <Badge variant={pending ? 'brand' : 'success'} shape='tag'>
      {pending ? t('还有 {n} 个平台未执行', { n: pending }) : t('自动执行已开启')}
    </Badge>
  )
}

/* ─── ─────────────────────────────────── */

const PAGE_SELECTOR = '.page[data-page="checkin"]'

let mounted = false

function mount(): void {
  if (mounted) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  mounted = true
  section.replaceChildren()
  createRoot(section).render(<CheckinPage />)
}

mountOnPage('checkin', PAGE_SELECTOR, mount)

declare global {
  interface Window {
    /**
     * to in （， and `wbReport` / `wbAccountsView` 1 ）。
     *
     * `load` by `app.js` in to when ；`refresh` to 「 to after need update
     * usage」 this 1 。
     */
    wbCheckin?: {
      load(): Promise<void>
      refresh(): void
    }
  }
}

window.wbCheckin = {
  load: loadCenter,
  refresh: () => { void loadCenter(); void loadRecords() },
}
