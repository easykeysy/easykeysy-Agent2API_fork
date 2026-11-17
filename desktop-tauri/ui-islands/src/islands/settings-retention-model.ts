import { t } from '../i18n'
import type { NumberField } from './settings-model'

export type RetentionField = NumberField & { kind: 'days' | 'count' }

export const RETENTION_MIN = 1
export const RETENTION_MAX = 99999
export const RETENTION_MIN_COUNT = 1
export const RETENTION_MAX_COUNT = 100_000_000

export const RETENTION_FIELDS: RetentionField[] = [
  {
    key: 'logRetentionDays',
    id: 'settings-retention-log',
    label: t('Event Log Retention Days'),
    unit: t('days'),
    kind: 'days',
    min: RETENTION_MIN,
    max: RETENTION_MAX,
    hint: t('System events such as sign-ins, account switches, and 429 switches. Range: 1–99999 days.'),
  },
  {
    key: 'logRetentionCount',
    id: 'settings-retention-log-count',
    label: t('Log record limit'),
    unit: t('rows'),
    kind: 'count',
    min: RETENTION_MIN_COUNT,
    max: RETENTION_MAX_COUNT,
    hint: t('Defaults to 10,000 records, with capacity allocated by log level. The oldest excess records are removed; retention days also apply.'),
  },
  {
    key: 'requestLogRetentionDays',
    id: 'settings-retention-request',
    label: t('Request log retention (days)'),
    unit: t('days'),
    kind: 'days',
    min: RETENTION_MIN,
    max: RETENTION_MAX,
    hint: t('Per-request records on the Request Logs page. Defaults to 99999 days; accepts 1–99999 days. The record limit also applies.'),
  },
  {
    key: 'requestLogRetentionCount',
    id: 'settings-retention-request-count',
    label: t('Request log record limit'),
    unit: t('rows'),
    kind: 'count',
    min: RETENTION_MIN_COUNT,
    max: RETENTION_MAX_COUNT,
    hint: t('Defaults to 1,000,000 records. The oldest excess request records and their raw bodies are removed without affecting daily aggregate reports; retention days also apply.'),
  },
  {
    key: 'dailyAggregateRetentionDays',
    id: 'settings-retention-daily',
    label: t('Daily aggregate retention (days)'),
    unit: t('days'),
    kind: 'days',
    min: RETENTION_MIN,
    max: RETENTION_MAX,
    hint: t('The heatmap and daily trends on the Reports page. Defaults to 99999 days (about 274 years — kept in full for practical purposes); range 1–99999 days.'),
  },
]

export function retentionShrinkHead(field: RetentionField, previous: number | null, value: number): string {
  if (field.kind === 'count') {
    return previous === null
      ? t('Could not read the current value of "{label}". Setting it to {count} records may delete excess historical data.', {
        label: field.label, count: value,
      })
      : t('"{label}" will change from {previous} to {count} records.', { label: field.label, previous, count: value })
  }
  return previous === null
    ? t('Could not read the current value of "{label}"; changing it to {days} days may delete excess historical data.', {
      label: field.label, days: value,
    })
    : t('"{label}" will change from {previous} days to {days} days.', { label: field.label, previous, days: value })
}

export function retentionSavedMessage(field: RetentionField, value: number, shrinking: boolean): string {
  if (field.kind === 'count') {
    return shrinking
      ? t('✅ Record limit set to {count}; excess records removed', { count: value })
      : t('✅ Record limit set to {count}', { count: value })
  }
  return shrinking
    ? t('✅ Kept for {days} days; older data cleared', { days: value })
    : t('✅ Kept for {days} days', { days: value })
}
