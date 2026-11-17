import * as React from 'react'
import {
  Badge, Button, Dialog, DialogBody, DialogContent, DialogDescription, DialogFooter,
  DialogHeader, DialogSection, DialogTitle, Input, Label, Switch,
} from '@ui'
import { api, formatTime, pluginDescription, pluginName } from './plugins-shared'
import type { PackProvider, PackSummary, PluginOperation, PluginUpdate, StorePlugin } from './plugins-types'
import { PluginArtwork } from '../../../ui-shared/plugins-store'
import { PluginHelpers } from '../../../ui-shared/plugin-helpers'
import { PluginStartupDetails, requestPluginDiagnostics, usePluginLoadStatus } from '../../../ui-shared/plugin-startup-view'
import { pluginPhaseLabel } from '../../../ui-shared/plugin-startup'
import { PluginUpdateDetails } from './plugins-cards'
import { t } from '../i18n'

export function PluginPackSettingsDialog({ pack, store, update, operation, disabled, onClose, onEnabled, onProvider,
  onRemove, onUpdate, onNotes }: {
  pack: PackSummary
  store?: StorePlugin
  update?: PluginUpdate
  operation?: PluginOperation
  disabled: boolean
  onClose: () => void
  onEnabled: (enabled: boolean) => void
  onProvider: (provider: PackProvider, enabled: boolean) => void
  onRemove: () => void
  onUpdate: () => void
  onNotes: () => void
}) {
  const startup = usePluginLoadStatus(pack.id, pack.startup)
  const [search, setSearch] = React.useState('')
  const [permissionBusy, setPermissionBusy] = React.useState(false)
  const locked = disabled || permissionBusy
  const idPrefix = React.useId()
  const keyword = search.trim().toLocaleLowerCase()
  const providers = pack.providers.filter(provider => !keyword
    || `${t(provider.label)} ${provider.id}`.toLocaleLowerCase().includes(keyword))

  return <Dialog open onOpenChange={(next, details) => { if (!next) { if (permissionBusy) details.cancel(); else onClose() } }}>
    <DialogContent className='w-[min(560px,calc(100vw-48px))] text-foreground'>
      <DialogHeader className='gap-3'>
        <PluginArtwork src={pack.icon || store?.icon} kind={pack.id === 'agent2api-web-providers' ? 'web' : 'providers'} />
        <div className='min-w-0 flex-1'>
          <DialogTitle className='m-0 break-words'>{pluginName(pack.id, store?.name || pack.name)}</DialogTitle>
          <div className='mt-1 text-[11.5px] text-muted-foreground'>v{pack.version} · {t('Community plugin')}</div>
        </div>
      </DialogHeader>
      <DialogBody className='min-h-0 gap-4'>
        <DialogDescription className='text-[12.5px] leading-relaxed text-subtle'>
          {pluginDescription(pack.id, store?.description || pack.description)}
        </DialogDescription>
        <DialogSection>
          <Label htmlFor={`${idPrefix}-enabled`} className='justify-between text-[12.5px] text-foreground'>
            {t('enabled')}<Switch id={`${idPrefix}-enabled`} checked={pack.enabled} disabled={locked} onCheckedChange={onEnabled} />
          </Label>
          <div className='flex flex-wrap items-center gap-x-2 gap-y-1 text-[11.5px] text-muted-foreground'>
            <span className={pack.signed ? undefined : 'text-warning'}>{pack.signed ? t('Signed') : t('Unsigned')}</span>
            <span aria-hidden='true'>·</span><span>{t('Protocol version')} {pack.protocol}</span>
            {pack.installedAt ? <><span aria-hidden='true'>·</span>
              <span>{t('Installed {time}', { time: formatTime(pack.installedAt) })}</span></> : null}
          </div>
        </DialogSection>
        <DialogSection>
          <h3 className='m-0'>{t('Load status')}</h3>
          <PluginStartupDetails id={pack.id} status={startup} t={t} />
          <Button variant='link' size='sm' onClick={() => { onClose(); requestPluginDiagnostics(pack.id) }}>{t('Load diagnostics')}</Button>
        </DialogSection>
        <DialogSection>
          <h3 className='m-0'>{t('Version')}</h3>
          <PluginUpdateDetails pack={pack} update={update} operation={operation} disabled={locked} onUpdate={onUpdate} onNotes={onNotes} />
        </DialogSection>
        <DialogSection>
          <h3 className='m-0 flex items-baseline justify-between gap-3'>
            <span>{t('Provider switches')}</span>
            <span className='text-[11.5px] font-normal text-muted-foreground'>
              {pack.providers.filter(provider => provider.enabled && !provider.overriddenBy).length}/{pack.providers.length} {t('Enabled')}
            </span>
          </h3>
          <p className='m-0'>{t('Providers in this pack (each can be disabled separately)')}</p>
          {pack.providers.length > 6 ? <Input type='search' value={search} placeholder={t('Search providers')} aria-label={t('Search providers')}
            onChange={event => setSearch(event.currentTarget.value)} /> : null}
          <div className='max-h-[36vh] min-h-0 overflow-y-auto overscroll-contain'>
            {providers.map(provider => <div key={provider.id} className='flex items-center justify-between gap-3 border-t border-hairline py-2 first:border-t-0'>
              <Label htmlFor={`${idPrefix}-${provider.id}`} className='min-w-0 flex-1 flex-wrap text-[12.5px] text-foreground'>
                <span className='break-words'>{t(provider.label)}</span>
                {provider.overriddenBy ? <Badge variant='secondary' shape='tag'
                  title={t('This provider is managed by another plugin. Manage it in that plugin\'s settings.')}>{t('Managed by another plugin')}</Badge>
                  : provider.enabled && !provider.loaded ? <Badge variant='secondary' shape='tag'>{pluginPhaseLabel(startup, t)}</Badge> : null}
              </Label>
              <Switch id={`${idPrefix}-${provider.id}`} checked={provider.enabled && !provider.overriddenBy}
                disabled={locked || !pack.enabled || Boolean(provider.overriddenBy)}
                onCheckedChange={value => onProvider(provider, value)} />
            </div>)}
            {!providers.length ? <p className='m-0 py-3 text-center'>{t('No providers match "{keyword}"', { keyword: search.trim() })}</p> : null}
          </div>
        </DialogSection>
        <PluginHelpers key={`${pack.id}:${pack.version}`} packId={pack.id} providers={pack.providers} enabled={pack.enabled}
          disabled={disabled} onBusyChange={setPermissionBusy} request={api} t={t} />
      </DialogBody>
      <DialogFooter>
        <Button variant='destructive' size='sm' className='mr-auto' disabled={locked} onClick={onRemove}>{t('Uninstall plugin')}</Button>
        <Button variant='outline' size='sm' disabled={permissionBusy} onClick={onClose}>{t('Done')}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
}
