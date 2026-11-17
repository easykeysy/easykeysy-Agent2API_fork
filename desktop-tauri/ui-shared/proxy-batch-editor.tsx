import * as React from 'react'
import { Button, Checkbox, Input, Label, Switch, Table, TableBody, TableCell, TableHead, TableHeader, TableRow, Textarea } from '@ui'
import { parseProxyList, proxyAddress, type ProxyImportDraft, type ProxyImportIssue, type ProxyImportReport } from './proxy-import'

type Translate = (key: string, values?: Record<string, string | number>) => string
type TestResult = { success?: boolean; error?: string; ip?: string; durationMs?: unknown }
type BatchRow = { key: number; draft: ProxyImportDraft; selected: boolean; testing?: boolean; result?: TestResult; error?: string }
const PAGE_SIZE = 20

export type ProxyBatchHandle = { hasDraft: () => boolean }

export const ProxyBatchEditor = React.forwardRef<ProxyBatchHandle, {
  t: Translate
  locked: boolean
  onBusyChange: (busy: boolean) => void
  save: (items: ProxyImportDraft[]) => Promise<ProxyImportReport>
  test: (item: ProxyImportDraft) => Promise<TestResult>
  onNotice: (message: string, failed?: boolean) => void
}>(({ t, locked, onBusyChange, save, test, onNotice }, ref) => {
  const [input, setInput] = React.useState('')
  const [rows, setRows] = React.useState<BatchRow[]>([])
  const [issues, setIssues] = React.useState<ProxyImportIssue[]>([])
  const [nonDirect, setNonDirect] = React.useState(false)
  const [busy, setBusy] = React.useState<'save' | 'test' | ''>('')
  const [error, setError] = React.useState('')
  const [summary, setSummary] = React.useState('')
  const [page, setPage] = React.useState(1)
  const [progress, setProgress] = React.useState({ done: 0, total: 0 })
  const nextKey = React.useRef(0)
  const busyRef = React.useRef(false)
  const alive = React.useRef(true)
  const current = React.useRef({ rows, input })
  current.current = { rows, input }
  React.useEffect(() => { alive.current = true; return () => { alive.current = false } }, [])
  React.useImperativeHandle(ref, () => ({ hasDraft: () => Boolean(current.current.rows.length || current.current.input.trim()) }), [])
  const disabled = locked || Boolean(busy)
  const selected = rows.filter(row => row.selected)
  const pages = Math.max(1, Math.ceil(rows.length / PAGE_SIZE))
  const currentPage = Math.min(page, pages)
  const pageRows = rows.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE)

  function parse(): void {
    if (disabled) return
    const result = parseProxyList(input, nonDirect, rows.map(row => row.draft), t)
    setRows(previous => [...previous, ...result.items.map(draft => ({ key: ++nextKey.current, draft, selected: true }))])
    setInput(result.remaining)
    setIssues(result.issues)
    setError('')
    setPage(1)
    onNotice(t('Parsed {n} proxies and skipped {duplicates} duplicates', { n: result.items.length, duplicates: result.duplicates }), result.issues.length > 0)
  }

  function update(key: number, change: Partial<BatchRow>): void {
    setRows(previous => previous.map(row => row.key === key ? { ...row, ...change } : row))
  }

  async function act(mode: 'save' | 'test'): Promise<void> {
    if (disabled || busyRef.current || !selected.length) return
    if (mode === 'save' && selected.some(row => !row.draft.name.trim())) { setError(t('Enter a proxy name')); return }
    busyRef.current = true
    setBusy(mode)
    onBusyChange(true)
    setError('')
    const targets = selected.map(row => ({ ...row, draft: { ...row.draft, name: row.draft.name.trim() } }))
    setProgress({ done: 0, total: targets.length })
    try {
      if (mode === 'save') {
        const result = await save(targets.map(row => row.draft))
        if (!alive.current) return
        const finished = new Set([...result.added.map(item => item.index), ...result.duplicates].map(index => targets[index]?.key))
        const errors = new Map(result.errors.map(item => [targets[item.index]?.key, item.error]))
        setRows(previous => previous.filter(row => !finished.has(row.key)).map(row => ({ ...row, error: errors.get(row.key) })))
        const message = t('Batch complete: {added} added, {duplicates} duplicates, {failed} failed', {
          added: result.added.length, duplicates: result.duplicates.length, failed: result.errors.length,
        })
        setSummary(message)
        onNotice(message, result.errors.length > 0)
      } else {
        let ok = 0
        for (const [index, row] of targets.entries()) {
          if (!alive.current) break
          update(row.key, { testing: true, result: undefined, error: undefined })
          let result: TestResult
          try { result = await test(row.draft) }
          catch (failure) { result = { success: false, error: failure instanceof Error ? failure.message : String(failure) } }
          if (!alive.current) break
          if (result.success) ok += 1
          update(row.key, { testing: false, result })
          setProgress({ done: index + 1, total: targets.length })
        }
        if (alive.current) onNotice(t('Batch test complete: {ok} available, {failed} failed ({total} total)', {
          ok, failed: targets.length - ok, total: targets.length,
        }), ok !== targets.length)
      }
    } catch (failure) {
      if (alive.current) setError(failure instanceof Error ? failure.message : String(failure))
    } finally {
      busyRef.current = false
      if (alive.current) { setBusy(''); onBusyChange(false) }
    }
  }

  return <section className='flex min-w-0 flex-col gap-3' aria-label={t('Add in bulk')}>
    <div>
      <h3 className='text-sm font-semibold'>{t('Add in bulk')}</h3>
      <p className='mt-1 text-xs leading-relaxed text-subtle'>{t('Enter one proxy per line. Parse the list, edit names, choose chained connections and test, then save the selected proxies.')}</p>
    </div>
    <Textarea value={input} rows={5} disabled={disabled} spellCheck={false} autoComplete='off'
      className='resize-y font-mono text-xs' aria-label={t('Proxy list')}
      onChange={event => { setInput(event.currentTarget.value); setIssues([]) }}
      placeholder={'192.0.2.1:8080\n192.0.2.1:8080:user:password\nsocks5://user:password@proxy.example:1080\nhttp://[2001:db8::1]:8080{' + t('Name') + '}'} />
    <Label className='flex items-center gap-2 text-xs font-normal'>
      <Checkbox checked={nonDirect} disabled={disabled} onCheckedChange={setNonDirect} />{t('Use chained connections for newly parsed proxies')}
    </Label>
    <p className='text-xs leading-relaxed text-subtle'>{t('Connect through the gateway device\'s system proxy first, then through this proxy. If the system proxy is unavailable, requests will not fall back to a direct connection.')}</p>
    <div className='flex flex-wrap items-center gap-2'>
      <Button size='sm' variant='outline' disabled={disabled || !input.trim()} onClick={parse}>{t('Parse')}</Button>
      <Button size='sm' variant='ghost' disabled={disabled || !rows.length} onClick={() => { setRows([]); setPage(1); setError('') }}>{t('Clear parsed list')}</Button>
      <span className='ml-auto text-xs text-subtle'>{t('{n} parsed · {selected} selected', { n: rows.length, selected: selected.length })}</span>
    </div>
    {issues.length ? <div className='max-h-24 overflow-auto text-xs text-destructive' role='alert'>
      <p>{t('These lines could not be parsed and remain in the input. Correct them and parse again.')}</p>
      {issues.map(issue => <p key={issue.line}>{t('Line {line}: {error}', { line: issue.line, error: issue.error })}</p>)}
    </div> : null}
    <Table containerClassName='max-h-[320px] rounded-md border border-border' className='min-w-[560px]'>
      <TableHeader><TableRow>
        <TableHead className='w-8'><Checkbox checked={rows.length > 0 && selected.length === rows.length}
          indeterminate={selected.length > 0 && selected.length < rows.length} disabled={disabled || !rows.length}
          aria-label={t('Select all proxies')} onCheckedChange={value => setRows(previous => previous.map(row => ({ ...row, selected: value })))} /></TableHead>
        <TableHead>{t('Proxy name')}</TableHead><TableHead>{t('Address')}</TableHead>
        <TableHead className='text-center'>{t('Chained')}</TableHead><TableHead>{t('Test')}</TableHead><TableHead>{t('Actions')}</TableHead>
      </TableRow></TableHeader>
      <TableBody>{pageRows.length ? pageRows.map(row => <TableRow key={row.key}>
        <TableCell><Checkbox checked={row.selected} disabled={disabled} aria-label={t('Select "{name}"', { name: row.draft.name })}
          onCheckedChange={value => update(row.key, { selected: value })} /></TableCell>
        <TableCell><Input maxLength={60} value={row.draft.name} disabled={disabled} className='h-7 min-w-24 text-xs'
          aria-label={t('Proxy name')} onChange={event => update(row.key, { draft: { ...row.draft, name: event.currentTarget.value }, error: undefined })} /></TableCell>
        <TableCell><span className='block max-w-44 truncate font-mono text-[11px]' title={proxyAddress(row.draft)}>{proxyAddress(row.draft)}</span>
          {row.draft.username || row.draft.password ? <span className='text-[10px] text-subtle'>{t('Authentication configured')}</span> : null}</TableCell>
        <TableCell className='text-center'><Switch checked={row.draft.nonDirect} disabled={disabled} aria-label={t('Chain through system proxy')}
          onCheckedChange={value => update(row.key, { draft: { ...row.draft, nonDirect: value }, result: undefined, error: undefined })} /></TableCell>
        <TableCell><span className={`block max-w-36 truncate text-xs ${row.error || row.result?.success === false ? 'text-destructive' : 'text-subtle'}`}
          title={row.error || row.result?.error || row.result?.ip}>
          {row.error || (row.testing ? t('Testing…') : row.result ? row.result.success ? row.result.ip || t('usable') : row.result.error || t('Failed') : t('Not tested'))}
        </span></TableCell>
        <TableCell><Button size='xs' variant='ghost' disabled={disabled} onClick={() => setRows(previous => previous.filter(item => item.key !== row.key))}>{t('Remove')}</Button></TableCell>
      </TableRow>) : <TableRow><TableCell colSpan={6} className='py-8 text-center text-xs text-subtle'>{t('Paste a proxy list, then click Parse')}</TableCell></TableRow>}</TableBody>
    </Table>
    {pages > 1 ? <div className='flex items-center justify-end gap-2 text-xs'>
      <Button size='xs' variant='outline' disabled={currentPage === 1} onClick={() => setPage(currentPage - 1)}>{t('Previous')}</Button>
      <span>{currentPage} / {pages}</span>
      <Button size='xs' variant='outline' disabled={currentPage === pages} onClick={() => setPage(currentPage + 1)}>{t('Next')}</Button>
    </div> : null}
    {summary ? <p className='text-xs text-subtle' role='status'>{summary}</p> : null}
    {error ? <p className='text-xs text-destructive' role='alert'>{error}</p> : null}
    <div className='mt-auto flex flex-wrap justify-end gap-2 pt-1'>
      <Button variant='outline' disabled={disabled || !selected.length} onClick={() => void act('test')}>
        {busy === 'test' ? t('Testing {done}/{total}', progress) : t('Test selected ({n})', { n: selected.length })}
      </Button>
      <Button disabled={disabled || !selected.length} onClick={() => void act('save')}>
        {busy === 'save' ? t('Saving…') : t('Add selected ({n})', { n: selected.length })}
      </Button>
    </div>
  </section>
})
