/**
 * ** + set + **（：`.ts` not ）。
 *
 * from proxies-page.tsx ： this need for —— 、set
 * （ / / for ） and （ + ）， must is 1 ，
 * key in 1 ， and can from this 。
 *
 * ── and 1 ────────────────────────────────────
 * · **set** `window.wbColSettings`（islands/table-col-settings.tsx）：
 * `register` 1 （ 「set」 by ）， use 
 * `visibleColumns()` 「 by use config can 」 and ；
 * · **** `window.wbTableColumns`（ui/table-columns.js）： by `data-col`
 * `<col>` and `<th>`，default in `<col>` up （ no 
 * default）， by inline width、 ；
 * · **current can **（`window.wbProxiesPanel.visibleColumns`），
 * need by 「 to 1 / 1 in after （ ）」。
 *
 * 1 when not will ：config is `visible: false`，key in ，
 * use new when old 。
 */

import { t } from '../i18n'

type Align = 'left' | 'center' | 'right'

/** （`key` and `<th data-col>` / `<col data-col>` ） */
export type ProxyColumn = { key: string; label: string; align?: Align }

/**
 * ： / / / enabled / / 。
 *
 * default for before ： and enabled in （ that is ， in in ）、
 * （ is 1 ）、 its for （ and failureReason is ）。
 * 「」 in that 1 is 、 no has ，set has ——
 * empty will key（ and 「」 1 ）。
 */
export const PROXY_COLUMNS: readonly ProxyColumn[] = [
  { key: 'pick', label: t('选择'), align: 'center' },
  { key: 'name', label: t('代理名称') },
  { key: 'addr', label: t('地址') },
  { key: 'usage', label: t('使用情况') },
  { key: 'state', label: t('启用'), align: 'center' },
  { key: 'test', label: t('最近检测') },
  { key: 'act', label: t('操作'), align: 'right' },
]


export const COL_CLASS: Record<string, string> = {
  pick: 'p-pick',
  name: '',
  addr: 'p-addr',
  usage: 'p-usage',
  state: 'p-state',
  test: 'p-test',
  act: 'p-act',
}

/** set id： and table-columns.js 、wbTableColumns.repaint 1 */
export const COL_SETTINGS_ID = 'proxies'

export type ColSettingsHandle = {
  apply<C extends { key: string }>(columns: C[]): (C & { align: Align })[]
  config(): unknown
}

/**
 * can is / up ， this by 「 + 」，
 * not Window up （file not 1 will TS2717）。
 */
type ColumnsWindow = {
  wbColSettings?: {
    register(spec: {
      id: string
      label?: string
      columns: readonly { key: string; label?: string; align?: Align }[]
      mount?: () => Element | null
      buttonPlacement?: 'first' | 'last'
      onChange?: () => void
    }): ColSettingsHandle
    close?(): void
  }
  wbTableColumns?: { repaint?: (id: string) => void }
}

function columnsWindow(): ColumnsWindow {
  return window as unknown as ColumnsWindow
}

/** （ to not ； can when as null） */
let handle: ColSettingsHandle | null = null

/** set after ： after （onChange from React ） */
let onColumnsChanged: (() => void) | null = null

/**
 * this current can （ use ；set when ）。
 * by ， by —— 1 config not will 1 。
 */
export function visibleColumns(): (ProxyColumn & { align: Align })[] {
  if (handle) return handle.apply([...PROXY_COLUMNS])
  return PROXY_COLUMNS.map(column => ({ ...column, align: column.align ?? 'left' }))
}

/** new for 1 and （、 after ； etc ） */
export function repaintColumnWidths(): void {
  columnsWindow().wbTableColumns?.repaint?.(COL_SETTINGS_ID)
}

/**
 * set（ 1 ） and 1 。
 *
 * `mount` ： by need ** **， in React
 * after use ； in glob in table-col-settings before ， that 1 
 * `window.wbColSettings` no up —— not to return false， by use 1 。
 *
 * @returns already （true） is can no 、 need need （false）
 */
export function setupColumns(onChange: () => void): boolean {
  onColumnsChanged = onChange
  if (handle) return true
  const api = columnsWindow().wbColSettings
  if (!api?.register) return false
  handle = api.register({
    id: COL_SETTINGS_ID,
    label: t('代理池表'),
    columns: PROXY_COLUMNS.map(column => ({
      key: column.key,
      label: column.label,
      align: column.align ?? 'left',
    })),
    mount: () => document.querySelector('.page[data-page="proxies"] .panel-head .head-actions'),
    buttonPlacement: 'last',
    // ： 1 ， and by 1 can （ by that 
    // repaintColumnWidths up ， proxies-page use ）
    onChange: () => onColumnsChanged?.(),
  })
  return true
}

/** set（ when ， in up ） */
export function closeColumnPanel(): void {
  columnsWindow().wbColSettings?.close?.()
}
