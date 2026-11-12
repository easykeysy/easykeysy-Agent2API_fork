import * as React from 'react'
import { Input, InputGroup, InputGroupAddon, InputGroupInput, cn } from '@ui'

/**
 * ： 「 use 」 。
 *
 * is shadcn （Input / InputGroup）， this ：
 * `domRef` for ↔ ref、`onInput(value)` ↔ ； its props 。
 *
 * has when InputGroup（ InputGroupAddon）， no has when is 
 * 1 Input —— not has icon prop and 。
 *
 * to need ：
 * · `className` → in **** up （InputGroup）。 and flex then 
 * （ `.fm-search { width: 240px }`） in this 。
 * · `inputClassName` → in **** up （ `<input>` ）。
 *
 * has 1 must by ：****。 Input / InputGroup default `w-full`
 * （shadcn use is 1 or ）， 1 
 * `input[type=…] { flex: 1 1 auto }` in and —— this 1 ，
 * that `flex-wrap` will 、 by to down 1 。
 */

/**
 * and CSS etc ：`flex: 1 1 auto` + `width: auto`。
 * `w-auto` is as `w-full`（tailwind-merge by ， after ）。
 */
const layoutClass = 'w-auto flex-auto'

type InputControlProps = {
  id?: string
  /** input ，default search（ by ） */
  type?: string
  /** （， after by has ） */
  defaultValue?: string
  /** （「 etc 」）， by DOM */
  hidden?: boolean
  placeholder?: string
  /** in （InputGroup） up ： then */
  className?: string
  /** `<input>` ， to */
  inputClassName?: string
  min?: string | number
  max?: string | number
  step?: string | number
  maxLength?: number
  inputMode?: React.HTMLAttributes<HTMLInputElement>['inputMode']
  autoComplete?: string
  spellCheck?: boolean
  disabled?: boolean
  readOnly?: boolean
  name?: string
  title?: string
  'aria-label'?: string
  /** before （ ⌕）； not 1 not InputGroup */
  icon?: string
  onInput?: (value: string) => void
  /** to DOM，used for and */
  domRef: { current: HTMLInputElement | null }
}

function InputControl({
  onInput,
  domRef,
  icon,
  className,
  inputClassName,
  ...props
}: InputControlProps) {
  const attach = (node: HTMLElement | null) => {
    domRef.current = node as HTMLInputElement | null
  }
  const handleInput = (event: React.FormEvent<HTMLInputElement>) => {
    onInput?.(event.currentTarget.value)
  }

  if (!icon) {
    return (
      <Input
        ref={attach}
        className={cn(layoutClass, inputClassName)}
        onInput={handleInput}
        {...props}
      />
    )
  }

  return (
    <InputGroup className={cn(layoutClass, className)}>
      {/* in addon before ：Tab is 「 after addon」， by align */}
      <InputGroupInput
        ref={attach}
        className={inputClassName}
        onInput={handleInput}
        {...props}
      />
      <InputGroupAddon aria-hidden='true'>{icon}</InputGroupAddon>
    </InputGroup>
  )
}

export { InputControl, type InputControlProps }
