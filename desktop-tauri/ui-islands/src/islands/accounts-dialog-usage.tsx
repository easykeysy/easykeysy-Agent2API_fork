/**
 * set 「set」：**** 。
 *
 * file Reason：accounts-dialogs.tsx already file， this 1 is 
 * can （ + + ）， will 。
 *
 * ── already ， is ────────────────────────
 * 「 when 」delete after ， by to 
 * （`core::usage_query` ）， is that 1 config。
 * PATCH /api/accounts `usageQuery` （ after `apply_patch` 1 ），
 * （seconds / minutes / when ） is ， 1 is **seconds**。
 *
 * ── 「 not 」 already ──────────────────────────────────
 * this 「 not when + 」 「limit」
 * （ / Token then ， accounts-dialog-limiter.tsx and after 
 * `core::limiter`）—— is limit then ，config not in 。
 */

import * as React from 'react'
import {
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Field,
  FieldGroup,
  FieldLabel,
} from '@ui'
import { t } from '../i18n'
import { type AccountRecord } from './accounts-shared'
import { usageQueryOf, USAGE_QUERY_MAX_SECONDS, USAGE_QUERY_MIN_SECONDS } from './accounts-domain'

/** （； 1 seconds） */
type IntervalUnit = 'seconds' | 'minutes' | 'hours'

const UNIT_MS: Record<IntervalUnit, number> = { seconds: 1, minutes: 60, hours: 3600 }

/** 「set」 ： in （ in as empty / ） */
export type UsageDraft = {
  enabled: boolean
  intervalInput: string
  unit: IntervalUnit
}

/** seconds「 + can 」（900 seconds → 15 minutes）；0 → 5 minutes */
function splitInterval(seconds: number): { value: string; unit: IntervalUnit } {
  if (seconds > 0 && seconds % 3600 === 0) return { value: String(seconds / 3600), unit: 'hours' }
  if (seconds > 0 && seconds % 60 === 0) return { value: String(seconds / 60), unit: 'minutes' }
  if (seconds > 0) return { value: String(seconds), unit: 'seconds' }
  return { value: '5', unit: 'minutes' }
}

/** has config → （：、1 minutes —— accounts-domain） */
export function usageDraftOf(account: AccountRecord | null | undefined): UsageDraft {
  const { enabled, interval } = usageQueryOf(account)
  // when to 5 minutes ： new when not use for empty
  const intervalDraft = splitInterval(enabled ? interval : 0)
  return {
    enabled,
    intervalInput: intervalDraft.value,
    unit: intervalDraft.unit,
  }
}

export type UsageChanges =
  | { error: string }
  | { patch: { usageQuery?: { enabled: boolean; interval: number } } }

/** config；set not and 1 。 */
export function readUsageDraft(draft: UsageDraft):
  | { error: string }
  | { value: { enabled: boolean; interval: number } } {
  if (!draft.enabled) return { value: { enabled: false, interval: 0 } }
  const input = Number(draft.intervalInput)
  const seconds = Math.round(input * UNIT_MS[draft.unit])
  if (!Number.isFinite(input) || input <= 0) return { error: t('请填写查询间隔') }
  if (seconds < USAGE_QUERY_MIN_SECONDS || seconds > USAGE_QUERY_MAX_SECONDS) {
    return { error: t('查询间隔必须是 {min} 秒 ~ 24 小时', { min: USAGE_QUERY_MIN_SECONDS }) }
  }
  return { value: { enabled: true, interval: seconds } }
}

/** config not ，returnerror。 */
export function readUsageChanges(account: AccountRecord, draft: UsageDraft): UsageChanges | null {
  const result = readUsageDraft(draft)
  if ('error' in result) return result
  const seconds = result.value.interval
  const patch: { usageQuery?: { enabled: boolean; interval: number } } = {}
  const current = usageQueryOf(account)
  // in when ： down intervalInput is when old ，
  // not （otherwisenot configured 1 will empty 1 ）
  if (draft.enabled !== current.enabled || (draft.enabled && seconds !== current.interval)) {
    patch.usageQuery = { enabled: draft.enabled, interval: draft.enabled ? seconds : current.interval }
  }
  if (!patch.usageQuery) return null
  return { patch }
}

/** 「set」（： in ， when by readUsageChanges total） */
export function UsageSettingsSection({
  draft, onChange, idPrefix = 'account-usage',
}: {
  draft: UsageDraft
  onChange: (next: UsageDraft) => void
  idPrefix?: string
}) {
  const patch = (next: Partial<UsageDraft>): void => onChange({ ...draft, ...next })
  return (
    <>
      <Field orientation='horizontal' className='field-row mt-2.5 acct-usage-toggle'>
        <FieldLabel className='inline-flex cursor-pointer items-center gap-2.5 font-normal'>
          <Switch checked={draft.enabled} onCheckedChange={next => patch({ enabled: next === true })}
            aria-label={t('自动查询余额')} />
          <span className='text-xs text-subtle'>{t('自动查询余额（按下面的间隔自动刷新余额列的读数）')}</span>
        </FieldLabel>
      </Field>
      {draft.enabled ? (
        <FieldGroup orientation='horizontal' className='field-row mt-2.5 acct-usage-interval'>
          <Field className='contents'>
            <FieldLabel htmlFor={`${idPrefix}-interval`}>{t('查询间隔')}</FieldLabel>
            <Input id={`${idPrefix}-interval`} type='number' min={1} step={1}
              className='max-w-[110px]' value={draft.intervalInput}
              onChange={event => patch({ intervalInput: event.currentTarget.value })} />
          </Field>
          <Select value={draft.unit} onValueChange={value => patch({ unit: value as IntervalUnit })}>
            <SelectTrigger className='w-[92px]' aria-label={t('间隔单位')}>
              <SelectValue>{draft.unit === 'hours' ? t('小时') : draft.unit === 'minutes' ? t('分钟') : t('秒')}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='seconds'>{t('秒')}</SelectItem>
              <SelectItem value='minutes'>{t('分钟')}</SelectItem>
              <SelectItem value='hours'>{t('小时')}</SelectItem>
            </SelectContent>
          </Select>
          <span className='detail'>{t('30 秒 ~ 24 小时；手动查询会顺延下一轮')}</span>
        </FieldGroup>
      ) : null}
    </>
  )
}
