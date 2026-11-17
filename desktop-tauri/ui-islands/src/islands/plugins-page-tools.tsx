import * as React from 'react'
import { Button, Dialog, DialogBody, DialogContent, DialogHeader, DialogTitle, Popover, PopoverContent,
  PopoverTitle, PopoverTrigger } from '@ui'
import type { PluginsPayload } from './plugins-types'
import { GithubFailureText } from '../../../ui-shared/github-failure'
import { t } from '../i18n'

export function PluginPageTools({ disabled, fileDisabled, scriptDisabled, scriptsAvailable, checking,
  onRefresh, onCheck, onSettings, onFile, onScript, onDiagnostics }: {
  disabled: boolean
  fileDisabled: boolean
  scriptDisabled: boolean
  scriptsAvailable: boolean
  checking: boolean
  onRefresh: () => void
  onCheck: () => void
  onSettings: () => void
  onFile: () => void
  onScript: () => void
  onDiagnostics: () => void
}) {
  const [open, setOpen] = React.useState(false)
  const actions = [
    { label: checking ? t('Checking…') : t('Check for updates'), disabled: disabled || checking, action: onCheck },
    { label: t('Refresh'), disabled, action: onRefresh },
    { label: t('Plugin settings'), disabled, action: onSettings },
    { label: t('Install from file…'), disabled: fileDisabled, action: onFile },
    ...(scriptsAvailable ? [{ label: t('Install from npm / GitHub'), disabled: scriptDisabled, action: onScript }] : []),
    { label: t('Diagnostics'), disabled: false, action: onDiagnostics },
  ]
  return <Popover open={open} onOpenChange={setOpen}>
    <PopoverTrigger render={<Button variant='ghost' size='icon' aria-label={t('More actions')} title={t('More actions')} />}>
      <svg viewBox='0 0 24 24' className='size-4' fill='currentColor' aria-hidden='true'>
        <circle cx='5' cy='12' r='1.7' /><circle cx='12' cy='12' r='1.7' /><circle cx='19' cy='12' r='1.7' />
      </svg>
    </PopoverTrigger>
    <PopoverContent align='end' className='w-56 p-1.5'>
      <PopoverTitle className='sr-only'>{t('More actions')}</PopoverTitle>
      {actions.map(item => <Button key={item.label} variant='ghost' className='justify-start' disabled={item.disabled}
        onClick={() => { setOpen(false); item.action() }}>{item.label}</Button>)}
    </PopoverContent>
  </Popover>
}

export function PluginDiagnostics({ open, payload, onClose }: {
  open: boolean
  payload: PluginsPayload | null
  onClose: () => void
}) {
  return <Dialog open={open} onOpenChange={next => { if (!next) onClose() }}>
    <DialogContent className='text-foreground'>
      <DialogHeader><DialogTitle>{t('Diagnostics')}</DialogTitle></DialogHeader>
      <DialogBody className='gap-3 text-[12px] text-subtle'>
        <div>{t('Plugin directory')} <code className='break-all'>{payload?.pluginsDir || '—'}</code></div>
        <div>{t('Current platform')} <code>{payload?.target || '—'}</code></div>
        <div>{t('Protocol version')} {payload?.protocol || '—'}</div>
        {payload?.catalog.reason ? <div className='text-destructive'><GithubFailureText value={payload.catalog} t={t} /></div> : null}
        {payload?.host?.lastError ? <div className='break-words text-destructive'>{payload.host.lastError}</div> : null}
        {payload?.scripts ? <div className='flex flex-col gap-2 border-t border-hairline pt-3'>
          <span>Bun {payload.scripts.runtime.version} · {payload.scripts.runtime.installed ? t('Downloaded') : t('Download on demand')}</span>
          {payload.scripts.runtime.path ? <code className='break-all'>{payload.scripts.runtime.path}</code> : null}
          {payload.scripts.runtime.error ? <p className='m-0 whitespace-pre-wrap break-words text-destructive'>{payload.scripts.runtime.error}</p> : null}
          <p className='m-0'>{t('Load compatible Magpie / OpenCode v1 provider packages directly. Multiple plugins share one Bun process.')}</p>
          <p className='m-0'>{t('Currently supports provider plugins without runtime npm dependencies. Magpie pages, Matrix tools and Pi extensions are not loaded.')}</p>
          <p className='m-0'>{t('Supports automatic OAuth/device-code sign-in and API keys. Plugins requiring manual callbacks are not supported yet.')}</p>
        </div> : null}
      </DialogBody>
    </DialogContent>
  </Dialog>
}
