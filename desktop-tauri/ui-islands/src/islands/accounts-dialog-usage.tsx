/**
 * 账号设置弹窗的「查询设置」段：每账号的自动余额查询间隔与失败重试。
 *
 * 独立成文件的原因：accounts-dialogs.tsx 已超过单文件行数约定，这一段是完整
 * 的功能块（状态草稿 + 校验 + 渲染），放进去只会继续膨胀。
 *
 * ── 全局任务已退役，间隔是每账号自己的 ────────────────────────
 * 全局「定时查询积分」任务删除后，余额查询按账号各自的间隔到期触发
 * （`core::usage_query` 的心跳循环），本段就是那个间隔的唯一配置入口。保存走
 * PATCH /api/accounts 的 `usageQuery` 键（后端 `apply_patch` 归一化校验），
 * 界面的单位选择（秒 / 分钟 / 小时）只是输入辅助，落库一律是**秒**。
 *
 * ── 「余额不足处理」已迁走 ──────────────────────────────────
 * 原先这里的「余额不足时 + 余额阈值」两行升级成了独立的「限制器」段
 * （余额 / Token 规则列表，见 accounts-dialog-limiter.tsx 与后端
 * `core::limiter`）—— 它是限制器的余额规则，配置入口不再在本段。
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
import {
  usageQueryOf, USAGE_QUERY_MAX_SECONDS, USAGE_QUERY_MIN_SECONDS, USAGE_QUERY_MAX_RETRIES, USAGE_QUERY_MAX_TIMEOUT, USAGE_QUERY_DEFAULT_TIMEOUT,
  type UsageQuerySettings,
} from '../../../ui-shared/usage-query-settings'

/** 间隔输入的单位（界面辅助；落库统一换算成秒） */
type IntervalUnit = 'seconds' | 'minutes' | 'hours'

const UNIT_MS: Record<IntervalUnit, number> = { seconds: 1, minutes: 60, hours: 3600 }

/** 「查询设置」段的草稿：数字输入框以字符串中转（输入中途允许为空 / 非法） */
export type UsageDraft = {
  enabled: boolean
  intervalInput: string
  unit: IntervalUnit
  retriesInput: string
  timeoutInput?: string
}

/** 把秒数拆成「数字 + 最大可整除的单位」（900 秒 → 15 分钟）；0 → 5 分钟缺省 */
function splitInterval(seconds: number): { value: string; unit: IntervalUnit } {
  if (seconds > 0 && seconds % 3600 === 0) return { value: String(seconds / 3600), unit: 'hours' }
  if (seconds > 0 && seconds % 60 === 0) return { value: String(seconds / 60), unit: 'minutes' }
  if (seconds > 0) return { value: String(seconds), unit: 'seconds' }
  return { value: '5', unit: 'minutes' }
}

/** 账号现有配置 → 草稿（读侧缺省：开启、1 分钟 —— 见 accounts-domain） */
export function usageDraftOf(account: AccountRecord | null | undefined): UsageDraft {
  const { enabled, interval, retries, timeoutSeconds } = usageQueryOf(account)
  // 关着的时候间隔输入给个 5 分钟的落点值：重新拨开时不用面对空输入框
  const intervalDraft = splitInterval(enabled ? interval : 0)
  return {
    enabled,
    intervalInput: intervalDraft.value,
    unit: intervalDraft.unit,
    retriesInput: String(retries),
    timeoutInput: String(timeoutSeconds || USAGE_QUERY_DEFAULT_TIMEOUT),
  }
}

export type UsageChanges =
  | { error: string }
  | { patch: { usageQuery?: UsageQuerySettings } }

/** 草稿转成完整查询配置；批量设置不与某一条账号的现值比较。 */
export function readUsageDraft(draft: UsageDraft):
  | { error: string }
  | { value: UsageQuerySettings } {
  const retries = Number(draft.retriesInput)
  if (!draft.retriesInput.trim() || !Number.isInteger(retries) || retries < 0 || retries > USAGE_QUERY_MAX_RETRIES) {
    return { error: t('Retry count must be an integer from 0 to {max}', { max: USAGE_QUERY_MAX_RETRIES }) }
  }
  const timeoutSeconds = Number(draft.timeoutInput ?? USAGE_QUERY_DEFAULT_TIMEOUT)
  if (!Number.isInteger(timeoutSeconds) || timeoutSeconds < 15 || timeoutSeconds > USAGE_QUERY_MAX_TIMEOUT) {
    return { error: t('The overall timeout must be an integer from 15 to 180 seconds.') }
  }
  if (!draft.enabled) return { value: { enabled: false, interval: 0, retries, timeoutSeconds } }
  const input = Number(draft.intervalInput)
  const seconds = Math.round(input * UNIT_MS[draft.unit])
  if (!Number.isFinite(input) || input <= 0) return { error: t('Enter the query interval') }
  if (seconds < USAGE_QUERY_MIN_SECONDS || seconds > USAGE_QUERY_MAX_SECONDS) {
    return { error: t('The query interval must be {min} seconds to 24 hours', { min: USAGE_QUERY_MIN_SECONDS }) }
  }
  return { value: { enabled: true, interval: seconds, retries, timeoutSeconds } }
}

/** 相同配置不提交，非法间隔先于保存返回错误。 */
export function readUsageChanges(account: AccountRecord, draft: UsageDraft): UsageChanges | null {
  const result = readUsageDraft(draft)
  if ('error' in result) return result
  const { enabled, interval, retries, timeoutSeconds } = result.value
  const current = usageQueryOf(account)
  if (enabled === current.enabled && (!enabled || interval === current.interval) && retries === current.retries && timeoutSeconds === current.timeoutSeconds) return null
  return { patch: { usageQuery: { enabled, interval: enabled ? interval : current.interval, retries, timeoutSeconds } } }
}

/** 「查询设置」段（受控组件：草稿在弹窗手里，保存时由 readUsageChanges 汇总） */
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
            aria-label={t('Auto-query balance')} />
          <span className='text-xs text-subtle'>{t('Auto-query balance (refresh the balance column reading at the interval below)')}</span>
        </FieldLabel>
      </Field>
      {draft.enabled ? (
        <FieldGroup orientation='horizontal' className='field-row mt-2.5 acct-usage-interval'>
          <Field className='contents'>
            <FieldLabel htmlFor={`${idPrefix}-interval`}>{t('Query interval')}</FieldLabel>
            <Input id={`${idPrefix}-interval`} type='number' min={1} step={1}
              className='max-w-[110px]' value={draft.intervalInput}
              onChange={event => patch({ intervalInput: event.currentTarget.value })} />
          </Field>
          <Select value={draft.unit} onValueChange={value => patch({ unit: value as IntervalUnit })}>
            <SelectTrigger className='w-[92px]' aria-label={t('Interval unit')}>
              <SelectValue>{draft.unit === 'hours' ? t('hours') : draft.unit === 'minutes' ? t('minutes') : t('seconds')}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectItem value='seconds'>{t('seconds')}</SelectItem>
              <SelectItem value='minutes'>{t('minutes')}</SelectItem>
              <SelectItem value='hours'>{t('hours')}</SelectItem>
            </SelectContent>
          </Select>
          <span className='detail'>{t('30 seconds to 24 hours; a manual query postpones the next round')}</span>
        </FieldGroup>
      ) : null}
      <Field orientation='horizontal' className='field-row mt-2.5 acct-usage-interval'>
        <FieldLabel htmlFor={`${idPrefix}-timeout`}>{t('Overall query timeout')}</FieldLabel>
        <Input id={`${idPrefix}-timeout`} type='number' min={15} max={USAGE_QUERY_MAX_TIMEOUT} step={1}
          className='max-w-[110px]' value={draft.timeoutInput ?? String(USAGE_QUERY_DEFAULT_TIMEOUT)}
          onChange={event => patch({ timeoutInput: event.currentTarget.value })} />
        <span className='detail'>{t('seconds; defaults to 60, including retry waits. Increase for multi-step queries if needed.')}</span>
      </Field>
      <Field orientation='horizontal' className='field-row mt-2.5 acct-usage-interval'>
        <FieldLabel htmlFor={`${idPrefix}-retries`}>{t('Retries after failure')}</FieldLabel>
        <Input id={`${idPrefix}-retries`} type='number' min={0} max={USAGE_QUERY_MAX_RETRIES} step={1}
          className='max-w-[110px]' value={draft.retriesInput} aria-describedby={`${idPrefix}-retries-hint`}
          onChange={event => patch({ retriesInput: event.currentTarget.value })} />
        <span id={`${idPrefix}-retries-hint`} className='detail'>
          {t('Only temporary failures are retried. Stops when either the retry count or overall time limit is reached. Credential errors are not retried.')}
        </span>
      </Field>
    </>
  )
}
