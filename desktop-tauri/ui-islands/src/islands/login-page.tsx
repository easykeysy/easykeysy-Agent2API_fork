/**
 * Agent2API · / （ui/login.html）—— React 。
 *
 * this 1 （ no has index.html that ， not .page）， by 
 * 。** that **： by id 
 * #username / #password / #submit / #error / #mode-tip， submit in 
 * #form up （capture ， its submit ）。
 * 「 1 not update」—— not state、 not ， for this 
 * （ by and use 、error、、） not will React 。
 * this file before that ， is this 1 as 。
 *
 * must（flushSync）： is after ， when need to 
 * this （`widget.addEventListener` to null will ，、
 * none）； createRoot().render() default to ， will in 
 * before 、 to empty DOM。 is `<script src="islands/login.js">` must in 
 * #login-app after ， and not can defer/async —— 。
 *
 * this ** not in island **： ui/islands/login.js
 * （ src/login-entry.tsx， vite.login.config.ts），otherwise as 
 * 1 need down 2.3 MB islands/ui.js。
 *
 * up not 1 ： 1 （ui.css --ui-*，
 * `<html data-theme="dark">`）， use Tailwind 。
 * （html/body、 widget） in login.html <style> 。
 */

import * as React from 'react'
import { flushSync } from 'react-dom'
import { createRoot } from 'react-dom/client'
import { Button, Card, Input, Label, Verification } from '@ui'
import { t } from '../i18n'

/** id：login.html 1 */
const HOST_ID = 'login-app'

/**
 * ：`<panel-captcha>` by ui/panel-captcha.js （
 * server::altcha，fileWhy not use altcha widget —— in 
 * http://<IP> this up down no has crypto.subtle， will 1 in 「 in …」）。
 *
 * use createElement not is JSX ：TS IntrinsicElements no has this ， as 1 
 * JSX empty will and its （ can can and and file）。
 * id：、、 in ， this not 。
 */
type CaptchaElement = HTMLElement & { state?: 'idle' | 'verifying' | 'verified' | 'error'; start?(): Promise<void> }

function CaptchaControl() {
  const [state, setState] = React.useState<'idle' | 'verifying' | 'verified' | 'error'>('idle')
  const ref = React.useRef<CaptchaElement | null>(null)
  React.useEffect(() => {
    const element = ref.current
    if (!element) return
    const sync = () => setState(element.getAttribute('state') as typeof state || 'idle')
    const observer = new MutationObserver(sync)
    observer.observe(element, { attributes: true, attributeFilter: ['state'] })
    sync()
    return () => observer.disconnect()
  }, [])
  return React.createElement('panel-captcha', { id: 'captcha', ref },
    <Verification state={state} className='mx-auto mt-5' onClick={() => { void ref.current?.start?.() }}>
      {state === 'verifying' ? t('验证中…') : state === 'verified' ? t('验证成功') : state === 'error' ? t('验证失败，请重试') : t('我不是机器人')}
    </Verification>,
  )
}

/**
 * and its in 。
 *
 * id 1 not can （file）。#mode-tip and #error empty： by 
 * ， this and （min-h-5 error before after not ）。
 */
function LoginPage() {
  return (
    <Card className='w-[min(400px,100%)] p-[36px_34px_30px] shadow-3'>
      {/* and / 1 （icons.js brand ）， is */}
      <div className='mb-[26px] flex flex-col items-center gap-3 text-center'>
        <svg
          viewBox='0 0 24 24'
          role='img'
          aria-label='Agent2API'
          className='size-[52px] rounded-lg shadow-[0_6px_18px_rgba(0,122,255,0.35)]'
        >
          <rect width='24' height='24' rx='5.4' fill='#007AFF' />
          <g fill='none' stroke='#fff' strokeWidth='2.2' strokeLinecap='round' strokeLinejoin='round'>
            <path d='M5.23 9.24h10.71' />
            <path d='M15.94 6.72 18.77 9.24 15.94 11.76' />
            <path d='M18.77 14.76H8.06' />
            <path d='M8.06 12.24 5.23 14.76 8.06 17.28' />
          </g>
        </svg>
        <div>
          <h1 className='m-0 text-[21px] font-bold tracking-[0.2px]'>Agent2API</h1>
          <div className='text-[12.5px] text-muted-foreground'>{t('OpenAI 兼容网关 · 管理面板')}</div>
        </div>
      </div>

      <div id='mode-tip' className='mb-[18px] text-center text-[13px] text-muted-foreground' />

      <form id='form' autoComplete='on'>
        <Label htmlFor='username' className='mt-3.5 mb-[5px] text-[12.5px]'>
          {t('账号')}
        </Label>
        <Input id='username' autoComplete='username' placeholder={t('管理员账号')} />
        <Label htmlFor='password' className='mt-3.5 mb-[5px] text-[12.5px]'>
          {t('密码')}
        </Label>
        <Input id='password' type='password' autoComplete='new-password' placeholder={t('至少 8 位')} />
        {/* after → already （ ui/panel-captcha.js） */}
        <CaptchaControl />
        {/* type='submit' must to ：Base UI useButton will to button 1 
            type="button"， has （ and when props ） */}
        <Button id='submit' type='submit' size='lg' className='w-full'>
          {t('继续')}
        </Button>
        <div id='error' className='mt-3 min-h-5 text-center text-[13px] whitespace-pre-wrap text-destructive' />
      </form>

      <div className='mt-[22px] text-center text-[12px] text-muted-foreground'>
        {t('登录后可在「网关 Key」页为 API 客户端创建密钥')}
      </div>
    </Card>
  )
}

/**
 * 。file by login.js ， in ； #login-app
 * in is as will has login.js to its —— that down 
 * can ， not this 。
 */
function mount(): void {
  const host = document.getElementById(HOST_ID)
  if (!host) return
  // ， by file
  flushSync(() => {
    createRoot(host).render(<LoginPage />)
  })
}

mount()
