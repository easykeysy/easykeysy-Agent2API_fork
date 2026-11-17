import { useAccountUsage } from '../hooks/useAccountUsage'
import { useEffect, useState } from 'react'
import { Sheet, SheetContent, SheetHeader, SheetBody, SheetFooter, SheetTitle , Input, Button } from '@ui'
import { Icon } from '../components/Icon'
import { ProviderLogo } from '../components/ProviderLogo'
import { Badge } from '../components/ui'
import { store } from '../state/store'
import {
  clearAccountRateLimits,
  deleteAccount,
  promoteAccount,
  refreshAccount,
  renameAccount,
  setAccountEnabled,
  type Account,
} from '../api/gateway'
import { accountName, activeLimits, formatExpiry, formatRelative, formatStamp, isRateLimited } from '../lib/format'
import { expiryOf, hasExpiry, isLongLivedCredential, supportsUsage } from '../lib/providerFacts'
import { usageText } from '../lib/usage'
import { useDisplayClock } from '../../../ui-shared/use-display-clock'
import { credentialExpiryRowsOf, credentialExpiryText } from '../../../ui-shared/credential-expiry'
import { upstreamCooldownOf, upstreamSuspendedReasonOf } from '../../../ui-shared/account-upstream-state'
import { providerLabel } from '../lib/providers'
import { serverIdentity } from '../lib/servers'
import { t } from '../i18n'
import { confirmDialog } from '@ui'
import { MetricGrid, MetricCard } from '@ui'

/**
 * 账号详情抽屉。
 *
 * 手机上不做「编辑表单」：账号的凭证、代理、限流规则都是多字段表单，窄屏填
 * 起来痛苦且容易出错。这一层只给**查看 + 几个高频动作**（启停、刷新凭证、
 * 查询余额、设为首选、清除限流、删除），编辑仍回桌面端。
 */
export function AccountSheet({
  account,
  onClose,
  onChanged,
}: {
  account: Account | null
  onClose: () => void
  onChanged: () => void
}) {
  const now = useDisplayClock()
  const [busy, setBusy] = useState('')
  const [editingName, setEditingName] = useState(false)
  const [nameDraft, setNameDraft] = useState('')
  const open = Boolean(account)
  const accountId = account?.id || ''
  const identity = serverIdentity()
  const usage = useAccountUsage(accountId, Boolean(account && supportsUsage(account)))
  const usageRow = usage.row
  useEffect(() => {
    if (usageRow?.code === 'refresh_token_expired') onChanged()
  }, [accountId, usageRow?.code, usageRow?.requestId])

  useEffect(() => {
    setEditingName(false)
    setNameDraft('')
  }, [accountId])

  const run = async (kind: string, task: () => Promise<unknown>, success: string): Promise<boolean> => {
    if (serverIdentity() !== identity) return false
    setBusy(kind)
    try {
      await task()
      if (serverIdentity() !== identity) return false
      store.toast(success)
      onChanged()
      return true
    } catch (error) {
      if (serverIdentity() === identity) store.toast(error instanceof Error ? error.message : String(error), 'err')
      return false
    } finally {
      if (serverIdentity() === identity) setBusy('')
    }
  }

  const remove = async () => {
    if (!account) return
    const name = accountName(account)
    // 删除不可恢复：手机上用两次确认（第一次进二次确认态）
    if (!await confirmDialog(t('Delete account "{name}"? This action cannot be undone.', { name }))) return
    if (await run('delete', () => deleteAccount(account.id), t('The account has been deleted'))) onClose()
  }

  const saveName = async () => {
    if (!account || serverIdentity() !== identity) return
    const next = nameDraft.trim()
    if (!next) {
      store.toast(t('Label cannot be empty'), 'err')
      return
    }
    setBusy('name')
    try {
      await renameAccount(account.id, next)
      if (serverIdentity() !== identity) return
      store.toast(t('Label updated'))
      setEditingName(false)
      onChanged()
    } catch (error) {
      if (serverIdentity() === identity) store.toast(error instanceof Error ? error.message : String(error), 'err')
    } finally {
      if (serverIdentity() === identity) setBusy('')
    }
  }

  /** 不传 model 时同时清除账号冷却和全部模型限流。 */
  const clearLimit = async (model?: string) => {
    if (!account) return
    const question = model
      ? t('Clear the rate-limit flag for model "{model}"?', { model })
      : t('Clear this account\'s cooldown and all model rate-limit flags?')
    if (!await confirmDialog(question)) return
    await run('clear', () => clearAccountRateLimits(account.id, model), t('Rate limit cleared'))
  }

  const promote = async () => {
    if (!account) return
    await run('promote', () => promoteAccount(account.id), t('Set as first'))
  }

  const limits = account ? activeLimits(account) : []
  const cooldown = upstreamCooldownOf(account)
  const suspendedReason = upstreamSuspendedReasonOf(account)
  const expiresAt = account ? expiryOf(account) : 0
  const credentialRows = account ? credentialExpiryRowsOf(account, expiresAt) : []
  const usageView = usageText(usageRow, now)
  const usageColor =
    usageView.kind === 'bad'
      ? 'var(--m-danger)'
      : usageView.kind === 'warn'
        ? 'var(--m-warn)'
        : 'var(--m-text)'
  const isPrimary = (store.get().session?.routedAccountId || store.get().session?.accounts?.currentAccountId) === account?.id
  const sortActive = (store.get().session?.accounts?.sorting?.rules[String(account?.provider || '')] || []).some(rule => rule.on)
  const canQueryUsage = Boolean(account && supportsUsage(account))
  const canRefresh = Boolean(account && account.desktop !== true && (account.canRefresh === true || account.hasRefreshToken === true))

  return (
    <Sheet open={open} onOpenChange={next => { if (!next) onClose() }}>
      <SheetContent>
        <SheetHeader>
          <SheetTitle>{t('Account details')}</SheetTitle>
        </SheetHeader>
        <SheetBody>
          {account ? (
            <>
              <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                <ProviderLogo id={account.provider} size='lg' />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 16, fontWeight: 600, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {accountName(account)}
                  </div>
                  <div style={{ fontSize: 12, color: 'var(--m-text-3)', marginTop: 3 }}>
                    {providerLabel(account.provider, account.editionLabel)}
                    {account.priority !== undefined ? ` · #${account.priority}` : ''}
                  </div>
                </div>
                <Badge tone={statusTone(account)} dot>
                  {statusText(account)}
                </Badge>
              </div>

              {/* 备注名就地编辑：手机上没有表单页，改一个字段不该跳走 */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  minHeight: 38,
                  padding: '6px 10px',
                  borderRadius: 11,
                  background: 'var(--m-surface-2)',
                  border: '1px solid var(--m-line)',
                }}
              >
                {editingName ? (
                  <>
                    <Input
                      value={nameDraft}
                      autoFocus
                      onChange={event => setNameDraft(event.target.value)}
                      placeholder={t('Label')}
                      style={{
                        flex: 1,
                        minWidth: 0,
                        border: 0,
                        outline: 'none',
                        background: 'transparent',
                        color: 'var(--m-text)',
                        // 16px 以下 iOS Safari 聚焦时会自动放大页面
                        fontSize: 16,
                      }}
                    />
                    <Button size='touch-sm' variant='default'
                      type='button'
                      className='m-btn m-sm'
                      disabled={busy === 'name'}
                      onClick={() => void saveName()}
                    >
                      {t('Save')}
                    </Button>
                    <Button size='touch-sm' variant='outline'
                      type='button'
                      className='m-btn m-ghost m-sm'
                      disabled={busy === 'name'}
                      onClick={() => {
                        setEditingName(false)
                        setNameDraft('')
                      }}
                    >
                      {t('Cancel')}
                    </Button>
                  </>
                ) : (
                  <Button size='default' variant='ghost'
                    type='button'
                    onClick={() => {
                      setNameDraft(account.name || '')
                      setEditingName(true)
                    }}
                    style={{
                      flex: 1,
                      minWidth: 0,
                      display: 'flex',
                      alignItems: 'center',
                      gap: 8,
                      padding: 0,
                      border: 0,
                      background: 'none',
                      textAlign: 'left',
                      color: account.name ? 'var(--m-text)' : 'var(--m-text-3)',
                      fontSize: 13,
                    }}
                  >
                    <span style={{ flex: 1, minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {account.name || t('No label')}
                    </span>
                    <Icon name='edit' style={{ width: 15, height: 15, flex: 'none', color: 'var(--m-text-3)' }} />
                  </Button>
                )}
              </div>

              <MetricGrid className='m-kv'>
                {credentialRows.length || hasExpiry(account.provider) ? (
                  <MetricCard className='m-kv-cell'>
                    <div className='k'>{t('Expiry')}</div>
                    {credentialRows.length ? <div className='flex flex-col gap-1.5'>
                      {credentialRows.map(row => <div key={row.label} className='flex items-center justify-between gap-2 text-xs'
                        title={row.expiresAt === null ? undefined : formatStamp(row.expiresAt)}>
                        <span className='text-subtle' translate='no'>{row.label}</span>
                        <span className={row.expiresAt !== null && row.expiresAt <= Date.now() ? 'text-destructive' : undefined}>
                          {credentialExpiryText(row, t)}
                        </span>
                      </div>)}
                    </div> : <div className='v'>
                      {isLongLivedCredential(account.provider) ? t('Long-lived') : formatExpiry(expiresAt)}
                    </div>}
                  </MetricCard>
                ) : null}
                <MetricCard className='m-kv-cell'>
                  <div className='k'>{t('Balance')}</div>
                  <div className='v' title={canQueryUsage ? usageView.detail : undefined}
                    style={{ color: usageView.failureStatus ? 'var(--m-text)' : usageColor }}>
                    {canQueryUsage ? usageView.text : t('Query not integrated')}
                  </div>
                  {canQueryUsage && usageView.expiry ? (
                    <div className='text-xs text-warning mt-1 whitespace-normal leading-snug'>
                      <div>{usageView.expiry.summary}</div>
                      {usageView.expiry.today ? <div>{usageView.expiry.today}</div> : null}
                    </div>
                  ) : null}
                  {canQueryUsage && usageView.failureStatus ? (
                    <div className='text-xs text-destructive mt-1 whitespace-normal leading-snug' title={usageView.failureDetail}>
                      {usageView.failureStatus}
                    </div>
                  ) : null}
                </MetricCard>
                <MetricCard className='m-kv-cell'>
                  <div className='k'>{t('Credentials')}</div>
                  <div className='v'>{account.hasCredentials === false ? t('Missing') : t('Complete')}</div>
                </MetricCard>
                <MetricCard className='m-kv-cell'>
                  <div className='k'>{t('Rate-limited models')}</div>
                  <div className='v'>{limits.length}</div>
                </MetricCard>
                <MetricCard className='m-kv-cell'>
                  <div className='k'>{t('The most recent update')}</div>
                  <div className='v m-mono' style={{ fontSize: 12 }}>
                    {formatStamp(account.updatedAt as number)}
                  </div>
                </MetricCard>
              </MetricGrid>

              {/* 首选是队列位次，不是账号属性：给它一个整行按钮，别混进删除那一排 */}
              <Button size='touch' variant='outline'
                type='button'
                className='m-btn m-ghost w-full'
                disabled={isPrimary || Boolean(busy) || sortActive}
                title={sortActive ? t('Automatic consumption sorting is enabled. Turn off the rules before reordering manually') : undefined}
                onClick={() => void promote()}
                style={{ flexDirection: 'column', gap: 2, height: 'auto', padding: '10px 16px' }}
              >
                <span>{isPrimary ? t('Already first') : t('Set as first')}</span>
                <span style={{ fontSize: 11, fontWeight: 400, color: 'var(--m-text-3)' }}>
                  {t('Move to the front of the global queue')}
                </span>
              </Button>

              <div className='m-rows'>
                <div className='m-row'>
                  <span className='m-k'>{t('Account ID')}</span>
                  <span className='m-v m-mono' style={{ fontSize: 11 }}>
                    {account.id}
                  </span>
                </div>
                {account.email ? (
                  <div className='m-row'>
                    <span className='m-k'>{t('Email')}</span>
                    <span className='m-v'>{account.email}</span>
                  </div>
                ) : null}
                <div className='m-row'>
                  <span className='m-k'>{t('enabled')}</span>
                  <span className='m-v'>{account.enabled ? t('Yes') : t('No')}</span>
                </div>
                {/* 读数之外还有话要说时才占一行（部分失败、钱包明细） */}
                {canQueryUsage && usageView.detail && usageView.detail !== usageView.text ? (
                  <div className='m-row' style={{ alignItems: 'flex-start' }}>
                    <span className='m-k' style={{ flex: 'none' }}>{t('Balance details')}</span>
                    <span
                      className='m-v'
                      style={{ flex: 1, minWidth: 0, textAlign: 'right', lineHeight: 1.5, wordBreak: 'break-word' }}
                    >
                      {usageView.detail}
                    </span>
                  </div>
                ) : null}
                {suspendedReason ? (
                  <div className='m-row' style={{ alignItems: 'flex-start' }}>
                    <span className='m-k'>{t('Upstream restriction · Disabled')}</span>
                    <span className='m-v' style={{ color: 'var(--m-danger)', whiteSpace: 'normal', wordBreak: 'break-word', lineHeight: 1.5 }}>
                      {suspendedReason}
                    </span>
                  </div>
                ) : null}
                {account.needsRelogin ? (
                  <div className='m-row' style={{ alignItems: 'flex-start' }}>
                    <span className='m-k'>{t('Sign-in required')}</span>
                    <span className='m-v' style={{ color: 'var(--m-danger)', whiteSpace: 'normal', wordBreak: 'break-word', lineHeight: 1.5 }}>
                      {account.needsReloginReason || t('Sign-in authorization is no longer valid. Sign in again; automatic renewal cannot restore an expired or revoked refresh token.')}
                    </span>
                  </div>
                ) : null}
              </div>

              {limits.length || cooldown ? (
                <>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 2px' }}>
                    <span style={{ flex: 1, fontSize: 12, color: 'var(--m-text-3)' }}>{t('Rate-limit details')}</span>
                    <Button size='touch-sm' variant='outline'
                      type='button'
                      className='m-btn m-ghost m-sm'
                      disabled={busy === 'clear'}
                      onClick={() => void clearLimit()}
                    >
                      {t('Clear all')}
                    </Button>
                  </div>
                  <div className='m-rows'>
                    {cooldown ? (
                      <div className='m-row' style={{ alignItems: 'flex-start' }}>
                        <span className='m-k'>{t('Account cooldown')}</span>
                        <div className='m-v' style={{ whiteSpace: 'normal', wordBreak: 'break-word', lineHeight: 1.5 }}>
                          <div title={formatStamp(cooldown.resetAt)}>{t('Resets {time}', { time: formatRelative(cooldown.resetAt) })}</div>
                          <div>{t('All models on this account are paused · Resumes automatically at the recovery time')}</div>
                          {cooldown.message ? <div>{cooldown.message}</div> : null}
                        </div>
                      </div>
                    ) : null}
                    {limits.map(({ model, resetAt }) => (
                      <div key={model} className='m-row'>
                        <span className='m-k m-mono' style={{ fontSize: 11 }}>
                          {model}
                        </span>
                        <span className='m-v'>
                          {t('Resets {time}', { time: formatRelative(resetAt) })}
                        </span>
                        <Button size='touch-sm' variant='outline'
                          type='button'
                          className='m-btn m-ghost m-sm'
                          style={{ flex: 'none', marginLeft: 8 }}
                          disabled={busy === 'clear'}
                          onClick={() => void clearLimit(model)}
                        >
                          {t('Clear')}
                        </Button>
                      </div>
                    ))}
                  </div>
                </>
              ) : null}
            </>
          ) : null}
        </SheetBody>
        <SheetFooter style={{ flexDirection: 'column', alignItems: 'stretch', gap: 8 }}>
          {canQueryUsage ? <Button size='touch' variant='outline'
            type='button'
            className='m-btn m-ghost w-full'
            disabled={usage.starting || Boolean(busy) || !account}
            onClick={() => void usage.query()}
          >
            <Icon name='wallet' />
            {usage.starting ? t('Starting…') : usage.task?.status === 'running' ? t('Check again') : t('Query balance')}
          </Button> : null}
          {usage.status ? <p className='text-xs text-subtle' role='status'>{usage.status}</p> : null}
          {usageRow?.stale ? <p className='text-xs text-subtle'>{t('Previous balance, not yet verified with the new settings')}</p> : null}
          {usage.task?.status === 'running' ? <Button variant='outline' onClick={() => void usage.cancel()}>{t('Cancel query')}</Button> : null}
          {canRefresh ? <Button size='touch' variant='default'
            type='button'
            className='m-btn w-full'
            disabled={Boolean(busy) || !account || account.needsRelogin === true}
            title={account?.needsRelogin ? account.needsReloginReason : undefined}
            onClick={() => account && void run('refresh', () => refreshAccount(account.id), t('Credentials refreshed'))}
          >
            <Icon name='refresh' />
            {busy === 'refresh' ? t('Refreshing…') : t('Refresh the credentials')}
          </Button> : account ? <p className='text-xs text-subtle'>{t('Manual renewal is not declared. For local references, manage sign-in in the original client.')}</p> : null}
          <div className='m-btn-row'>
            <Button size='touch' variant='outline'
              type='button'
              className='m-btn m-ghost w-full'
              style={{ flex: 1 }}
              disabled={busy === 'toggle' || !account}
              onClick={() =>
                account &&
                void run(
                  'toggle',
                  () => setAccountEnabled(account.id, !account.enabled),
                  account.enabled ? t('Disabled') : t('Enabled'),
                )
              }
            >
              <Icon name={account?.enabled ? 'pause' : 'play'} />
              {account?.enabled ? t('Disable') : t('enabled')}
            </Button>
            <Button size='touch' variant='destructive'
              type='button'
              className='m-btn m-danger w-full'
              style={{ flex: 1 }}
              disabled={busy === 'delete' || !account}
              onClick={() => void remove()}
            >
              <Icon name='trash' />
              {busy === 'delete' ? t('Deleting…') : t('Delete')}
            </Button>
          </div>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}

function statusTone(account: Account): 'ok' | 'warn' | 'bad' | 'mute' {
  if (account.needsRelogin || upstreamSuspendedReasonOf(account)) return 'bad'
  if (!account.enabled) return 'mute'
  if (isRateLimited(account)) return 'warn'
  const expiresAt = expiryOf(account)
  if (expiresAt > 0 && expiresAt <= Date.now()) return 'bad'
  return 'ok'
}

function statusText(account: Account): string {
  if (account.needsRelogin) return t('Relogin')
  if (upstreamSuspendedReasonOf(account)) return t('Upstream restriction · Disabled')
  if (!account.enabled) return t('Disabled')
  if (upstreamCooldownOf(account)) return t('Account cooldown')
  if (isRateLimited(account)) return t('Rate-limited')
  const expiresAt = expiryOf(account)
  if (expiresAt > 0 && expiresAt <= Date.now()) return t('Expired')
  return t('Normal')
}
