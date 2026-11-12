/**
 * down ****（ / ）—— ， not 。
 *
 * ── Why is ， not is HTML5 ─────────────────────────
 * and 、set 1 ： in 2 seconds will 
 * 1 ， HTML5 will in ** that ** up ， 1 ； in 
 * JS ， React not will （ from DOM 1 ）。
 *
 * ── Why not ──────────────────────────────────
 * tbody React ， 1 `<tr>` will in down 1 diff when 。 this 
 * in ** up 1 **（`.drop-before` / `.drop-after`）， by CSS 
 * inset box-shadow —— not 、 not ， not use and React 。
 * can in DOM up ， is that className by React and not 
 * （React in prop when DOM）。
 *
 * ── 「 to 」────────────────────────────
 * by use （） from ： in DOM no has ， DOM
 * not ， need 「 1 」（ store_order.rs）。
 *
 * ── when not to （）──────────────────────────────────
 * ****（）： or only“enabled / use ” when ； by after 
 * can ， by 。 its 、、 down and 。
 * ****（ ）： is ** **， by 「 」 in 
 * （= to can before ）， by use by members —— can and 
 * is ， not will ，** not **。
 */

/** ：`index` is （0 ， already in that 1 ） */
export type QueueDrop =
  | { kind: 'group'; provider: string; index: number }
  | { kind: 'row'; provider: string; accountId: string; index: number }

export type QueueDragHandlers = {
  /** by by and ； not limit。 */
  isLocked(): boolean
  /** when use is by down ： to 1 （ by ） */
  onLockedAttempt(): void
  /**
   * this 1 is then （ then ）。** has will **：
   * then when will ， and to 1 。
   * （ not ）= not then ， and this before as 1 。
   */
  isRowLocked?(provider: string): boolean
  /** then down ： to 1 */
  onRowLockedAttempt?(provider: string): void
  /** ： new ， by use */
  onDrop(drop: QueueDrop): void
}

type DragState = {
  kind: 'group' | 'row'
  provider: string
  accountId: string
  item: HTMLElement
  container: HTMLElement
  items: HTMLElement[]
  /** current（ in that 1 down ） */
  index: number
}

/** in / （ and set when 1 ） */
const DRAGGING_CLASS = 'dragging'
/** ： in this 1 before / after */
const DROP_CLASSES = ['drop-before', 'drop-after']

let drag: DragState | null = null

function clearDropMarks(container: ParentNode): void {
  container.querySelectorAll(`.${DROP_CLASSES[0]}, .${DROP_CLASSES[1]}`).forEach(element => {
    element.classList.remove(...DROP_CLASSES)
  })
}

/** 1 「」： （ = tbody， = 1 tr） */
function siblingsOf(item: HTMLElement, kind: 'group' | 'row'): HTMLElement[] {
  if (kind === 'group') {
    const list = item.parentElement
    return list ? ([...list.children].filter(element =>
      element.classList.contains('acct-group')) as HTMLElement[]) : []
  }
  const tbody = item.parentElement
  return tbody ? ([...tbody.children].filter(element =>
    element.classList.contains('acct-row')) as HTMLElement[]) : []
}

/**
 * in 1 ： 1 in ，return** in its down **。
 *
 * and 「 to 1 」 for ： in to before / after ，
 * and set 1 。
 */
function insertionIndex(others: HTMLElement[], clientY: number): number {
  for (let index = 0; index < others.length; index += 1) {
    const rect = others[index].getBoundingClientRect()
    if (clientY < rect.top + rect.height / 2) return index
  }
  return others.length
}

function paintDropMark(state: DragState): void {
  const others = state.items.filter(item => item !== state.item)
  clearDropMarks(state.container)
  if (!others.length) return
  if (state.index >= others.length) others[others.length - 1].classList.add(DROP_CLASSES[1])
  else others[state.index].classList.add(DROP_CLASSES[0])
}

function onPointerMove(event: PointerEvent): void {
  if (!drag) return
  if (event.buttons === 0) { finish(false); return }
  const others = drag.items.filter(item => item !== drag!.item)
  const next = insertionIndex(others, event.clientY)
  if (next === drag.index) return
  drag.index = next
  paintDropMark(drag)
}

function finish(drop: boolean): void {
  const state = drag
  if (!state) return
  drag = null
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  window.removeEventListener('pointercancel', onPointerCancel)
  window.removeEventListener('keydown', onKeyDown)
  document.body.classList.remove('queue-dragging')
  state.item.classList.remove(DRAGGING_CLASS)
  clearDropMarks(state.container)
  if (!drop) return
  if (state.kind === 'group') {
    handlersRef?.onDrop({ kind: 'group', provider: state.provider, index: state.index })
  } else {
    handlersRef?.onDrop({ kind: 'row', provider: state.provider, accountId: state.accountId, index: state.index })
  }
}

function onPointerUp(): void { finish(true) }
function onPointerCancel(): void { finish(false) }

/** Esc = this （ and set 1 ） */
function onKeyDown(event: KeyboardEvent): void {
  if (event.key === 'Escape') finish(false)
}

/** 1 bind （finish in ， not to ） */
let handlersRef: QueueDragHandlers | null = null

/**
 * to up （****， 1 can ）： after React will tbody 1 ，
 * in up will ， in up then 1 valid —— and 1 by 。
 */
export function bindQueueDrag(host: HTMLElement | null, handlers: QueueDragHandlers): void {
  handlersRef = handlers
  if (!host) return
  // 1 1 ： use （） effect can can in ， addEventListener
  // will down —— has drag not will ， that is not is 
  if (host.dataset.queueDragBound === '1') return
  host.dataset.queueDragBound = '1'
  host.addEventListener('pointerdown', event => {
    if (event.button !== 0 || drag) return
    const target = event.target as Element | null
    const groupGrip = target?.closest?.('.queue-grip')
    const rowGrip = target?.closest?.('.order-grip')
    if (!groupGrip && !rowGrip) return
    event.preventDefault()
    // ****（groupGrip）： can by can ， down 
    if (groupGrip && handlers.isLocked()) {
      handlers.onLockedAttempt()
      return
    }
    const item = (groupGrip ?? rowGrip)!.closest<HTMLElement>('tbody.acct-group, tr.acct-row')
    if (!item) return
    const kind: 'group' | 'row' = groupGrip ? 'group' : 'row'
    const provider = item.dataset.provider
      || item.closest<HTMLElement>('tbody.acct-group')?.dataset.provider
      || ''
    // then when ， then （ will ）： in this 1 ，
    // not is use 1 no 
    if (kind === 'row' && provider && handlers.isRowLocked?.(provider)) {
      handlers.onRowLockedAttempt?.(provider)
      return
    }
    const accountId = item.dataset.id || ''
    if (kind === 'row' && !accountId) return
    const items = siblingsOf(item, kind)
    if (items.length < 2) return
    // down 1 by 「** ** that 1 」 （ and `insertionIndex` ，
    // otherwise and after will 1 —— in items ）
    const others = items.filter(candidate => candidate !== item)
    drag = {
      kind, provider, accountId, item,
      container: item.parentElement || host,
      items,
      index: insertionIndex(others, event.clientY),
    }
    item.classList.add(DRAGGING_CLASS)
    document.body.classList.add('queue-dragging')
    window.addEventListener('pointermove', onPointerMove)
    window.addEventListener('pointerup', onPointerUp)
    window.addEventListener('pointercancel', onPointerCancel)
    window.addEventListener('keydown', onKeyDown)
  })
}
