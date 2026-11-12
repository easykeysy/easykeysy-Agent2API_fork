import { useState } from 'react'
import { Icon } from './Icon'
import { usePwa } from '../lib/usePwa'
import { useStore } from '../state/store'
import { applyUpdate } from '../lib/pwa'
import { t } from '../i18n'
import { Alert, Button } from '@ui'

/**
 * PWA 。
 *
 * ， not ：
 * · ****：。 is 「Why up not 」，
 * in up 、 1 can to ， and not 。
 * · ** has new **：。 need need 1 （），
 * in to ； when to 「 after 」， not current。
 *
 * is `.m-app` flex / ， not ： 
 * down ，update in up 。
 *
 * set not update： that 1 「 use 」 by ，
 * 1 is 。
 */
export function PwaNotice() {
  const pwa = usePwa()
  const { route } = useStore()
  const [busy, setBusy] = useState(false)
  // for will valid： new SW in etc ， down will 1 
  const [snoozed, setSnoozed] = useState(false)
  const showUpdate = pwa.updateReady && !snoozed && route.screen !== 'settings'

  return (
    <>
      {pwa.offline ? (
        <Alert variant='warning' className='m-notice-top rounded-none border-x-0 border-t-0' role='status'>
          <Icon name='alert' />
          <span>{t('已离线 · 显示的是缓存内容')}</span>
        </Alert>
      ) : null}

      {showUpdate ? (
        <Alert className='m-notice-update' role='status'>
          <span className='m-notice-text'>{t('有新版本，重启后生效')}</span>
          <Button size='touch-sm' variant='default'
            type='button'
            className='m-btn m-sm'
            disabled={busy}
            onClick={() => {
              setBusy(true)
              void applyUpdate()
            }}
          >
            {busy ? t('重启中…') : t('重启更新')}
          </Button>
          <Button size='default' variant='ghost' type='button' className='m-notice-later' onClick={() => setSnoozed(true)}>
            {t('稍后')}
          </Button>
        </Alert>
      ) : null}
    </>
  )
}
