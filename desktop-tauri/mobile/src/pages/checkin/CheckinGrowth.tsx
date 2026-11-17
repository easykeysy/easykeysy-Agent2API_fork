import { useState } from 'react'
import { Button } from '@ui'
import { Badge } from '../../components/ui'
import { ProviderLogo } from '../../components/ProviderLogo'
import {
  getGrowth, runGrowthSchool, runGrowthTask, runGrowthTravel,
  type GrowthEntry, type GrowthResult, type GrowthRun, type GrowthTask,
} from '../../api/checkin'
import type { CheckinAction } from '../CheckinPage'
import { store } from '../../state/store'
import { t } from '../../i18n'
import { growthTaskRunnable } from '../../../../ui-shared/checkin'

function canRun(task: GrowthTask, mode: 'full' | 'claim'): boolean {
  return growthTaskRunnable(task, mode)
}

export function CheckinGrowth({ row, busy, action }: { row: GrowthEntry; busy: string; action: CheckinAction }) {
  const [data, setData] = useState<GrowthResult | null>(null)
  const [results, setResults] = useState<GrowthRun[]>([])
  const tasks = data?.tasks?.tasks || []

  const query = async () => {
    const result = await action(`growth-query:${row.id}`, () => getGrowth(row.id))
    if (result) setData(result)
  }

  const runTasks = async (mode: 'full' | 'claim') => {
    const targets = tasks.filter(task => canRun(task, mode))
    if (!targets.length) return
    setResults([])
    const result = await action(`growth-run:${row.id}`, async requireCurrent => {
      const rows: GrowthRun[] = []
      for (const task of targets) {
        requireCurrent()
        try {
          const result = await runGrowthTask(row.id, task.code, mode)
          requireCurrent()
          rows.push(result)
        } catch (cause) {
          requireCurrent()
          rows.push({ code: task.code, outcome: 'failed', message: cause instanceof Error ? cause.message : String(cause) })
        }
        setResults([...rows])
      }
      const next = await getGrowth(row.id)
      requireCurrent()
      setData(next)
      return rows
    })
    if (result) {
      const failed = result.filter(item => item.outcome === 'failed' || item.error).length
      store.toast(t('Execution complete: {n} items, {failed} failed', { n: result.length, failed }), failed ? 'err' : 'ok')
    }
  }

  const extra = async (kind: 'travel' | 'school' | 'lottery') => {
    const result = await action(`growth-${kind}:${row.id}`, async requireCurrent => {
      const result = kind === 'travel' ? await runGrowthTravel(row.id) : await runGrowthSchool(row.id, kind === 'lottery')
      requireCurrent()
      const next = await getGrowth(row.id)
      requireCurrent()
      setData(next)
      return result
    })
    if (result) store.toast(result.message || result.error || t('Execution complete'), result.outcome === 'failed' || result.error ? 'err' : 'ok')
  }

  return (
    <div className='m-card-pad m-ck-account'>
      <div className='m-ck-toolbar'>
        <ProviderLogo id={row.provider} />
        <div className='m-ck-grow'>
          <div className='m-ck-title'>{row.name || row.id}</div>
          {data ? <div className='m-ck-note'>{t('energy {energy}· {streak} days in a row', { energy: data.energy ?? '—', streak: data.streak ?? '—' })}</div> : null}
        </div>
        <Button size='sm' variant='outline' disabled={Boolean(busy)} onClick={() => void query()}>
          {busy === `growth-query:${row.id}` ? t('Checking…') : t('Query tasks')}
        </Button>
      </div>
      {data?.tasks?.error ? <p className='m-ck-error'>{data.tasks.error}</p> : null}
      {data ? <div className='m-ck-actions'>
        <Button size='sm' disabled={Boolean(busy) || !tasks.some(task => canRun(task, 'full'))} onClick={() => void runTasks('full')}>
          {busy === `growth-run:${row.id}` ? t('Running…') : t('Complete all')}
        </Button>
        <Button size='sm' variant='outline' disabled={Boolean(busy) || !tasks.some(task => canRun(task, 'claim'))}
          onClick={() => void runTasks('claim')}>{t('Claim reward')}</Button>
      </div> : null}
      {tasks.map(task => <div className='m-ck-task' key={task.code}>
        <div className='m-ck-grow'>
          <div>{task.title || task.code}</div>
          {task.target && task.target > 1 ? <div className='m-ck-note'>{task.current || 0} / {task.target}</div> : null}
          {results.find(result => result.code === task.code)?.message ? <div className='m-ck-note'>{results.find(result => result.code === task.code)?.message}</div> : null}
        </div>
        <Badge tone={task.status === 'claimed' ? 'ok' : !task.supported || task.unforgeable || task.locked ? 'mute' : 'warn'}>
          {task.status === 'claimed' ? t('Claimed') : canRun(task, 'claim') ? t('Claimable')
            : task.unforgeable ? t('Requires real action') : !task.supported ? t('Not supported') : task.locked ? t('Locked') : t('To complete')}
        </Badge>
      </div>)}
      {data ? <div className='m-ck-actions'>
        <Button size='sm' variant='outline' disabled={Boolean(busy)} onClick={() => void extra('travel')}>{t('One cat travel trip')}</Button>
        <Button size='sm' variant='outline' disabled={Boolean(busy)} onClick={() => void extra('school')}>{t('The back-to-school task')}</Button>
        <Button size='sm' variant='outline' disabled={Boolean(busy)} onClick={() => void extra('lottery')}>{t('Back-to-school + lottery')}</Button>
      </div> : <p className='m-ck-note'>{t('After querying, view the task list; only execute tasks the plugin marks as supported')}</p>}
    </div>
  )
}
