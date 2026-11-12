import { flushSync } from 'react-dom'
import { createRoot } from 'react-dom/client'

import { InputControl, type InputControlProps } from './input-control'

/**
 * —— use ：
 *
 * 1. ****（，）： in `<input>` up `data-island-input`，
 * in when Input。 id will to input，
 * `$('id').value` / `el.addEventListener('input', …)` 1 not use 。
 * `data-island-icon="⌕"` InputGroup（ InputGroupAddon），
 * as `.input-affix` will 、none need use 。
 * 、、 by MutationObserver ，none need 
 * use —— and select.js / desktop-tooltip.tsx 「」 is 1 。
 *
 * 2. ****：`mount(host, options)` —— to ， by 
 * parameter（ models ， is `<div class="island">`）。
 *
 * is **** ， by has ； need return `setValue`。
 * by input-control.tsx （ will etc React flush， when ）。
 */

type InputIslandOptions = {
  /** */
  value?: string
  placeholder?: string
  /** input ，default search */
  type?: string
  /** none。 has placeholder when ， to 1 */
  ariaLabel?: string
  /** before （ ⌕） */
  icon?: string
  onInput?: (value: string) => void
}

type InputIsland = {
  /** （empty / ）。 can this ， not onInput。 */
  setValue(value: string): void
  /** current（ when use ； has that ） */
  getValue(): string
  focus(): void
  unmount(): void
}

/** to */
function mountInput(host: HTMLElement, options: InputIslandOptions): InputIsland {
  const root = createRoot(host)
  /** React after up ；mount return when if setValue can can no */
  const domRef: { current: HTMLInputElement | null } = { current: null }

  root.render(
    <InputControl
      domRef={domRef}
      defaultValue={options.value}
      placeholder={options.placeholder}
      // default search： is empty div， no has 「」 can 。 this default not can ——
      // none type <input> not `input[type=…]` （ need 
      // in ）， and flex: 1 1 auto will 1 empty，
      // default、。
      type={options.type ?? 'search'}
      icon={options.icon}
      aria-label={options.ariaLabel}
      onInput={options.onInput}
    />
  )

  return {
    setValue(value: string) {
      const element = domRef.current
      if (element && element.value !== value) element.value = value
    },
    getValue() {
      return domRef.current?.value ?? ''
    },
    focus() {
      domRef.current?.focus()
    },
    unmount() {
      root.unmount()
    },
  }
}

/** `<input>` up 「config」 to props （ and by ， not ） */
const CARRIED_ATTRS = [
  ['id', 'id'],
  ['type', 'type'],
  ['name', 'name'],
  ['title', 'title'],
  ['placeholder', 'placeholder'],
  ['min', 'min'],
  ['max', 'max'],
  ['step', 'step'],
  ['autocomplete', 'autoComplete'],
] as const

function readInputProps(el: HTMLInputElement): Omit<InputControlProps, 'domRef'> {
  const props: Omit<InputControlProps, 'domRef'> = {
    // ： is 
    defaultValue: el.value || undefined,
    inputClassName: el.className || undefined,
  }
  for (const [attr, key] of CARRIED_ATTRS) {
    const value = el.getAttribute(attr)
    if (value != null) (props as Record<string, unknown>)[key] = value
  }
  const maxLength = el.getAttribute('maxlength')
  if (maxLength != null) props.maxLength = Number(maxLength)
  const inputMode = el.getAttribute('inputmode')
  if (inputMode != null) props.inputMode = inputMode as InputControlProps['inputMode']
  if (el.hasAttribute('spellcheck')) props.spellCheck = el.spellcheck
  if (el.disabled) props.disabled = true
  if (el.readOnly) props.readOnly = true
  // need up ： 「 etc 」 is hidden ， by 
  // models-reasoning.js by down in （ .hidden / .disabled， is 
  // DOM ， after ）。， this 1 will 。
  if (el.hidden) props.hidden = true
  const ariaLabel = el.getAttribute('aria-label')
  if (ariaLabel != null) props['aria-label'] = ariaLabel
  // data-* ： `[data-task-interval]` 、
  // `target.matches('[data-task-interval]')` ， 1 this 。
  // data-island-* is ， not 。
  for (const attr of el.getAttributeNames()) {
    if (!attr.startsWith('data-') || attr.startsWith('data-island-')) continue
    ;(props as Record<string, unknown>)[attr] = el.getAttribute(attr) ?? ''
  }
  const icon = el.getAttribute('data-island-icon')
  if (icon != null) props.icon = icon
  return props
}

/**
 * 1 `<input data-island-input>`。
 *
 * use `flushSync` ： is in ， its after 
 * and —— if to ， `$('id')` will to null。
 * after ， that 1 new input id in 。
 *
 * ： is 「 `.input-affix` + input」， 
 * in InputGroup —— and ， 1 will 
 * （`.input-affix input` 28px up addon ）。
 * ****： `input-affix` 、 flex ， and id in 
 * up （`.fm-search { width: 240px }` this then 、`#add-search-wrap` this 
 * in up ， 1 ）。 no has when 1 `display: contents`
 * —— not ， and ， before after etc 。
 */
function upgradeInput(el: HTMLInputElement) {
  if (el.dataset.islandUpgraded) return
  // in up ： already down ， to not will 
  el.dataset.islandUpgraded = '1'

  const props = readInputProps(el)
  const shell = el.parentElement?.classList.contains('input-affix') ? el.parentElement : null

  let host: HTMLElement
  if (shell) {
    shell.className = shell.className.replace(/\binput-affix\b/, '').trim()
    shell.style.display = 'flex'
    shell.style.alignItems = 'center'
    host = shell
  } else {
    host = document.createElement('span')
    host.style.display = 'contents'
    el.replaceWith(host)
  }

  const domRef: { current: HTMLInputElement | null } = { current: null }
  const root = createRoot(host)
  flushSync(() => {
    // React will empty old （ input）， not 
    root.render(<InputControl {...props} domRef={domRef} />)
  })
  // not ，；root 1 ，none need unmount
  void root
}

/** root down all （ root ） */
function autoMountInputs(root: ParentNode = document) {
  if (root instanceof HTMLInputElement && root.matches('[data-island-input]')) {
    upgradeInput(root)
    return
  }
  root.querySelectorAll<HTMLInputElement>('input[data-island-input]').forEach(upgradeInput)
}

/**
 * ： already has ， after 
 * （、、 is when HTML）。
 *
 * not will ： is `<span>`， not `input[data-island-input]`。
 */
function observeInputs() {
  autoMountInputs()
  const observer = new MutationObserver(records => {
    for (const record of records) {
      for (const node of record.addedNodes) {
        if (node instanceof Element) autoMountInputs(node)
      }
    }
  })
  observer.observe(document.body, { childList: true, subtree: true })
  return observer
}

export {
  autoMountInputs,
  mountInput,
  observeInputs,
  type InputIsland,
  type InputIslandOptions,
}
