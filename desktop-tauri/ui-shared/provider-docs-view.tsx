import { Badge, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@ui'
import { t } from '../ui-kit/src/i18n'
import { documentText, PROVIDER_CAPABILITY_KEYS, type PluginProviderDocumentation,
  type ProviderCapabilityKey, type ProviderCapabilityStatus, type ProviderDocumentation } from './provider-docs'

function columns(): Record<ProviderCapabilityKey, string> {
  return {
    chat: t('LLLM requests'),
    refresh: t('Automatic token renewal'),
    model_refresh: t('Online model list'),
    usage: t('Balance query'),
    checkin: t('Check in'),
    claims: t('Reward claims'),
  }
}

function statusLabel(status: ProviderCapabilityStatus): string {
  switch (status) {
    case 'supported': return t('Supported')
    case 'implemented_unverified': return t('Implemented; pending live testing')
    case 'unsupported': return t('Not supported')
    case 'not_applicable': return t('Not applicable')
    default: return t('Not declared')
  }
}

function Status({ status }: { status: ProviderCapabilityStatus }) {
  const symbols: Record<ProviderCapabilityStatus, string> = {
    supported: '✓', implemented_unverified: t('Pending live testing'), unsupported: '✗', not_applicable: '—', undeclared: '?',
  }
  return <span aria-label={statusLabel(status)} title={statusLabel(status)}
    className={status === 'supported' ? 'text-success font-semibold' : 'text-muted-foreground'}>
    {symbols[status] || '?'}
  </span>
}

function rowNotes(provider: ProviderDocumentation, locale: string, labels: Record<ProviderCapabilityKey, string>): string[] {
  const notes = provider.notes.map(note => documentText(note, locale)).filter(Boolean)
  for (const key of PROVIDER_CAPABILITY_KEYS) {
    const note = documentText(provider.capabilities[key]?.note, locale)
    if (note) notes.push(`${labels[key]}：${note}`)
  }
  return [...new Set(notes)]
}

function packTitle(pack: PluginProviderDocumentation): string {
  return pack.id === 'builtin-subscriptions' ? t('Native subscriptions') : t(pack.name)
}

export function ProviderDocsTable({ packs }: { packs: PluginProviderDocumentation[] }) {
  const locale = (window as unknown as { wbI18n?: { locale(): string } }).wbI18n?.locale() || 'zh-Hans'
  const labels = columns()
  return <div className='flex min-w-0 flex-col gap-3'>
    {packs.map(pack => <Card key={pack.id}>
      <CardHeader>
        <CardTitle>{packTitle(pack)}</CardTitle>
        <Badge variant='outline'>v{pack.version}</Badge>
        {pack.description ? <CardDescription className='w-full'>
          {documentText(pack.description, locale)}
        </CardDescription> : null}
      </CardHeader>
      <CardContent className='p-0'>
        <Table className='min-w-[860px]' aria-label={`${packTitle(pack)} ${t('Provider capability matrix')}`}>
          <TableHeader><TableRow>
            <TableHead className='min-w-[170px]'>{t('Provider')}</TableHead>
            {PROVIDER_CAPABILITY_KEYS.map(key => <TableHead key={key} className='text-center'>
              {labels[key]}
            </TableHead>)}
            <TableHead className='min-w-[250px]'>{t('Note')}</TableHead>
          </TableRow></TableHeader>
          <TableBody>{pack.providers.map(provider => {
            const notes = rowNotes(provider, locale, labels)
            return <TableRow key={provider.id}>
              <TableCell>
                <div className='flex flex-col items-start gap-1'>
                  <span className='font-medium'>{t(provider.label)}</span>
                  {!provider.enabled ? <Badge variant='outline'>{t('Disabled')}</Badge>
                    : !provider.loaded ? <Badge variant='outline'>{t('Not loaded')}</Badge> : null}
                </div>
              </TableCell>
              {PROVIDER_CAPABILITY_KEYS.map(key => <TableCell key={key} className='text-center'>
                <Status status={provider.capabilities[key]?.status || 'undeclared'} />
              </TableCell>)}
              <TableCell className='max-w-[380px] text-[11.5px] leading-relaxed text-subtle'>
                {notes.length ? <div className='flex flex-col gap-1'>
                  {notes.map(note => <span key={note}>{note}</span>)}
                </div> : '—'}
              </TableCell>
            </TableRow>
          })}</TableBody>
        </Table>
      </CardContent>
      <CardFooter className='flex-col items-start gap-1'>
        <span>{t('✓ Supported · ✗ Unsupported · — Not applicable · ? Undeclared')}</span>
        <span>{t('Implemented; pending live testing: the code is integrated, but has not been validated with a real account.')}</span>
        <span>{t('Online model lists mean the upstream catalog can be fetched automatically; a static list alone does not count as online support.')}</span>
        <span>{t('Undeclared means the gateway or plugin has not provided a definite conclusion; it does not mean unsupported.')}</span>
      </CardFooter>
    </Card>)}
  </div>
}
