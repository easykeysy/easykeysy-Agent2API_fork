import * as React from 'react'
import {
  Badge, Button, Checkbox, Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
  type BadgeProps,
} from '@ui'
import { t } from '../i18n'

export type RefreshItem = {
  provider?: string; providerLabel?: string; status?: string; count?: unknown
  refreshedAt?: unknown; fixed?: boolean; message?: string; accountId?: string
}

export type Row = { provider: string; label: string; item?: RefreshItem }

type Props = {
  custom: boolean
  status: 'idle' | 'loading' | 'error' | 'ready'
  rows: Row[]
  upstream: string[]
  managed: ReadonlySet<string>
  picked: ReadonlySet<string>
  keyword: string
  busy: boolean
  busyRow: string
  onPick: (id: string, checked: boolean) => void
  onFetchRow: (row: Row) => void
  sourceCell: (provider: string, reported?: unknown) => React.ReactNode
  refreshedAt: (provider: string) => number
  formatTime: (value: number) => string
}

// 列宽层按表 id / data-col 定位，卸载失败态后重新挂载时必须恢复列宽与拖动把手。
const TABLE_ID = { intl: 'fm-table-intl', custom: 'fm-table-custom' }
const TABLE_COL_ID = { intl: 'fetch-models', custom: 'fetch-models-custom' }
const COLS_INTL = ['provider', 'source', 'state', 'count', 'note', 'updated', 'op']
const COLS_CUSTOM = ['pick', 'model', 'state']
const WIDTH_INTL: Record<string, string> = {
  provider: 'w-[13%]', source: 'w-[21%]', state: 'w-[9%]',
  count: 'w-[6%]', note: 'w-[20%]', updated: 'w-[16%]', op: 'w-[15%]',
}
const WIDTH_CUSTOM: Record<string, string> = { pick: 'w-[8%]', model: 'w-[74%]', state: 'w-[18%]' }
const HEAD_LABEL: Record<string, string> = {
  pick: '', provider: t('Provider'), source: t('Model source'), state: t('Status'), count: t('Models'),
  note: t('Note'), updated: t('Updated'), model: t('Upstream model'), op: t('Actions'),
}
const KIND: Record<string, { variant: NonNullable<BadgeProps['variant']>; label: string }> = {
  refreshed: { variant: 'brand', label: t('Refreshed') },
  failed: { variant: 'destructive', label: t('Failed') },
  skipped: { variant: 'outline', label: t('Skipped') },
}

function norm(id: string) {
  return id.trim().toLowerCase()
}

function rateCell(text: string) {
  return <span className='font-mono text-[11.5px] tabular-nums text-muted-foreground'>{text}</span>
}

function nameCell(text: string) {
  return <div className='truncate font-mono text-[12px] font-semibold'>{t(text)}</div>
}

export function ModelsFetchTable({
  custom, status, rows, upstream, managed, picked, keyword, busy, busyRow,
  onPick, onFetchRow, sourceCell, refreshedAt, formatTime,
}: Props) {
  React.useEffect(() => {
    const bridge = window as unknown as { wbTableColumns?: { repaint?: (id: string) => void } }
    bridge.wbTableColumns?.repaint?.(custom ? TABLE_COL_ID.custom : TABLE_COL_ID.intl)
  }, [custom])

  const cols = custom ? COLS_CUSTOM : COLS_INTL
  const needle = keyword.trim().toLowerCase()
  const shown = upstream.filter(id => !needle || id.toLowerCase().includes(needle))

  function stateRow(text: string) {
    return (
      <TableRow>
        <TableCell colSpan={cols.length} className='py-[26px] text-center text-muted-foreground'>
          {text}
        </TableCell>
      </TableRow>
    )
  }

  function rowPick(event: React.MouseEvent<HTMLTableRowElement>, id: string, added: boolean) {
    if (added) return
    const target = event.target as Element | null
    // 复选框已经处理过点击；拖选模型 id 时也不能改变勾选状态。
    if (target?.closest('[data-slot="checkbox"], input[type="checkbox"]')) return
    const selection = window.getSelection()
    if (selection && !selection.isCollapsed && event.currentTarget.contains(selection.anchorNode)) return
    onPick(id, !picked.has(id))
  }

  function customBody() {
    if (status === 'idle') return stateRow(t('Click “Fetch models” on the right to pull this provider\'s model catalog from upstream'))
    if (status === 'loading') return stateRow(t('Fetching…'))
    if (!shown.length) return stateRow(upstream.length
      ? t('No models match “{keyword}”', { keyword: needle })
      : t('Upstream returned no models'))
    return shown.map(id => {
      const added = managed.has(norm(id))
      return (
        <TableRow key={id} className={added ? 'opacity-50' : 'cursor-pointer'}
          onClick={event => rowPick(event, id, added)}>
          <TableCell>
            <Checkbox checked={picked.has(id)} disabled={added} aria-label={id}
              onCheckedChange={next => onPick(id, Boolean(next))} />
          </TableCell>
          <TableCell>{nameCell(id)}</TableCell>
          <TableCell>
            {added ? (
              <Badge variant='brand' title={t('This model is already in this provider\'s catalog; it won\'t be imported again')}>{t('Added')}</Badge>
            ) : (
              <Badge variant='outline' title={t('Check it to add it to this provider\'s catalog on import')}>{t('Not added')}</Badge>
            )}
          </TableCell>
        </TableRow>
      )
    })
  }

  function rowFetchButton(row: Row) {
    const fetching = busy && busyRow === row.provider
    return (
      <Button variant='outline' size='sm' className='w-full' disabled={busy}
        title={t('Refresh only this provider\'s model catalog')} onClick={() => onFetchRow(row)}>
        {fetching ? t('Fetching…') : row.item ? t('Fetch again') : t('Fetch')}
      </Button>
    )
  }

  function intlBody() {
    if (status === 'loading') return stateRow(t('Fetching…'))
    if (!rows.length) return stateRow(t('No providers to refresh yet: add an account on the Accounts page first'))
    return rows.map(row => {
      const item = row.item
      if (!item) {
        const at = formatTime(refreshedAt(row.provider))
        return (
          <TableRow key={row.provider}>
            <TableCell>{nameCell(row.label)}</TableCell>
            <TableCell>{sourceCell(row.provider)}</TableCell>
            <TableCell><Badge variant='outline'>{t('Not fetched')}</Badge></TableCell>
            <TableCell>{rateCell('—')}</TableCell>
            <TableCell>{rateCell('—')}</TableCell>
            <TableCell>{rateCell(at || '—')}</TableCell>
            <TableCell>{rowFetchButton(row)}</TableCell>
          </TableRow>
        )
      }
      const kind = KIND[String(item.status)]
        ?? { variant: 'outline' as const, label: String(item.status || t('Unknown')) }
      const count = Number(item.count)
      const note = item.status === 'refreshed'
        ? t('Remote catalog updated; the “Source” column will show “Remote”')
        : item.message || (item.fixed ? t('This provider uses a fixed model catalog; nothing to refresh') : t('No new content this time'))
      const at = formatTime(Number(item.refreshedAt) || 0)
      return (
        <TableRow key={row.provider}>
          <TableCell>{nameCell(row.label)}</TableCell>
          <TableCell>{sourceCell(row.provider, item.accountId)}</TableCell>
          <TableCell><Badge variant={kind.variant}>{kind.label}</Badge></TableCell>
          <TableCell>{rateCell(count ? t('{n}', { n: count }) : '—')}</TableCell>
          <TableCell>
            <span className='text-[11.5px] leading-[1.5] text-muted-foreground [overflow-wrap:anywhere]'>{t(note)}</span>
          </TableCell>
          <TableCell>{rateCell(at || '—')}</TableCell>
          <TableCell>{rowFetchButton(row)}</TableCell>
        </TableRow>
      )
    })
  }

  return (
    <Table id={custom ? TABLE_ID.custom : TABLE_ID.intl} className='table-fixed'
      containerClassName='max-h-[min(48vh,460px)]'>
      <colgroup>
        {cols.map(key => (
          <col key={key} data-col={key} className={custom ? WIDTH_CUSTOM[key] : WIDTH_INTL[key]} />
        ))}
      </colgroup>
      <TableHeader>
        <TableRow>
          {cols.map(key => (
            <TableHead key={key} data-col={key}>{HEAD_LABEL[key]}</TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody className='[&_tr:last-child>td]:border-b-0'>{custom ? customBody() : intlBody()}</TableBody>
    </Table>
  )
}
