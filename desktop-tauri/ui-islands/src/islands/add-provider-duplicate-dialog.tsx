import * as React from 'react'
import { Button, Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle, Item, ItemContent, ItemDescription, ItemTitle } from '@ui'

import type { CustomProviderRecord } from './add-account-bridge'
import { t } from '../i18n'

export function DuplicateProviderDialog({
  providers,
  onCreate,
  onExisting,
  onClose,
}: {
  providers: CustomProviderRecord[]
  onCreate: () => void
  onExisting: (provider: CustomProviderRecord) => void
  onClose: () => void
}): React.ReactElement {
  return (
    <Dialog open onOpenChange={open => { if (!open) onClose() }}>
      <DialogContent overlayForceRender className='w-[min(560px,calc(100vw-48px))]'>
        <DialogHeader>
          <DialogTitle>{t('A provider already uses this address')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <p className='mb-3 text-sm text-subtle'>{t('A provider with the same protocol and address already exists. Create a separate provider, or select an existing one to add an account.')}</p>
          <div className='space-y-2'>
            {providers.map(provider => (
              <Item key={provider.id} variant='outline'>
                <ItemContent>
                  <ItemTitle>{provider.name || provider.id}</ItemTitle>
                  <ItemDescription>{provider.baseUrl}</ItemDescription>
                </ItemContent>
                <Button variant='outline' onClick={() => onExisting(provider)}>
                  {t('Add to this provider')}
                </Button>
              </Item>
            ))}
          </div>
        </DialogBody>
        <DialogFooter>
          <Button variant='outline' onClick={onClose}>{t('Cancel')}</Button>
          <Button onClick={onCreate}>{t('Create a separate provider')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
