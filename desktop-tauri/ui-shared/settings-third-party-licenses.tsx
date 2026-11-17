import {
  Badge, Button, Collapsible, CollapsibleContent, CollapsibleTrigger,
  Dialog, DialogBody, DialogClose, DialogContent, DialogDescription,
  DialogFooter, DialogHeader, DialogTitle, DialogTrigger,
  Sheet, SheetBody, SheetClose, SheetContent, SheetDescription,
  SheetFooter, SheetHeader, SheetTitle, SheetTrigger,
} from '../ui-kit/src'
import { licenseNotices, sources } from '../../licenses/ICONS_SOURCE.json'

type Translate = (key: string) => string

type LicenseEntry = {
  id: string
  name: string
  repository: string
  sourceRevision?: string
  licenseRevision?: string
  license: string
  copyright: string
  text: string
}

const LICENSES: LicenseEntry[] = [
  {
    id: 'magpie',
    name: 'Magpie',
    repository: sources.magpie.repository,
    sourceRevision: sources.magpie.revision,
    ...licenseNotices.magpie,
  },
  {
    id: 'lobe-icons',
    name: 'Lobe Icons',
    repository: sources.lobeIcons.repository,
    licenseRevision: sources.lobeIcons.licenseRevisionVerified,
    ...licenseNotices.lobeIcons,
  },
  {
    id: 'opendesign',
    name: 'OpenDesign',
    repository: sources.openDesign.repository,
    sourceRevision: sources.openDesign.revision,
    ...licenseNotices.openDesign,
  },
]

type LicenseViewerProps = {
  t: Translate
  openExternal: (url: string) => void | Promise<void>
  mobile?: boolean
}

function LicenseList({ t, openExternal }: LicenseViewerProps) {
  return (
    <>
      <p className='text-xs leading-relaxed text-subtle'>
        {t('This list contains the icon source licenses currently recorded in the repository, not a complete inventory of third-party dependencies.')}
      </p>
      <div className='flex min-w-0 flex-col gap-3'>
        {LICENSES.map(entry => (
          <Collapsible key={entry.id} className='overflow-hidden rounded-md border border-border bg-surface'>
            <h3 className='m-0'>
              <CollapsibleTrigger className='group h-auto w-full justify-start gap-3 rounded-none px-4 py-3 text-left whitespace-normal'>
                <span className='min-w-0 flex-1'>
                  <span className='flex flex-wrap items-center gap-2'>
                    <span className='text-[13px] font-semibold'>{entry.name}</span>
                    <Badge variant='outline' shape='tag'>{entry.license}</Badge>
                  </span>
                  <span className='mt-1 block text-[11px] font-normal break-words text-subtle'>{entry.copyright}</span>
                </span>
                <svg viewBox='0 0 24 24' className='size-4 shrink-0 text-subtle transition-transform group-data-panel-open:rotate-180' aria-hidden='true'>
                  <path d='m6 9 6 6 6-6' fill='none' stroke='currentColor' strokeWidth='1.5' strokeLinecap='round' strokeLinejoin='round' />
                </svg>
              </CollapsibleTrigger>
            </h3>
            <CollapsibleContent className='border-t border-hairline p-4'>
              <dl className='space-y-2 text-xs leading-relaxed'>
                <div>
                  <dt className='text-subtle'>{t('Project source')}</dt>
                  <dd>
                    <a
                      href={entry.repository}
                      target='_blank'
                      rel='noopener noreferrer'
                      className='break-all text-primary underline underline-offset-2'
                      onClick={event => { event.preventDefault(); void openExternal(entry.repository) }}
                    >
                      {entry.repository}
                    </a>
                  </dd>
                </div>
                <div>
                  <dt className='text-subtle'>{entry.licenseRevision ? t('Revision used to verify the license') : t('Source revision')}</dt>
                  <dd className='font-mono break-all'>{entry.licenseRevision || entry.sourceRevision}</dd>
                </div>
              </dl>
              {entry.id === 'lobe-icons' ? (
                <p className='mt-2 text-xs leading-relaxed text-subtle'>
                  {t('These icons were obtained through Magpie. The Lobe Icons asset revision was not recorded separately; the revision above was used only to verify the license.')}
                </p>
              ) : null}
              <h4 className='mt-4 text-xs font-semibold'>{t('Original license text')}</h4>
              <pre
                tabIndex={0}
                role='region'
                aria-label={`${entry.name} ${t('Original license text')}`}
                lang='en'
                className='mt-2 max-h-[40vh] overflow-auto rounded-md border border-border bg-surface-inset p-3 font-mono text-[11px] leading-relaxed whitespace-pre-wrap break-words select-text'
              >{entry.text}</pre>
            </CollapsibleContent>
          </Collapsible>
        ))}
      </div>
      <p className='text-xs leading-relaxed text-subtle'>
        {t('This project\'s AGPL-3.0 license does not replace these third-party licenses. Trademark rights in names and logos belong to their respective owners.')}
      </p>
    </>
  )
}

export function ThirdPartyLicensesButton({ t, openExternal, mobile = false }: LicenseViewerProps) {
  const title = t('Third-party open-source licenses')
  const description = t('License texts are bundled with the app and available offline. Opening project source links requires an internet connection.')

  if (mobile) {
    return (
      <Sheet>
        <SheetTrigger render={<Button variant='outline' size='touch' className='w-full' />}>
          {title}
        </SheetTrigger>
        <SheetContent showHandle={false}>
          <SheetHeader><SheetTitle>{title}</SheetTitle></SheetHeader>
          <SheetBody>
            <SheetDescription className='text-xs leading-relaxed text-subtle'>{description}</SheetDescription>
            <LicenseList t={t} openExternal={openExternal} />
          </SheetBody>
          <SheetFooter>
            <SheetClose render={<Button variant='outline' size='touch' className='w-full' />}>
              {t('Close')}
            </SheetClose>
          </SheetFooter>
        </SheetContent>
      </Sheet>
    )
  }

  return (
    <Dialog>
      <DialogTrigger render={<Button variant='outline' size='xs' />}>
        {title}
      </DialogTrigger>
      <DialogContent className='w-[min(800px,calc(100vw-48px))]'>
        <DialogHeader className='shrink-0'><DialogTitle>{title}</DialogTitle></DialogHeader>
        <DialogBody className='min-h-0 [&>*]:shrink-0'>
          <DialogDescription className='text-xs leading-relaxed text-subtle'>{description}</DialogDescription>
          <LicenseList t={t} openExternal={openExternal} />
        </DialogBody>
        <DialogFooter className='shrink-0'>
          <DialogClose render={<Button variant='outline' />}>{t('Close')}</DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
