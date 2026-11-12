import * as React from 'react'
import { createRoot } from 'react-dom/client'
import {
  AlertDialog,
  AlertDialogBanner,
  AlertDialogBody,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  HtmlContent,
} from '@ui'

/**
 * Agent2API · use （ confirm）。
 *
 * ui/confirm-dialog.js —— that use innerHTML .modal-mask / .modal that ，
 * in index.html #confirm-modal 。 for and ** 1 **：
 * `window.wbConfirm.ask(options?)`，13 use 1 not use 。
 *
 * Why this not can ：Tauri WebView window.confirm() not 、
 * （return true），all （empty / delete） etc no has 。
 *
 * ── and AlertDialog （file ）─────────────
 * Base UI AlertDialog is 「」： by disablePointerDismissal
 * ， not to Esc this 。 has need 「 / Esc etc 」
 * （ use to false in ）， this up this —— 
 * that useEffect 。（z-38、min(440px,…)、danger-zone ）
 * by ui-kit AlertDialogContent / AlertDialogBanner ， not 。
 */

type AskOptions = {
  /** （） */
  title?: string
  /** HTML， use —— */
  html?: string
  /** （ and html 1 ，，\n <br>） */
  text?: string
  /** （「empty」「delete」） */
  okText?: string
  /** ：primary（default） or danger（ not can ） */
  okClass?: string
  /** ：danger-zone（default，）；empty */
  bodyClass?: string
  /** */
  cancelText?: string
}

/** and old 1 （Note: not ， can can and use HTML not 1 ） */
function escapeHtml(value: unknown): string {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

type ConfirmDialogProps = {
  options: AskOptions
  /** ：true = ，false = / ✕ / / Esc */
  onSettle: (ok: boolean) => void
}

function ConfirmDialog({ options, onSettle }: ConfirmDialogProps) {
  const {
    title = '确认操作',
    html = '',
    text = '',
    okText = '确定',
    okClass = 'primary',
    bodyClass = 'danger-zone',
    cancelText = '取消',
  } = options

  // need in 「」 up ： not can ， not this etc 。
  // use AlertDialogContent initialFocus not is by up autoFocus —— 
  // when will ，autoFocus will 。
  const cancelRef = React.useRef<HTMLButtonElement | null>(null)

  /**
   * Esc = 。AlertDialog is not to this ， has need （ old 
   * 13 use to false）， 1 document 。
   * not stopPropagation： old no ，app.js / settings-panel that 
   * Esc old ， as not 。
   */
  React.useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') onSettle(false)
    }
    document.addEventListener('keydown', onKeyDown)
    return () => document.removeEventListener('keydown', onKeyDown)
  }, [onSettle])

  /**
   * = 。AlertDialog by in Base UI disablePointerDismissal
   * ， ui-kit AlertDialogContent in 、 not props，
   * no in up onClick —— can to document up ： is 
   * （data-slot is ）「」。
   * in up by down 、 to up when ，click is （portal ），
   * not will ； not will 。
   */
  React.useEffect(() => {
    function onClick(event: MouseEvent) {
      const target = event.target as HTMLElement | null
      if (target?.dataset.slot === 'alert-dialog-overlay') onSettle(false)
    }
    document.addEventListener('click', onClick)
    return () => document.removeEventListener('click', onClick)
  }, [onSettle])

  return (
    // open（ as true）： is 「 when 、」， 1 by onSettle
    // 。onOpenChange use ✕ this by Base UI request。
    <AlertDialog open onOpenChange={next => { if (!next) onSettle(false) }}>
      {/* ✕ ： has is 「」 1 （ AlertDialog default not this 
          ，default is ）。 Base UI Close → onOpenChange(false)。 */}
      <AlertDialogContent initialFocus={cancelRef} showCloseButton>
        {/* pr-10 to up ✕ ， not will to down */}
        <AlertDialogHeader className='pr-10'>
          <AlertDialogTitle>{title}</AlertDialogTitle>
        </AlertDialogHeader>
        <AlertDialogBody>
          {/* html and text 1 ：html empty use （ use ，，
               has use will <strong>/<code>/<b> this ），otherwise use after text
               and <br>。 */}
          <AlertDialogBanner tone={bodyClass === 'danger-zone' ? 'danger' : 'neutral'}>
            <HtmlContent html={html || escapeHtml(text).replace(/\n/g, '<br>')} />
          </AlertDialogBanner>
        </AlertDialogBody>
        <AlertDialogFooter>
          <div className='mr-auto' />
          <Button ref={cancelRef} variant='outline' onClick={() => onSettle(false)}>
            {cancelText}
          </Button>
          {/* danger destructive（ + ）， its 1 default */}
          <Button
            variant={okClass === 'danger' ? 'destructive' : 'default'}
            onClick={() => onSettle(true)}
          >
            {okText}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

/* ─── ： and old window.wbConfirm 1 ─── */

let root: ReturnType<typeof createRoot> | null = null
let host: HTMLElement | null = null
/** current etc ；empty（ and old resolver ） */
let pending: ((ok: boolean) => void) | null = null

function unmountDialog() {
  if (root) {
    root.unmount()
    root = null
  }
  if (host) {
    host.remove()
    host = null
  }
}

/** ： and to etc （ use none use ） */
function settle(ok: boolean) {
  const resolve = pending
  pending = null
  unmountDialog()
  resolve?.(ok)
}

function ask(options: AskOptions = {}): Promise<boolean> {
  return new Promise<boolean>(resolve => {
    // when 1 ： when before 1 by 「」，otherwise before 1 use Promise
    // （ old 1 to ）。 settle new ，。
    if (pending) settle(false)
    pending = resolve
    host = document.createElement('div')
    document.body.append(host)
    root = createRoot(host)
    root.render(<ConfirmDialog options={options} onSettle={settle} />)
  })
}

declare global {
  interface Window {
    wbConfirm?: {
      ask(options?: AskOptions): Promise<boolean>
    }
  }
}

window.wbConfirm = { ask }
