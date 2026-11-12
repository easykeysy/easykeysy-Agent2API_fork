import * as React from 'react'
import { t } from '../i18n'
import { bridge, errorMessage, host, type RoutingWorkspace, type StickyEntry } from './routing-types'

export type ReplayQuery = { model?: string; minutes: number; scope: 'window' | 'running'; search?: string }

/**
 * request ：（、、） and will 。
 *
 * request by `getRoutingFlow(detail=requests)`； this not to ，
 * after not will has 2.5 seconds 1 300 path。
 */
export function useRoutingReplay(active: boolean, workspaceModel?: string) {
  const [model, setModel] = React.useState('')
  const [minutes, setMinutes] = React.useState(10)
  const [scope, setScope] = React.useState<'window' | 'running'>('window')
  const [search, setSearch] = React.useState('')
  const [workspace, setWorkspace] = React.useState<RoutingWorkspace | null>(null)
  const [sticky, setSticky] = React.useState<StickyEntry[]>([])
  const [error, setError] = React.useState('')
  const [loading, setLoading] = React.useState(false)
  const [now, setNow] = React.useState(() => Date.now())
  const [token, refresh] = React.useReducer(value => value + 1, 0)
  const sequence = React.useRef(0)
  const suspended = React.useRef(false)
  const query: ReplayQuery = { model: model || undefined, minutes, scope, search: search || undefined }
  const currentModel = workspaceModel ?? model

  React.useEffect(() => {
    if (!active) return
    let alive = true
    let pending = false
    const load = async (silent: boolean) => {
      if (pending || suspended.current) return
      pending = true
      const id = ++sequence.current
      if (!silent) setLoading(true)
      try {
        const [nextWorkspace, nextSticky] = await Promise.all([
          bridge().getRoutingWorkspace(currentModel || undefined),
          bridge().getRoutingSticky(currentModel || undefined),
        ])
        if (!alive || id !== sequence.current) return
        if (!nextWorkspace?.profile || !Array.isArray(nextWorkspace.accounts)) throw new Error(t('路由接口返回了不完整的数据'))
        setWorkspace(nextWorkspace)
        setSticky(nextSticky.entries || [])
        setError('')
      } catch (failure) {
        if (alive && id === sequence.current) setError(errorMessage(failure))
      } finally {
        pending = false
        if (alive && id === sequence.current) setLoading(false)
      }
    }
    void load(false)
    const timer = window.setInterval(() => {
      if (!document.hidden && host().wbApp?.currentPage === 'routing') void load(true)
    }, 5000)
    return () => { alive = false; sequence.current++; window.clearInterval(timer) }
  }, [active, currentModel, token])

  React.useEffect(() => {
    if (!active) return
    const timer = window.setInterval(() => setNow(Date.now()), 1000)
    return () => window.clearInterval(timer)
  }, [active])

  const selectModel = (value: string) => { setModel(value); sequence.current++; setWorkspace(null); setError('') }
  const invalidateWorkspace = React.useCallback(() => { sequence.current++; setWorkspace(null) }, [])
  const suspendWorkspace = React.useCallback(() => { suspended.current = true; sequence.current++; setWorkspace(null) }, [])
  const resumeWorkspace = React.useCallback(() => { suspended.current = false; sequence.current++; setWorkspace(null); refresh() }, [])
  return { model, selectModel, minutes, setMinutes, scope, setScope, search, setSearch,
    workspace, sticky, error, loading, now, query, refresh, invalidateWorkspace, suspendWorkspace, resumeWorkspace }
}
