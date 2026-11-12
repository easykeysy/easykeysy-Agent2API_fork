/**
 * Agent2API · 「」 → 「」： from its 。
 *
 * old ui/add-provider-import.js。current has **cc-switch**（ after 
 * `GET /api/import/cc-switch` SQLite， core::import_ccswitch），
 * new-api / sub2api —— when in scan up and can ，
 * 「 → → create」 not use 。
 *
 * ── Why this 1 not 2 ────────────────────────────
 * 1 for 1 「 new + 」， no has need need use 
 * （ / / Base URL / Key config），。
 * is 1 that ， by **** that 。
 *
 * ── Why in ────────────────────────────────────
 * old （ / ） is ， not 、。
 * up use 1 to ： in this 、 use useSyncExternalStore ， is 
 * 「 to 」 and 「」 old as 。
 *
 * ： this current in up （IMPORT_SEGMENT_ENABLED = false，
 * add-provider-pick.tsx），， that available。
 */

import * as React from 'react'
import { Button, Checkbox } from '@ui'

import { shared, toast } from './add-account-bridge'
import { t } from '../i18n'
import { Badge } from '@ui'
import { Empty } from '@ui'

type ImportProvider = {
  id: string | number
  name?: string
  protocol?: string
  baseUrl?: string
  apiKey?: string
  models?: unknown[]
  modelMappings?: Array<{ alias?: string; target?: string }>
  importable?: boolean
  reason?: string
  notes?: string
}

type ImportScan = {
  available?: boolean
  reason?: string
  path?: string
  providers?: ImportProvider[]
}

type ImportState = {
  /** '' = no （ when ）；'loading'；'done' */
  scanState: '' | 'loading' | 'done'
  scan: ImportScan | null
  /** （ id； has importable will in ） */
  checked: Set<string>
  /** in ： and its 1 （， not ） */
  busy: boolean
  /** failure Reason（ old that 1 DOM， this in ） */
  failures: Map<string, string>
  /** up failure（toast seconds after no ， this 1 ） */
  footHint: string
}

let state: ImportState = {
  scanState: '',
  scan: null,
  checked: new Set(),
  busy: false,
  failures: new Map(),
  footHint: '',
}

const listeners = new Set<() => void>()

function patch(next: Partial<ImportState>): void {
  state = { ...state, ...next }
  for (const listener of listeners) listener()
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener)
  return () => { listeners.delete(listener) }
}

function useImportState(): ImportState {
  return React.useSyncExternalStore(subscribe, () => state)
}

const request = (method: string, path: string, body?: unknown): Promise<unknown> | undefined =>
  shared().wbProviders?.customRequest?.(method, path, body)

/** cc-switch（failure in to Reason， not toast —— empty is ） */
async function runScan(options: { force?: boolean } = {}): Promise<void> {
  const force = options.force === true
  if (state.busy) return
  if (state.scanState === 'loading' || (state.scanState === 'done' && !force)) return
  patch({ scanState: 'loading', failures: new Map() })
  let scan: ImportScan
  try {
    const data = (await request('GET', '/api/import/cc-switch')) as ImportScan | null
    scan = data || { available: false, reason: t('响应为空') }
  } catch (error) {
    scan = {
      available: false,
      reason: error instanceof Error ? error.message : String(error),
    }
  }
  // default can （ after use already ）
  const checked = new Set(state.checked)
  if (!checked.size) {
    for (const item of scan.providers || []) {
      if (item.importable === true) checked.add(String(item.id))
    }
  }
  patch({ scanState: 'done', scan, checked })
}

/** （ up ） */
function protocolLabel(protocol?: string): string {
  if (protocol === 'anthropic') return 'Anthropic'
  if (protocol === 'responses') return 'Responses'
  return String(protocol || '')
}

/** that 「 …」： will ， 「 etc N 」 */
function modelNote(item: ImportProvider): string {
  const models = (Array.isArray(item?.models) ? item.models : [])
    .map(entry => String(entry || '').trim())
    .filter(Boolean)
  if (!models.length) return ''
  const head = models.slice(0, 2).join(t('、'))
  const rest = models.length > 2 ? t(' 等 {n} 个', { n: models.length }) : ''
  return t(' · 模型 {head}{rest}', { head, rest })
}

/**
 * cc-switch config new ****（ `[1M]` ）。
 *
 * Whymust： is by before —— no will 
 * （custom_providers::bindings::resolve in models / mappings ）。cc-switch 
 * claude has ANTHROPIC_MODEL and 、codex has model， is use when in use 
 * ， new can use 。
 *
 * failure** not already **： and already ， can ；
 * this failure 1 ， +1。
 */
async function saveModels(created: unknown, item: ImportProvider): Promise<void> {
  const providerId = (created as { provider?: { id?: string } } | null)?.provider?.id
  const models = (Array.isArray(item?.models) ? item.models : [])
    .map(entry => String(entry || '').trim())
    .filter(Boolean)
    .map(id => ({ id, enabled: true, reasoning: '' }))
  if (!providerId || !models.length) return
  const mappings = (Array.isArray(item?.modelMappings) ? item.modelMappings : [])
    .map(pair => ({
      alias: String(pair?.alias || '').trim(),
      target: String(pair?.target || '').trim(),
    }))
    .filter(pair => pair.alias && pair.target
      && pair.alias.toLowerCase() !== pair.target.toLowerCase())
    .map(pair => ({ ...pair, enabled: true, reasoning: '' }))
  try {
    await request('POST', '/api/custom-providers/models', { providerId, models, mappings })
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    console.warn('导入时写入初始模型清单失败:', message)
  }
}

/**
 * ： cc-switch configcreate + 
 * （ use POST /api/custom-providers， 1 request）。、：
 * failure not its （failure Reason in can ），success。
 */
async function runImport(): Promise<void> {
  const scan = state.scan
  const selected = (Array.isArray(scan?.providers) ? scan.providers : [])
    .filter(item => item.importable === true && state.checked.has(String(item.id)))
  if (!selected.length || state.busy) return
  const failures = new Map(state.failures)
  const checked = new Set(state.checked)
  patch({ busy: true, footHint: '', failures })
  let ok = 0
  for (const item of selected) {
    const payload = {
      name: String(item.name || ''),
      protocol: String(item.protocol || 'chat_completions'),
      baseUrl: String(item.baseUrl || ''),
      apiKey: String(item.apiKey || ''),
    }
    try {
      const created = await request('POST', '/api/custom-providers', payload)
      await saveModels(created, item)
      checked.delete(String(item.id))
      ok += 1
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      failures.set(String(item.id), message)
    }
  }
  // has success ： down 1 ， will after 
  // 「」 not is 1 「 can 」—— otherwise use 1 is create。
  patch({
    busy: false,
    checked,
    failures,
    scanState: ok > 0 ? '' : state.scanState,
  })
  if (ok > 0) {
    // directory and need ： / / 
    void shared().wbProviders?.refreshCustom?.()
    await shared().wbApp?.refresh?.()
  }
  const failedCount = selected.length - ok
  if (ok > 0 && !failedCount) {
    shared().wbAddAccountModal?.close?.()
    toast(t('✅ 已从 cc-switch 导入 {n} 个供应商', { n: ok }))
  } else if (ok > 0) {
    toast(t('已导入 {ok} 个，失败 {fail} 个（失败项已标注在列表里）', { ok, fail: failedCount }), 'err')
  } else {
    const first = [...failures.values()][0]
    patch({ footHint: first || t('导入失败') })
  }
}

/* ─── （ 1 ）────────────────── */

/** by by （ old is setSegment）， */
export function ImportPanel({ segmentOn }: { segmentOn: boolean }): React.ReactElement {
  const current = useImportState()
  const style: React.CSSProperties = segmentOn ? {} : { display: 'none' }

  React.useEffect(() => {
    if (segmentOn) void runScan()
  }, [segmentOn])

  if (current.scanState === 'loading') {
    return (
      <div id='add-import-panel' style={style}>
        <div id='add-import-status'>{t('正在扫描本机 cc-switch…')}</div>
        <div id='add-import-list' />
      </div>
    )
  }

  const scan = current.scan
  const providers = Array.isArray(scan?.providers) ? scan.providers : []
  const importable = providers.filter(item => item.importable === true)

  return (
    <div id='add-import-panel' style={style}>
      <div id='add-import-status'>
        {!scan || !scan.available ? (
          <Empty className='add-import-empty'>
            <p>{scan?.reason || t('未能读取 cc-switch 数据')}</p>
            <Button variant='outline' size='sm' onClick={() => { void runScan({ force: true }) }}>
              {t('重新扫描')}
            </Button>
          </Empty>
        ) : (
          <div className='add-import-head'>
            <span>
              {t('在 cc-switch 里找到')} <b>{providers.length}</b> {t('条配置，其中')}{' '}
              <b>{importable.length}</b> {t('条可导入')}
            </span>
            <Button variant='ghost' size='xs' className='linkish' onClick={() => { void runScan({ force: true }) }}>
              {t('重新扫描')}
            </Button>
          </div>
        )}
      </div>
      <div id='add-import-list'>
        {providers.map(item => {
          const id = String(item.id)
          const canImport = item.importable === true
          const isChecked = current.checked.has(id)
          const failed = current.failures.get(id)
          // 1 and ： will new ， use 「」
          const meta = failed
            ? t('导入失败：{reason}', { reason: failed })
            : canImport
              ? String(item.baseUrl || '') + modelNote(item)
              : String(item.reason || t('不支持导入'))
          const toggle = (): void => {
            const checked = new Set(state.checked)
            if (checked.has(id)) checked.delete(id)
            else checked.add(id)
            patch({ checked })
          }
          return (
            // can （ old is <label> checkbox）：Checkbox is Base UI 
            // button， label will 「label can 」 ，
            // div + ， and in Checkbox up 1 
            <div
              key={id}
              className={`add-import-row${canImport && !failed ? '' : ' disabled'}`}
              title={canImport ? String(item.notes || '') : String(item.reason || '')}
              onClick={() => { if (canImport) toggle() }}
            >
              {/* failure （ old to .disabled and failure）：
                   in ， 1 「」 is */}
              {canImport ? (
                <Checkbox
                  checked={isChecked}
                  aria-label={String(item.name || id)}
                  onClick={event => event.stopPropagation()}
                  onCheckedChange={toggle}
                />
              ) : (
                <span className='add-import-dash' />
              )}
              <span className='add-import-info'>
                <span className='add-import-name'>
                  {String(item.name || id)}
                  {canImport ? (
                    <Badge variant='outline' shape='tag' className='add-import-badge'>{protocolLabel(item.protocol)}</Badge>
                  ) : null}
                </span>
                <span className='add-import-meta'>{meta}</span>
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}

/* ─── up 「」（ that ）─────── */

/** can / （）/ use and failure in this ， and old syncFoot 1 */
export function ImportFootActions(): React.ReactElement {
  const current = useImportState()
  const count = current.checked.size
  return (
    <>
      <span className={`add-foot-hint${current.footHint ? ' err' : ''}`} id='add-foot-hint'>
        {current.footHint}
      </span>
      <span className='add-foot-actions' id='add-foot-actions'>
        <Button
          id='import-foot-button'
          disabled={current.busy || !count}
          onClick={() => { void runImport() }}
        >
          {current.busy ? t('导入中…') : count ? t('导入所选（{n}）', { n: count }) : t('导入所选')}
        </Button>
      </span>
    </>
  )
}
