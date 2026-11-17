import * as React from 'react'
import {
  Card,
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  SegmentedControl,
  useMediaQuery,
} from '@ui'
import { t } from '../i18n'

type SupportMethod = 'wechat' | 'alipay'

export function SupportDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const narrow = useMediaQuery('(max-width: 639px)')
  const [method, setMethod] = React.useState<SupportMethod>('wechat')
  const methods = [
    {
      value: 'wechat' as const,
      label: t('WeChat'),
      src: 'assets/support/wechat-reward.png',
      alt: t('WeChat donation code'),
      pixels: 710,
    },
    {
      value: 'alipay' as const,
      label: t('Alipay'),
      src: 'assets/support/alipay-receive.png',
      alt: t('Alipay payment QR code'),
      pixels: 1620,
    },
  ]

  return (
    <Dialog open={open} onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent className='w-[min(680px,calc(100vw-48px))]'>
        <DialogHeader>
          <DialogTitle>{t('Support the author')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <DialogDescription className='support-intro'>
            {t('Agent2API keeps improving because someone keeps investing time in it. If it has helped you, consider buying the author a coffee.')}
          </DialogDescription>
          {narrow ? (
            <SegmentedControl
              className='self-center'
              aria-label={t('Donation method')}
              options={methods}
              value={method}
              onValueChange={setMethod}
            />
          ) : null}
          <div className='support-codes' data-narrow={narrow}>
            {methods.filter(item => !narrow || item.value === method).map(item => (
              <Card key={item.value} render={<figure />} size='sm' className='support-code-card'>
                <figcaption className='support-code-title'>{item.label}</figcaption>
                <img
                  className='support-code-image'
                  src={item.src}
                  alt={item.alt}
                  width={item.pixels}
                  height={item.pixels}
                />
              </Card>
            ))}
          </div>
          <p className='support-note'>
            {t('Donations are entirely voluntary. They do not unlock any features or affect normal use.')}
          </p>
        </DialogBody>
      </DialogContent>
    </Dialog>
  )
}
