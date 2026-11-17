import * as React from 'react'
import { mountOnPage } from '../island-lazy'
import { createRoot } from 'react-dom/client'
import { Button, Card, CardHeader, CardFooter } from '@ui'
import { apiEndpoints } from '../../../ui-shared/docs-api-endpoints'
import { DocumentationBrowser } from '../../../ui-shared/docs-api-browser'
import { useProviderDocumentation } from './docs-provider-data'
import { useGatewayBase } from './port-panel'
import { t } from '../i18n'

// 地址节点由 port-panel 写入；id 与 clipboard.js 的 data-copy-from 契约必须保持稳定。
function EndpointRow({ id, name, note }: { id: string; name: string; note: string }) {
  return <div className='ep-row'>
    <div className='ep-head'>
      <span className='ep-name'>{name}</span>
      <span className='ep-note'>{note}</span>
    </div>
    <div className='ep-line'>
      <code className='ep-value' id={id}>—</code>
      <Button variant='ghost' size='icon-xs' className='copy-btn size-[21px] rounded-[var(--r-xs)]'
        data-copy-from={id} title={t('Copy address')} aria-label={t('Copy address')}>⧉</Button>
    </div>
  </div>
}

async function copyExample(text: string, label: string) {
  const shared = window as unknown as {
    copyToClipboard?: (text: string) => Promise<boolean>
    wbApp?: { toast?: (message: string, kind?: 'err' | 'ok') => void }
  }
  try {
    if (shared.copyToClipboard) {
      if (!(await shared.copyToClipboard(text))) throw new Error()
    } else await navigator.clipboard.writeText(text)
    shared.wbApp?.toast?.(t('Copied: {text}', { text: label }), 'ok')
  } catch {
    shared.wbApp?.toast?.(t('Copy failed, please select and copy manually'), 'err')
  }
}

function ApiAddresses({ origin }: { origin: string }) {
  const endpoints = apiEndpoints()

  React.useLayoutEffect(() => {
    window.wbPortPanel?.render()
  }, [origin])

  return <Card render={<section />} className='panel ep-panel docs-addresses'>
    <CardHeader className='panel-head'><h3>{t('API address')}</h3></CardHeader>
    <div className='ep-list'>
      {endpoints.map((endpoint, index) => <React.Fragment key={endpoint.id}>
        {endpoint.group && endpoint.group !== endpoints[index - 1]?.group
          ? <div className='ep-group'>{endpoint.group}</div> : null}
        <EndpointRow id={endpoint.id} name={endpoint.name} note={endpoint.note} />
      </React.Fragment>)}
    </div>
    <CardFooter className='panel-foot flex-col items-start gap-1.5'>
      <span>{t('Generation, editing and chat endpoints share gateway Keys. Find authentication rules, model capabilities and request examples in the documentation contents.')}</span>
      <span>{t('Root address for the official Anthropic SDK / Claude Code: ')}<code>{origin}</code></span>
    </CardFooter>
  </Card>
}

function DocsPage() {
  const origin = useGatewayBase()
  const { packs, error, loading, load } = useProviderDocumentation()

  return <DocumentationBrowser origin={origin} packs={packs} addresses={<ApiAddresses origin={origin} />}
    onCopy={copyExample} notice={error ? <div role='status' className='mb-3 flex flex-wrap items-center gap-2 text-[12px] text-subtle'>
      <span>{t('Failed to load provider capabilities: {error}', { error })}</span>
      <Button variant='outline' size='xs' disabled={loading} onClick={() => void load()}>{t('Retry')}</Button>
    </div> : null} />
}

const PAGE_SELECTOR = '.page[data-page="docs"]'
let mounted = false

function mount(): void {
  if (mounted) return
  const section = document.querySelector<HTMLElement>(PAGE_SELECTOR)
  if (!section) return
  mounted = true
  section.replaceChildren()
  createRoot(section).render(<DocsPage />)
}

mountOnPage('docs', PAGE_SELECTOR, mount)
