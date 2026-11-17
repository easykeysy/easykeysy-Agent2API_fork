import * as React from 'react'
import { Button, Dialog, DialogBody, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, SegmentedControl } from '@ui'
import { t } from '../i18n'
import {
  formatLogDuration, formatLogTime, groupTime, logAdvice, logCategory, logContext, logDetailText, logLevel, logNumber,
  logPlugins, logRawText, logSource, logSteps, logSummary, logText, shared, toast, type LogEntry, type LogGroup,
} from './logs-model'
import { DIALOG_SEG_CLASS } from './dialog-segmented'
import { Empty } from '@ui'

export function LogsList({ groups, empty, loading, listRef, onScroll, onDetail }: {
  groups: LogGroup[]
  empty: string
  loading: boolean
  listRef: React.RefObject<HTMLDivElement | null>
  onScroll: () => void
  onDetail: (group: LogGroup) => void
}) {
  return (
    <div className='log-list event-log-list' id='log-list' ref={listRef} onScroll={onScroll} aria-busy={loading}>
      <div className='event-log-head' aria-hidden='true'>
        <span>{t('Time')}</span><span>{t('Level')}</span><span>{t('Source')}</span>
        <span>{t('Event summary')}</span><span>{t('Count')}</span><span />
      </div>
      {groups.length ? groups.map(group => {
        const entry = group.entry
        const context = logContext(entry, t)
        const summary = logSummary(entry, t)
        const source = logSource(entry, t)
        return (
          <div className={`event-log-row ${entry.level || 'info'}`} key={group.id}>
            <time className='event-log-time' dateTime={entry.ts ? new Date(entry.ts).toISOString() : undefined} title={groupTime(group)}>
              {formatLogTime(group.lastTs)}
              {group.count > 1 ? <span>{formatLogTime(group.firstTs, true)}–{formatLogTime(group.lastTs, true)}</span> : null}
            </time>
            <span className={`event-log-level ${entry.level || 'info'}`}>{logLevel(entry.level, t)}</span>
            <span className='event-log-source' title={source}>{source}</span>
            <div className='event-log-message'>
              <span className='event-log-summary' title={summary}>{summary}</span>
              {context ? <span className='event-log-context' title={context}>{context}</span> : null}
            </div>
            <span className={group.count > 1 ? 'event-log-count repeated' : 'event-log-count'} title={t('{n} raw records', { n: group.count })}>
              {group.count > 1 ? `×${group.count}` : '—'}
            </span>
            <Button variant='ghost' size='xs' onClick={() => onDetail(group)}
              aria-label={t('View log details: {message}', { message: summary })}>{t('Details')}</Button>
          </div>
        )
      }) : <Empty className='log-empty'>{empty}</Empty>}
    </div>
  )
}

function fields(entry: LogEntry): [string, string][] {
  return [
    [t('Level'), logLevel(entry.level, t)], [t('Category'), logCategory(entry.category, t)],
    [t('Source'), logSource(entry, t)], [t('Provider'), logText(entry, 'provider')],
    [t('Plugin name'), logText(entry, 'pluginName')], [t('Plugin ID'), logText(entry, 'pluginId')],
    [t('Accounts'), logText(entry, 'accountName') || logText(entry, 'account')],
    [t('Model'), logText(entry, 'model')],
    [t('Status code'), logText(entry, 'status')],
    [t('Duration'), formatLogDuration(logNumber(entry, 'durationMs'), t)],
  ].filter((pair): pair is [string, string] => Boolean(pair[1]))
}

export function LogDetail({ group, onClose }: { group: LogGroup | null; onClose: () => void }) {
  const [tab, setTab] = React.useState<'detail' | 'raw'>('detail')
  React.useEffect(() => setTab('detail'), [group?.id])
  if (!group) return null
  const entry = group.entry
  const advice = logAdvice(entry, t)
  const detail = logDetailText(entry)
  const steps = logSteps(entry, t)
  const plugins = logPlugins(entry, t)
  const copy = async () => {
    const text = logRawText(group.entries)
    try {
      const fallback = shared().copyToClipboard
      if (fallback) {
        if (!(await fallback(text))) throw new Error()
      } else await navigator.clipboard.writeText(text)
      toast(t('Copied {n} log entries', { n: group.count }), 'ok')
    } catch { toast(t('Copy failed'), 'err') }
  }
  return (
    <Dialog open onOpenChange={open => { if (!open) onClose() }}>
      <DialogContent className='w-[min(860px,calc(100vw-48px))]'>
        <DialogHeader><DialogTitle>{t('Log details')}</DialogTitle></DialogHeader>
        <DialogBody>
          <DialogDescription>{logSummary(entry, t)}</DialogDescription>
          <div className='event-log-detail-meta'>{groupTime(group)} · {t('{n} raw records', { n: group.count })}</div>
          {/* 两档视图：走组件库的分段控件（弹窗档尺寸）—— 原来是两颗按钮切底色假装成分段 */}
          <SegmentedControl<'detail' | 'raw'>
            aria-label={t('Log details view')}
            className={DIALOG_SEG_CLASS}
            value={tab}
            onValueChange={setTab}
            options={[
              { value: 'detail', label: t('Event details') },
              { value: 'raw', label: t('Raw records') },
            ]}
          />
          {tab === 'detail' ? <>
            {advice ? <div className='event-log-advice'><h3>{t('Suggested action')}</h3><p>{advice}</p></div> : null}
            <dl className='event-log-detail-grid'>{fields(entry).map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl>
            {plugins.length ? <div className='event-log-detail-block'><h3>{t('Startup results')}</h3>
              <ul className='event-log-results'>{plugins.map(plugin => <li key={plugin.id}>
                <span>{plugin.name}</span><span className={plugin.phase === 'ready' ? '' : 'event-log-warnings'}>{plugin.status}</span>
              </li>)}</ul>
            </div> : null}
            {steps.length ? <div className='event-log-detail-block'><h3>{t('Time spent at each stage')}</h3>
              <dl className='event-log-detail-grid'>{steps.map((step, index) => <div key={index}><dt>{step.label}</dt><dd>{step.duration}</dd></div>)}</dl>
            </div> : null}
            {detail ? <div className='event-log-detail-block'><h3>{t('Details')}</h3><pre>{detail}</pre></div> : null}
            {group.count > 1 ? <div className='event-log-detail-block'><h3>{t('Occurrence times')}</h3>
              <div className='event-log-occurrences'>{group.entries.map((item, index) => <span key={item.id ?? index}>{formatLogTime(item.ts)}</span>)}</div>
            </div> : null}
          </> : <div className='event-log-detail-block'><pre>{group.entries.map(item => JSON.stringify(item, null, 2)).join('\n\n')}</pre></div>}
        </DialogBody>
        <DialogFooter><Button variant='outline' onClick={() => void copy()}>{t('Copy raw records')}</Button>
          <div className='spacer' /><Button variant='ghost' onClick={onClose}>{t('Close')}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
