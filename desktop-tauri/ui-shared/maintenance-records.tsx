import * as React from 'react'
import {
  Button, Card, Empty, Input, Pager, SegmentedControl, Select, SelectContent, SelectItem,
  SelectTrigger, SelectValue, Spinner, Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@ui'
import { t } from '../ui-kit/src/i18n'
import { MaintenanceDetailDialog } from './maintenance-details'
import { RunCounts, RunStatus, RowChevron } from './maintenance-shared'
import { duration, maintenanceLabel, sourceLabel, stamp } from './maintenance-format'
import type { MaintenanceDetail, MaintenanceQuery, MaintenanceRun, MaintenanceTaskId } from './maintenance-types'

/**
 * 执行记录。
 *
 * ── 版面 ─────────────────────────────────────────────────────
 * 这一页真正的内容就是它，所以它吃掉概览条以下的全部高度：卡内分三段 ——
 * 工具条（标题与筛选同一行）、记录表（自己滚动、表头常驻）、底栏（条数、保留说明、翻页）。
 * 页面本身不滚，行数只受窗口高度限制，不再被上方的卡挤掉。
 *
 * ── 明细在哪 ─────────────────────────────────────────────────
 * 逐项明细走弹窗（见 maintenance-details），列表里每一行等高，点「查看明细」打开。
 *
 * ── 窄屏 ─────────────────────────────────────────────────────
 * 700px 以下表格换成一行一张卡片（列太多，横向滚在手机上不可用），
 * 与原来那套的判定阈值一致；弹窗在手机上按视口收窄。
 */

export function MaintenanceRecords({ compact, loading, busy, runs, total, page, pages, query, search,
  onSearch, onQuery, onPage, openId, onToggle, detail, detailLoading, detailError, onReloadDetail,
  retentionDays, maxRecords, onRetry, onAccount }: {
  compact: boolean
  loading: boolean
  busy: boolean
  runs: MaintenanceRun[]
  total: number
  page: number
  pages: number
  query: MaintenanceQuery
  search: string
  onSearch(value: string): void
  onQuery(patch: Partial<MaintenanceQuery>): void
  onPage(delta: number): void
  openId: string
  onToggle(id: string): void
  detail?: MaintenanceDetail
  detailLoading: boolean
  detailError: string
  onReloadDetail(): void
  retentionDays: number
  maxRecords: number
  onRetry(runId: string, itemId?: string): void
  onAccount?(accountId: string): void
}) {
  const resultOptions = [{ value: '', label: t('All outcomes') }, { value: 'failed', label: t('Has failures') }, { value: 'success', label: t('No failures') }]
  const sourceOptions = [{ value: '', label: t('All triggers') }, { value: 'automatic', label: t('Automatic') }, { value: 'manual', label: t('Manual') }]
  const filtered = Boolean(query.taskId || query.source || query.status || query.keyword)

  return <Card className='maintenance-records'>
    <div className='maintenance-toolbar'>
      <span className='maintenance-toolbar-title'>{t('Execution records')}</span>
      <span className='maintenance-toolbar-sub'>{t('Automatic and manual maintenance in one place')}</span>
      <SegmentedControl
        value={query.taskId}
        aria-label={t('Maintenance type')}
        onValueChange={value => onQuery({ taskId: value as MaintenanceTaskId | '' })}
        options={[
          { value: '', label: t('All') },
          { value: 'credentialMaintenance', label: t('Credential maintenance') },
          { value: 'modelRefresh', label: t('Model catalog update') },
        ]}
      />
      <Select value={query.status} onValueChange={value => onQuery({ status: (value ?? '') as MaintenanceQuery['status'] })}>
        <SelectTrigger aria-label={t('Operation result')} className='min-w-[124px]'>
          <SelectValue>{resultOptions.find(option => option.value === query.status)?.label}</SelectValue>
        </SelectTrigger>
        <SelectContent>{resultOptions.map(option => <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>)}</SelectContent>
      </Select>
      <Select value={query.source} onValueChange={value => onQuery({ source: (value ?? '') as MaintenanceQuery['source'] })}>
        <SelectTrigger aria-label={t('Trigger')} className='min-w-[132px]'>
          <SelectValue>{sourceOptions.find(option => option.value === query.source)?.label}</SelectValue>
        </SelectTrigger>
        <SelectContent>{sourceOptions.map(option => <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>)}</SelectContent>
      </Select>
      <Input
        value={search}
        aria-label={t('Search accounts or providers')}
        placeholder={t('Search accounts or providers')}
        className='maintenance-push maintenance-search'
        onChange={event => onSearch(event.target.value)}
      />
    </div>

    {compact
      ? <div className='maintenance-history-cards'>
        {runs.map(run => <div className='maintenance-run-card' key={run.id}>
          <div className='maintenance-run-card-heading'>
            <strong>{maintenanceLabel(run.taskId)}</strong>
            <RunStatus status={run.status} />
          </div>
          <p className='maintenance-muted'>{stamp(run.startedAt)} · {sourceLabel(run.source)} · {duration(run.durationMs)}</p>
          <RunCounts run={run} />
          <Button size='sm' variant='link' className='hover:no-underline' onClick={() => onToggle(run.id)}>{t('View details')}</Button>
        </div>)}
        {!runs.length ? <Empty>{filtered ? t('No maintenance records match these filters') : t('No maintenance records yet. The first run will be saved automatically.')}</Empty> : null}
      </div>
      : <Table containerClassName='maintenance-scroll min-h-0 flex-1' className='maintenance-history-table'>
        <TableHeader><TableRow>
          <TableHead>{t('Run time')}</TableHead>
          <TableHead>{t('Maintenance type')}</TableHead>
          <TableHead>{t('Trigger')}</TableHead>
          <TableHead>{t('Result')}</TableHead>
          <TableHead>{t('Run summary')}</TableHead>
          <TableHead>{t('Duration')}</TableHead>
          <TableHead className='text-right'>{t('Actions')}</TableHead>
        </TableRow></TableHeader>
        <TableBody>
          {runs.map(run => <TableRow key={run.id}>
            <TableCell><span className='maintenance-num'>{stamp(run.startedAt)}</span></TableCell>
            <TableCell>
              {maintenanceLabel(run.taskId)}
              {run.retryOf ? <span className='maintenance-retry-label'>{t('Retry of failed items')}</span> : null}
            </TableCell>
            <TableCell>{sourceLabel(run.source)}</TableCell>
            <TableCell><RunStatus status={run.status} /></TableCell>
            <TableCell><RunCounts run={run} /></TableCell>
            <TableCell><span className='maintenance-num'>{duration(run.durationMs)}</span></TableCell>
            <TableCell className='text-right'>
              <Button size='sm' variant='link' className='hover:no-underline' onClick={() => onToggle(run.id)}>
                {t('View details')}
                <RowChevron />
              </Button>
            </TableCell>
          </TableRow>)}
          {!runs.length ? <TableRow><TableCell colSpan={7}>
            <Empty>{filtered ? t('No maintenance records match these filters') : t('No maintenance records yet. The first run will be saved automatically.')}</Empty>
          </TableCell></TableRow> : null}
        </TableBody>
      </Table>}

    <div className='maintenance-foot'>
      <span className='maintenance-num'>{t('{n} runs in total', { n: total })}</span>
      <span className='maintenance-foot-note'>{t('Maintenance records are kept for {days} days, up to {count} runs. Only results are saved, never keys or tokens.', { days: retentionDays, count: maxRecords })}</span>
      {loading ? <Spinner className='maintenance-foot-spinner' /> : null}
      <Pager className='maintenance-push' page={page} pageCount={pages} disabled={loading} onPageChange={next => onPage(next - page)} />
    </div>

    {openId ? <MaintenanceDetailDialog
      key={openId}
      detail={detail}
      loading={detailLoading}
      error={detailError}
      busy={busy}
      onReload={onReloadDetail}
      onClose={() => onToggle(openId)}
      onRetry={onRetry}
      onAccount={onAccount}
    /> : null}
  </Card>
}
