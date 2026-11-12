/**
 * Agent2API · 「 / 」 use 。
 *
 * this 1 1 macOS set that 1 （ in 、、
 * by ）， by this 。 in file Reason： 
 * add-provider-blocks / add-dynamic-provider / add-custom-provider use ，
 * in its in 1 up will to 。
 *
 * ── Why need use portal ────────────────────────────────
 * （web-login.js / sms-login.js / autoclaw-oauth.js） by **DOM id**
 * by and （`document.getElementById`）， not —— by from 
 * to not 1 。DeepSeek that 1 is this 
 * （ add-provider-deepseek-block footerHost）， this is use 。
 *
 * `active` must and in can 1 ：portal not in that 
 * `hidden` ， when if ， will 1 by 。
 */

import * as React from 'react'
import { createPortal } from 'react-dom'

/**
 * 1 set： in ， 1px （ in styles/add-account-dialog.css）。
 * `action` is ；`hidden` and DialogSection etc 。
 */
export function AddGroup({ label, action, hidden, onClick, children }: {
  label: React.ReactNode
  action?: React.ReactNode
  hidden?: boolean
  /** （ that 「file in 」 in up ） */
  onClick?: React.MouseEventHandler<HTMLElement>
  children: React.ReactNode
}): React.ReactElement {
  return (
    <section className='add-group' hidden={hidden} onClick={onClick}>
      <div className='add-group-label'>
        <span>{label}</span>
        {action ? <span className='label-action'>{action}</span> : null}
      </div>
      <div className='add-group-box'>{children}</div>
    </section>
  )
}

/**
 * by and ： to （`host`），、 by 。
 *
 * ── Why is 「」 not is 「 by need 」────────────────
 * sms-login.js and autoclaw-oauth.js in `create()` when by id to by click
 * （`$(\`${prefix}-sms-submit\`).addEventListener(...)`）， is 、
 * 。 if this 1 not when not ， not in ，`create()` that 1 
 * will empty —— use after to 「」 when by is 。： portal
 * ， to `hidden`， 1 in ，， when current this 1 。
 *
 * `host` as empty when （ not this ，）。
 */
export function AddFoot({ host, note, active = true, children }: {
  host: HTMLElement | null
  note?: React.ReactNode
  active?: boolean
  children: React.ReactNode
}): React.ReactElement {
  const content = <>
    <div className='acct-add-foot-note'>{note}</div>
    <div className='acct-add-foot-buttons'>{children}</div>
  </>
  if (!host) return <div className='acct-add-foot-inline'>{content}</div>
  return createPortal(<div className='acct-add-foot-slot' hidden={!active}>{content}</div>, host)
}
