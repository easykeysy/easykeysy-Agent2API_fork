/**
 * down new 。
 *
 * hook + ， not ： use `bindRef` to 、
 * `<PullIndicator />` can 。
 *
 * before ：`PullIndicator` use `position: absolute` ，
 * must is up down 。mobile.css already to `.m-page` 
 * `position: relative`， in `.m-page` can 。
 */
import { useEffect, useRef, useState } from 'react'
import type { ReactElement, RefObject } from 'react'
import { t } from '../i18n'
import { Icon } from './Icon'

type PullToRefreshOptions = {
  /** down this （px） new ，default 64 */
  threshold?: number
  /** new ；return Promise when will etc */
  onRefresh: () => Promise<unknown> | unknown
  /** 。 not when '.m-scroll' or '.m-body' */
  scrollRef?: RefObject<HTMLElement | null>
}

type PullToRefreshResult = {
  /** to up （ when use ） */
  bindRef: RefObject<HTMLDivElement | null>
  /** current down （px）， to use */
  pull: number
  /** is in new */
  refreshing: boolean
}

/** ： 2px， 1px */
const DAMPING = 0.5
/** to 1.8 ， */
const MAX_RATIO = 1.8

/**
 * 。
 *
 * use has （ mobile.css）：
 * 1. ： in `.m-scroll`；
 * 2. + ：`.m-scroll` `overflow: hidden` ， is 
 * `.m-body`。
 *
 * 2 must： is `.m-page.m-fixed` when ，
 * `.m-scroll` in not ，return `.m-body`， not can to 
 * `.m-scroll` up 。
 */
function resolveScrollContainer(bind: HTMLElement | null): HTMLElement | null {
  if (!bind) return null
  const fixedPage = bind.classList.contains('m-fixed')
    ? bind
    : bind.closest<HTMLElement>('.m-page.m-fixed')
  if (fixedPage) {
    const body = fixedPage.querySelector<HTMLElement>('.m-body')
    if (body) return body
  }
  const scroller = bind.closest<HTMLElement>('.m-scroll')
  if (scroller) return scroller
  return bind.querySelector<HTMLElement>('.m-body')
}

export function usePullToRefresh(options: PullToRefreshOptions): PullToRefreshResult {
  const { threshold = 64, onRefresh, scrollRef } = options
  const bindRef = useRef<HTMLDivElement | null>(null)
  const [pull, setPull] = useState(0)
  const [refreshing, setRefreshing] = useState(false)

  // in when ref： state will to old 
  const startYRef = useRef(0)
  const activeRef = useRef(false)
  const pullRef = useRef(0)
  const refreshingRef = useRef(false)

  // onRefresh ， render is new use ； ref，
  // when / 
  const onRefreshRef = useRef(onRefresh)
  useEffect(() => {
    onRefreshRef.current = onRefresh
  }, [onRefresh])

  useEffect(() => {
    const el = scrollRef?.current ?? resolveScrollContainer(bindRef.current)
    // not to not ， not 
    if (!el) return
    let alive = true

    const updatePull = (value: number) => {
      pullRef.current = value
      if (alive) setPull(value)
    }

    const onTouchStart = (event: TouchEvent) => {
      // new in not new 
      if (refreshingRef.current) {
        activeRef.current = false
        return
      }
      const touch = event.touches[0]
      if (!touch) {
        activeRef.current = false
        return
      }
      // has in can can is down new ； in to 
      if (el.scrollTop > 0) {
        activeRef.current = false
        return
      }
      startYRef.current = touch.clientY
      activeRef.current = true
    }

    const onTouchMove = (event: TouchEvent) => {
      if (!activeRef.current || refreshingRef.current) return
      const touch = event.touches[0]
      if (!touch) return
      const deltaY = touch.clientY - startYRef.current
      // to up not down new ， already 
      if (deltaY <= 0) {
        updatePull(0)
        return
      }
      // when in 、 in down （e.g.），
      if (el.scrollTop > 0) {
        activeRef.current = false
        updatePull(0)
        return
      }
      // has is down overscroll；otherwise will 
      event.preventDefault()
      updatePull(Math.min(deltaY * DAMPING, threshold * MAX_RATIO))
    }

    const onTouchEnd = () => {
      if (!activeRef.current) return
      activeRef.current = false
      if (refreshingRef.current) return
      // no to ：， not new 
      if (pullRef.current < threshold) {
        updatePull(0)
        return
      }
      refreshingRef.current = true
      setRefreshing(true)
      // in ， not 
      updatePull(threshold)
      void (async () => {
        try {
          await onRefreshRef.current()
        } catch {
          // failure by use ， this 1 
        } finally {
          refreshingRef.current = false
          if (!alive) return
          setRefreshing(false)
          updatePull(0)
        }
      })()
    }

    // （、 in etc ）：， not new 
    const onTouchCancel = () => {
      activeRef.current = false
      if (!refreshingRef.current) updatePull(0)
    }

    el.addEventListener('touchstart', onTouchStart, { passive: false })
    el.addEventListener('touchmove', onTouchMove, { passive: false })
    el.addEventListener('touchend', onTouchEnd, { passive: false })
    el.addEventListener('touchcancel', onTouchCancel, { passive: false })

    return () => {
      alive = false
      el.removeEventListener('touchstart', onTouchStart)
      el.removeEventListener('touchmove', onTouchMove)
      el.removeEventListener('touchend', onTouchEnd)
      el.removeEventListener('touchcancel', onTouchCancel)
      // / when 
      pullRef.current = 0
      setPull(0)
    }
    // not bindRef.current ：ref in commit （ passive
    // effect），effect when to ； if ， setPull 
    // will from null ，effect and —— down will 
    // 1 。 in not ，none need 。
  }, [scrollRef, threshold])

  return { bindRef, pull, refreshing }
}

/** down / / new in ， for in */
export function PullIndicator(props: {
  pull: number
  refreshing: boolean
  threshold?: number
}): ReactElement | null {
  const { pull, refreshing, threshold = 64 } = props
  if (pull === 0 && !refreshing) return null

  const ready = pull >= threshold
  const text = refreshing ? t('正在刷新…') : ready ? t('松开刷新') : t('下拉刷新')
  // new when down ， to after to .m-ptr-spin 
  const tilt = refreshing
    ? undefined
    : { transform: `rotate(${Math.min(pull / threshold, 1) * 180}deg)` }

  return (
    <div className='m-ptr' style={{ height: pull }} role='status' aria-live='polite'>
      <div className='m-ptr-inner'>
        <Icon
          name='refresh'
          width={16}
          height={16}
          className={refreshing ? 'm-ptr-spin' : undefined}
          style={tilt}
        />
        <span>{text}</span>
      </div>
    </div>
  )
}
