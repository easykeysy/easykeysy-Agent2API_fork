import { useSyncExternalStore } from 'react'
import { getPwaSnapshot, subscribePwa, type PwaState } from './pwa'

/**
 * PWA 。
 *
 * `getPwaSnapshot` return is 1 （ no when use not ），
 * useSyncExternalStore for 「must use 」 need —— otherwise
 * will as ，none。
 */
export function usePwa(): PwaState {
  return useSyncExternalStore(subscribePwa, getPwaSnapshot, getPwaSnapshot)
}
