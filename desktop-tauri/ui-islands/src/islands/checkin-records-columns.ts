import { t } from '../i18n'

type Align = 'left' | 'center' | 'right'

export type RecordColumn = {
  key: 'time' | 'account' | 'provider' | 'source' | 'outcome' | 'detail'
  label: string
  align?: Align
}

export const RECORDS_TABLE_ID = 'checkin-records'

const RECORD_COLUMNS: RecordColumn[] = [
  { key: 'time', label: t('Time') },
  { key: 'account', label: t('Accounts') },
  { key: 'provider', label: t('Provider') },
  { key: 'source', label: t('Source') },
  { key: 'outcome', label: t('Result') },
  { key: 'detail', label: t('Note') },
]

type ColSettingsHandle = {
  apply<C extends { key: string }>(columns: C[]): (C & { align: Align })[]
}

type ColumnsWindow = {
  wbTableColumns?: { repaint?: (id: string) => void }
}

let handle: ColSettingsHandle | null = null
let buttonHost: Element | null = null
let onColumnsChanged: (() => void) | null = null

export function visibleRecordColumns(): (RecordColumn & { align: Align })[] {
  if (handle) return handle.apply(RECORD_COLUMNS)
  return RECORD_COLUMNS.map(column => ({ ...column, align: column.align ?? 'left' }))
}

export function setupRecordColumns(mount: HTMLElement | null, onChange: () => void): boolean {
  onColumnsChanged = onChange
  if (!mount) return false
  if (handle) {
    // 标签切换会卸载记录页；移动原按钮宿主才能保留其 React 事件委托。
    if (buttonHost && buttonHost.parentElement !== mount) mount.appendChild(buttonHost)
    return true
  }
  const api = window.wbColSettings
  if (!api) return false
  handle = api.register({
    id: RECORDS_TABLE_ID,
    label: t('Check-in records table'),
    columns: RECORD_COLUMNS,
    mount: () => mount,
    onChange: () => onColumnsChanged?.(),
  })
  buttonHost = mount.firstElementChild
  return true
}

export function repaintRecordColumns(): void {
  const shared = window as unknown as ColumnsWindow
  shared.wbTableColumns?.repaint?.(RECORDS_TABLE_ID)
}

export function closeRecordColumns(): void {
  window.wbColSettings?.close()
  onColumnsChanged = null
}

declare global {
  interface Window {
    wbCheckinRecords?: { visibleColumns(): (RecordColumn & { align: Align })[] }
  }
}

window.wbCheckinRecords = { visibleColumns: visibleRecordColumns }
