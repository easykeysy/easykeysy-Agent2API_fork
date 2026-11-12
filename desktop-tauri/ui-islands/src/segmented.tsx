import { createRoot } from 'react-dom/client'

import { SegmentedControl, type SegmentedControlOption } from '@ui'

/**
 * —— 1 。
 *
 * use （）：
 *   const island = window.wbSegmented.mount(host, {
 * options: [{ value: 'all', label: '', count: 12 }, ...],
 *     value: current,
 * ariaLabel: 'enabled',
 *     onChange: next => { ... },
 *   })
 * island.setValue(next) // after 
 * island.setOptions(options) // / use ， or 
 *
 * （ and ui/select.js 「」 1 to ）： and 
 * —— 、、；、、， is ，
 * not ， is 「 after need 」 not will 。
 */

/** by （ use is JS， no has can ）； */
type SegmentedIslandOptions = {
  options: readonly SegmentedControlOption<string>[]
  value: string
  /** none， for up aria-label */
  ariaLabel: string
  onChange: (value: string) => void
  /** to `.seg` up （ add-seg ） */
  className?: string
}

/** ： by use to ， not */
type IslandState = {
  options: readonly SegmentedControlOption<string>[]
  value: string
  ariaLabel: string
  className?: string
}

/**
 * to up ，return 。
 *
 * use 「 + ref setter」： has 1 （ 
 * that ），， not in not 。update 1 ，
 * React by use ， etc props —— and 。
 */
function mountSegmented(host: HTMLElement, initial: SegmentedIslandOptions) {
  const root = createRoot(host)
  let current: IslandState = {
    options: initial.options,
    value: initial.value,
    ariaLabel: initial.ariaLabel,
    className: initial.className,
  }

  const render = () => {
    root.render(
      <SegmentedControl
        options={current.options}
        value={current.value}
        onValueChange={initial.onChange}
        aria-label={current.ariaLabel}
        className={current.className}
      />
    )
  }
  render()

  return {
    /** after ， in old up */
    setValue(value: string) {
      if (value === current.value) return
      current = { ...current, value }
      render()
    },
    /** （、 use 、） when */
    setOptions(options: readonly SegmentedControlOption<string>[]) {
      current = { ...current, options }
      render()
    },
    unmount() {
      root.unmount()
    },
  }
}

type SegmentedIsland = ReturnType<typeof mountSegmented>

export { mountSegmented, type SegmentedIsland, type SegmentedIslandOptions }
