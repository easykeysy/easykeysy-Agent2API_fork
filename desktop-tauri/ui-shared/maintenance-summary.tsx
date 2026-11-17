import { Button, Card, Spinner, Table, TableBody, TableCell, TableRow } from '@ui'
import { t } from '../ui-kit/src/i18n'
import { RunCounts, RunStatus } from './maintenance-shared'
import { duration, maintenanceLabel, sourceLabel, stamp } from './maintenance-format'
import type { MaintenanceRun, MaintenanceTask, MaintenanceTaskId } from './maintenance-types'

/**
 * 概览条：两类维护（凭证维护 / 模型目录更新）各一行。
 *
 * ── 为什么不是两张卡 ──────────────────────────────────────────
 * 这一页真正的内容是下面的执行记录。原来每个任务一张卡（标题行 + 元信息 + 大字读数 +
 * 说明 + 底栏），两张并排就要吃掉首屏一半高度，记录只剩三四行可见。改成两行状态带后
 * 固定高度从约 240px 降到约 90px，剩下的高度全部留给记录。
 *
 * ── 信息一点没少 ─────────────────────────────────────────────
 * 任务说明移到任务名的悬停提示（title）；冷却时间、自动维护是否关闭都在「调度」列写明；
 * 执行中的状态、上次结果、下次时间、两个动作（查看结果 / 立即检查）都保留。
 *
 * 窄屏（手机）用下面那份堆叠版：一行放不下六列，硬挤会把名字压成竖条。
 */

const describe = (id: MaintenanceTaskId): string => id === 'credentialMaintenance'
  ? t('Only expired or soon-to-expire credentials are maintained')
  : t('Updates each provider\'s catalog; model metadata results are listed separately')

const actionLabel = (id: MaintenanceTaskId): string => id === 'credentialMaintenance' ? t('Check now') : t('Update now')

type TaskRow = {
  task: MaintenanceTask
  run: MaintenanceRun | undefined
  running: boolean
  stalled: boolean
}

export function MaintenanceSummary({ tasks, latest, busy, compact, onRun, onShowLatest }: {
  tasks: MaintenanceTask[]
  latest: Partial<Record<MaintenanceTaskId, MaintenanceRun>>
  busy: MaintenanceTaskId | ''
  compact: boolean
  onRun(id: MaintenanceTaskId): void
  onShowLatest(id: MaintenanceTaskId): void
}) {
  const rows: TaskRow[] = tasks.map(task => {
    const run = latest[task.id]
    const running = task.running || busy === task.id
    return { task, run, running, stalled: running || run?.status === 'running' || run?.status === 'interrupted' }
  })
  const schedule = ({ task }: TaskRow) => task.enabled
    ? <>{t('Every {n} minutes', { n: task.interval })}<span className='maintenance-sep'>·</span>{t('Next run: {time}', { time: stamp(task.nextRunAt) })}</>
    : t('Automatic maintenance is off')
  const cooldown = ({ task }: TaskRow) => task.enabled && task.retryAt && task.retryAt > Date.now()
    ? <span className='maintenance-cooldown'>{t('Cooldown until {time}', { time: stamp(task.retryAt) })}</span>
    : null
  const lastRun = ({ task, run }: TaskRow) => run
    ? <>{t('Last run')}<span className='maintenance-sep'>·</span>{stamp(run.startedAt)}<span className='maintenance-sep'>·</span>{sourceLabel(run.source)}<span className='maintenance-sep'>·</span>{duration(run.durationMs)}</>
    : <span title={task.lastResult ?? undefined}>{task.lastResult ? t('Only a summary was recorded previously; item details are not available') : t('Not run yet')}</span>

  if (compact) {
    return <Card className='maintenance-summary'>
      {rows.map((row, index) => <div key={row.task.id} className={index ? 'maintenance-summary-block maintenance-divided' : 'maintenance-summary-block'}>
        <div className='maintenance-summary-block-head'>
          <span className='maintenance-summary-name' title={describe(row.task.id)}>{maintenanceLabel(row.task.id)}</span>
          <RunStatus status={row.running ? 'running' : row.run?.status} pulse={row.running} />
        </div>
        <p className='maintenance-summary-line'>{lastRun(row)}</p>
        {row.run ? <RunCounts run={row.run} /> : null}
        <p className='maintenance-summary-line'>{schedule(row)}{cooldown(row)}</p>
        <div className='maintenance-summary-block-actions'>
          <Button size='sm' variant='ghost' disabled={!row.run} onClick={() => onShowLatest(row.task.id)}>{t('View result')}</Button>
          <Button size='sm' disabled={Boolean(busy) || row.task.running} onClick={() => onRun(row.task.id)}>
            {row.running ? <Spinner /> : null}{row.running ? t('Running…') : actionLabel(row.task.id)}
          </Button>
        </div>
      </div>)}
    </Card>
  }

  return <Card className='maintenance-summary'>
    <Table className='maintenance-summary-table'>
      <TableBody>
        {rows.map(row => <TableRow key={row.task.id}>          <TableCell className='maintenance-summary-name' title={describe(row.task.id)}>{maintenanceLabel(row.task.id)}</TableCell>
          <TableCell><RunStatus status={row.running ? 'running' : row.run?.status} pulse={row.running} /></TableCell>
          <TableCell className='maintenance-summary-line'>{lastRun(row)}</TableCell>
          <TableCell>{row.run ? <RunCounts run={row.run} /> : null}</TableCell>
          <TableCell className='maintenance-summary-line'>{schedule(row)}{cooldown(row)}</TableCell>
          <TableCell className='maintenance-summary-actions'>
            <Button size='sm' variant='ghost' disabled={!row.run} onClick={() => onShowLatest(row.task.id)}>{t('View result')}</Button>
            <Button size='sm' disabled={Boolean(busy) || row.task.running} onClick={() => onRun(row.task.id)}>
              {row.running ? <Spinner /> : null}{row.running ? t('Running…') : actionLabel(row.task.id)}
            </Button>
          </TableCell>
        </TableRow>)}
      </TableBody>
    </Table>
  </Card>
}
