import * as React from 'react'

/**
 * by can new hook。
 *
 * must， 1 will 「 after 」：
 * ① not can （ to App / ） when ；
 * ② up 1 request no when not down 1 ；
 * ③ ， not when 。
 *
 * `interval` 0 not （）。
 */
export function usePolling(
  run: () => Promise<void> | void,
  interval: number,
  deps: React.DependencyList = [],
): { refreshing: boolean; refresh: () => void } {
  const runRef = React.useRef(run)
  runRef.current = run
  const [refreshing, setRefreshing] = React.useState(false)
  const busyRef = React.useRef(false)
  const timerRef = React.useRef<number | null>(null)

  const tick = React.useCallback(async () => {
    if (busyRef.current) return
    busyRef.current = true
    setRefreshing(true)
    try {
      await runRef.current()
    } finally {
      busyRef.current = false
      setRefreshing(false)
    }
  }, [])

  React.useEffect(() => {
    let disposed = false
    const clear = () => {
      if (timerRef.current !== null) {
        window.clearInterval(timerRef.current)
        timerRef.current = null
      }
    }
    const start = () => {
      clear()
      if (interval <= 0 || disposed) return
      timerRef.current = window.setInterval(() => {
        if (document.visibilityState === 'visible') void tick()
      }, interval)
    }

    void tick()
    start()

    const onVisibility = () => {
      if (document.visibilityState === 'visible') {
        // to before 1 ， use not will old 
        void tick()
        start()
      } else {
        clear()
      }
    }
    document.addEventListener('visibilitychange', onVisibility)

    return () => {
      disposed = true
      clear()
      document.removeEventListener('visibilitychange', onVisibility)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [interval, tick, ...deps])

  return { refreshing, refresh: () => void tick() }
}
