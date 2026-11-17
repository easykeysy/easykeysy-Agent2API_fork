import * as React from 'react'
import {
  Button, Checkbox, Dialog, DialogBody, DialogContent, DialogDescription,
  DialogFooter, DialogTitle, Label,
} from '../ui-kit/src'
import { acknowledgeProxyRetirement } from './proxy-retirement'

type Translate = (key: string) => string

export function ProxyRetirementDialog({ open, onClose, t, scope = '' }: {
  open: boolean
  onClose: () => void
  t: Translate
  scope?: string
}) {
  const [hide, setHide] = React.useState(false)
  const checkboxId = React.useId()
  React.useEffect(() => { if (open) setHide(false) }, [open])

  function acknowledge(): void {
    if (hide) acknowledgeProxyRetirement(scope)
    onClose()
  }

  return (
    <Dialog open={open} disablePointerDismissal onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent className='max-w-[540px]'>
        <DialogBody className='gap-0 px-7 pt-7 pb-6 max-sm:px-5'>
          <div className='mb-5 grid size-11 place-items-center rounded-md border border-primary-bd bg-primary-soft text-primary'>
            <svg width='23' height='23' viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='1.5' aria-hidden='true'>
              <circle cx='12' cy='12' r='9' /><path d='M12 11v5m0-9h.01' strokeLinecap='round' />
            </svg>
          </div>
          <span className='text-[11px] tracking-wide text-muted-foreground'>{t('Feature change notice')}</span>
          <DialogTitle className='mt-1.5 text-[22px] font-semibold leading-snug max-sm:text-xl'>
            {t('Proxy subscriptions have been discontinued')}
          </DialogTitle>
          <DialogDescription className='mt-3 text-xs leading-[1.9] text-subtle'>
            {t('To reduce maintenance costs, resource usage and compliance risks, Agent2API no longer provides proxy subscriptions or built-in proxy kernel features.')}
          </DialogDescription>
          <div className='mt-5 divide-y divide-border overflow-hidden rounded-md border border-border bg-surface-2'>
            <Change retained={false} title={t('No longer available')}>
              {t('Subscription import and updates, subscription node management, and proxy kernel download, installation and execution.')}
            </Change>
            <Change retained title={t('Manual proxies and Clash Verge sync remain available')}>
              {t('Existing manual configurations and Clash Verge endpoint references are not affected. You can still add HTTP / SOCKS5 addresses, sync Clash Verge endpoints and test connectivity. Install and run Clash Verge separately.')}
            </Change>
          </div>
          <div className='mt-5 text-xs leading-[1.85]'>
            <strong className='font-semibold'>{t('Previously using subscription nodes?')}</strong>
            <p className='mt-1.5 text-muted-foreground'>
              {t('Enter the HTTP / SOCKS5 address of an existing proxy service, or run Clash Verge separately and sync its endpoints, then select a proxy again for the affected accounts and service routes.')}
            </p>
          </div>
        </DialogBody>
        <DialogFooter className='justify-between px-7 max-sm:px-5'>
          <Label htmlFor={checkboxId} className='inline-flex cursor-pointer items-center gap-2 text-xs font-normal text-subtle'>
            <Checkbox id={checkboxId} checked={hide} onCheckedChange={next => setHide(next === true)} />
            {t('Do not show again')}
          </Label>
          <Button onClick={acknowledge}>{t('Got it')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function Change({ retained, title, children }: { retained: boolean; title: string; children: React.ReactNode }) {
  return (
    <div className='flex items-start gap-3 p-4'>
      <span className={`mt-0.5 grid size-[22px] shrink-0 place-items-center rounded-full ${retained ? 'bg-success-soft text-success' : 'bg-border text-subtle'}`}>
        <svg width='13' height='13' viewBox='0 0 24 24' fill='none' stroke='currentColor' strokeWidth='1.8' strokeLinecap='round' aria-hidden='true'>
          <path d={retained ? 'm5 12 4 4L19 6' : 'm6 6 12 12M6 18 18 6'} />
        </svg>
      </span>
      <div><strong className='text-xs font-semibold'>{title}</strong><p className='mt-1 text-xs leading-[1.85] text-subtle'>{children}</p></div>
    </div>
  )
}
