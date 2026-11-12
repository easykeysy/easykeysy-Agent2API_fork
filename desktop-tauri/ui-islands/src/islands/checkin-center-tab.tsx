import * as React from 'react'
import { Badge, Button, Card, CardHeader, Empty, Progress, SegmentedControl, Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from '@ui'
import {
  growthOf, loadCenter, patch, queryGrowth, runAllCheckin, runGrowth,
  runSchool, runTravelOnce, signSingle, store,
} from './checkin-store'
import {
  CHECKIN_SECTION_OPTIONS, MODE_LABEL, MODE_TITLE, formatTime, modesOf,
  type CheckinAccount, type CheckinCenter, type GrowthEntry, type OnboardingEntry,
  type PlanEntry, type ProviderGroup,
} from './checkin-types'
import { CheckinRow, ProviderLogo } from './checkin-row'
import { CheckinUmid } from './checkin-umid'
import { claimOnboarding, onboardingOf, queryOnboarding } from './checkin-onboarding'
import { t } from '../i18n'
import { growthTaskCompleted, growthTaskRunnable } from '../../../ui-shared/checkin'
import {
  accountCheckinStatus, checkinActionLabel, checkinPlanProgress, checkinStatusLabel, checkinStatusTone,
  type AccountCheckinStatus,
} from '../../../ui-shared/checkin-result'

/**
 * 「 to in 」：total + daily to + new + + 。
 *
 * ── ────────────────────────────────────────
 * 、 already 、Reason in （`api::checkin_center`）。 this 。
 * ** not need ** by provider id —— that is （`description` /
 * `modes` / `uiHooks` / `onboarding` / `plans` / `growth`）。
 *
 * ── ──────────────────────────────────────────────────
 * new and **** need up ， not （must「 up request」）。
 * up has 「」 by ，。
 */

/* ─── empty ─────────────────────────────────── */

/**
 * no when empty。
 *
 * this is 「 to in 」 ， is ： use to 1 and ，
 * not is 1 empty 。 is `checkinFeatures` as empty ——
 * that is ，empty = no has to can 。
 */
function EmptyState(): React.ReactElement {
  const hasFeature = Object.keys(store.center?.checkinFeatures ?? {}).length > 0
  const hasAccounts = (store.center?.daily?.providers?.length ?? 0) > 0
  const missingPlugin = !hasFeature
  const message = missingPlugin
    ? '还没有可签到的提供商。签到能力由提供商插件提供，请先到「插件」页安装 Agent2API 提供商插件。'
    : !hasAccounts
      ? '插件已安装，但还没有可签到的账号。到「账号」页添加并启用账号后，这里会自动出现。'
      : ''
  if (!message) return <></>
  return (
    <Card render={<section />} className="panel">
      <CardHeader className="panel-head">
        <h2>每日签到</h2>
      </CardHeader>
      <Empty className="ck-empty-state">
        <p>{message}</p>
        <div className="ck-empty-actions">
          {/* use `showPage` not is location： is use ， no has hash by */}
          <Button
            size="sm"
            variant="outline"
            onClick={() => window.wbApp?.showPage?.(missingPlugin ? 'plugins' : 'accounts')}
          >
            去「{missingPlugin ? '插件' : '账号'}」页
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void loadCenter()}>
            重新检查
          </Button>
        </div>
      </Empty>
    </Card>
  )
}

/* ─── total ───────────────────────────────── */

function StatCard({ label, value, foot, extra, onClick }: {
  label: string
  value: React.ReactNode
  foot?: React.ReactNode
  extra?: React.ReactNode
  onClick?: () => void
}): React.ReactElement {
  const className = onClick ? 'ck-stat ck-stat-link' : 'ck-stat'
  return (
    <Card
      render={onClick ? <Button variant='outline' nativeButton={false} render={<div />} /> : undefined}
      className={`${className} block h-auto w-full whitespace-normal text-left font-normal`}
      onClick={onClick}
    >
      <div className="ck-stat-label">{label}</div>
      <div className="ck-stat-value">{value}</div>
      {foot ? <div className="ck-stat-foot">{foot}</div> : null}
      {extra}
    </Card>
  )
}

/**
 * total（， and old 1 ）。
 *
 * old is ： to （）/ when to （ can 「 when to 」）/
 * new / to （ can 「 to 」）。
 * ， by `checkin-page.tsx` `wb:checkin-tab`。
 */
function StatsGrid({ center }: { center: CheckinCenter }): React.ReactElement {
  const { daily, extras } = center
  const auto = center.auto
  const percent = daily.todayEligible > 0
    ? (daily.todayDone / daily.todayEligible) * 100
    : 0
  const outCount = daily.outOfScope.reduce((sum, item) => sum + item.count, 0)

  // when to ：enabled and down 1 when 
  const enabledEntries = (auto?.schedules ?? []).filter(entry => entry.enabled)
  const nextRun = enabledEntries
    .map(entry => Number(entry.nextRunAt))
    .filter(stamp => Number.isFinite(stamp) && stamp > 0)
    .sort((a, b) => a - b)[0]

  // new ： already to has 
  let onboardingChecked = 0
  let onboardingPending = 0
  for (const row of extras.onboarding) {
    const cache = store.onboarding.get(row.id)
    if (cache?.status !== 'loaded') continue
    onboardingChecked += 1
    if (!cache.settled) onboardingPending += cache.tasks.filter(task => !task.done && !task.blocked).length
  }

  const goTab = (tab: string) => window.dispatchEvent(new CustomEvent('wb:checkin-tab', { detail: { tab } }))

  return (
    <div className="ck-stats">
      <StatCard
        label="今日签到"
        value={daily.todayEligible > 0
          ? <>{daily.todayDone}<small> / {daily.todayEligible}</small></>
          : <span className="ck-pending">—</span>}
        extra={daily.todayEligible > 0
          ? <Progress className="ck-stat-bar" value={percent} />
          : null}
        foot={outCount > 0 ? `另有 ${outCount} 个账号不参与每日签到` : '全部账号均可签到'}
      />
      <StatCard
        label="定时签到"
        value={enabledEntries.length > 0
          ? <>{enabledEntries.length}<small> / {auto?.schedules?.length ?? 0} 条启用</small></>
          : <span className="ck-pending">—</span>}
        foot={auto?.enabled !== true
          ? '总开关已关闭 · 在「定时签到」页打开'
          : nextRun
            ? `下次 ${formatTime(nextRun)} · 点此管理条目`
            : '暂无启用的条目 · 点此添加'}
        onClick={() => goTab('schedule')}
      />
      <StatCard
        label="待领新手福利"
        value={extras.onboarding.length > 0
          ? onboardingChecked > 0
            ? <>{onboardingPending}<small> 项</small></>
            : <span className="ck-pending">—</span>
          : <span className="ck-pending">无</span>}
        foot={extras.onboarding.length > 0
          ? `已有结果 ${onboardingChecked} / 共 ${extras.onboarding.length} 个账号`
          : '没有支持新手福利的账号'}
        onClick={() => patch({ centerSection: 'onboarding' })}
      />
      <StatCard
        label="累计签到记录"
        value={store.recordsLoaded ? store.recordsTotal : <span className="ck-pending">—</span>}
        foot="每账号一条明细 · 点此查看「签到记录」"
        onClick={() => goTab('records')}
      />
    </div>
  )
}

/**
 * （** 1 **， not is ）。
 *
 * old is in 「daily to 」 `ck-fold-note` ： is that 
 * ****（「Why this no in up 」）， will 
 * is 1 can 。 this is when 1 。
 */
function OutOfScopeNote({ items }: { items: { label: string; reason: string; count: number }[] }): React.ReactElement {
  const list = items
    .map(item => `${item.label} ×${item.count}（${item.reason}）`)
    .join('；')
  return (
    <div className="ck-fold-note">
      {t('不参与每日签到')}：{list}。
    </div>
  )
}

/* ─── daily to ─────────────────────────────── */

function StatusBadge({ status }: { status: AccountCheckinStatus }): React.ReactElement {
  const variants = { ok: 'success', warn: 'warning', bad: 'destructive', mute: 'outline', brand: 'brand' } as const
  return <Badge variant={variants[checkinStatusTone(status)]} shape='tag'>{t(checkinStatusLabel(status))}</Badge>
}

/** ；“ / ”total。 */
function GroupBadge({ group }: { group: ProviderGroup }): React.ReactElement {
  if (!group.totalCount) return <Badge variant="outline" shape="tag">无账号</Badge>
  if (group.doneCount >= group.totalCount) return <StatusBadge status='done' />
  const states = group.accounts.map(accountCheckinStatus)
  if (states.includes('confirming')) return <StatusBadge status='confirming' />
  if (states.includes('failed')) return <StatusBadge status='failed' />
  if (states.includes('captcha')) return <StatusBadge status='captcha' />
  if (states.every(status => status === 'inactive')) return <StatusBadge status='inactive' />
  if (states.every(status => status === 'ineligible')) return <StatusBadge status='ineligible' />
  if (group.doneCount > 0) return <Badge variant="warning" shape="tag">部分</Badge>
  return <StatusBadge status='pending' />
}

/** （） */
function AccountTable({ group }: { group: ProviderGroup }): React.ReactElement {
  const modes = modesOf(group)
  // by ：keepalive / claim / full 1 by —— 、
  // ， is 
  const dailyActivity = modes.some(mode => mode === 'keepalive' || mode === 'claim' || mode === 'full')
  return (
    <Table container={false} className="ck-table">
      <TableHeader>
        <TableRow>
          <TableHead>账号</TableHead>
          <TableHead>今日签到</TableHead>
          <TableHead>上次签到时间</TableHead>
          <TableHead />
        </TableRow>
      </TableHeader>
      <TableBody>
        {group.accounts.map(account => (
          <AccountRow key={account.id} account={account} modes={modes} dailyActivity={dailyActivity} manualPlans={group.manualPlans === true} />
        ))}
      </TableBody>
    </Table>
  )
}

function AccountRow({ account, modes, dailyActivity, manualPlans }: {
  account: CheckinAccount
  modes: string[]
  dailyActivity: boolean
  manualPlans: boolean
}): React.ReactElement {
  const busy = (mode: string) => store.signing.has(`${account.id}:${mode}`)
  const status = accountCheckinStatus(account)
  const daily = account.dailyCheckin
  return (
    <TableRow>
      <TableCell>
        {account.name}
        {/* ：0 = no to ， not 「 0 」 */}
        {account.streakDays > 0
          ? <span className="ck-sub"> · 连登 {account.streakDays} 天</span>
          : null}
        {account.available ? null : <Badge variant="outline" shape="tag">凭据不可用</Badge>}
      </TableCell>
      <TableCell>
        <StatusBadge status={status} />
        {checkinPlanProgress(daily) && daily
          ? <span className="ck-sub"> · {t('已领 {done} / {total} 份套餐', {
            done: daily.completedCount, total: daily.totalCount,
          })}</span> : null}
        {daily?.message && status !== 'done'
          ? <div className='ck-sub max-w-[420px] whitespace-normal break-words'>{t(daily.message)}</div> : null}
      </TableCell>
      <TableCell className="ck-sub">{formatTime(account.checkinAt)}</TableCell>
      <TableCell className="ck-right">
        <span className="ck-inline">
          {dailyActivity
            ? modes.map(mode => (
              <Button
                key={mode}
                size="sm"
                variant="ghost"
                disabled={busy(mode) || !account.available}
                title={MODE_TITLE[mode]}
                onClick={() => void signSingle(account.id, mode)}
              >
                {busy(mode) ? `${MODE_LABEL[mode] ?? mode}中…` : (MODE_LABEL[mode] ?? mode)}
              </Button>
            ))
            : (
              <Button
                size="sm"
                variant="ghost"
                disabled={busy('checkin') || !account.available}
                onClick={() => {
                  if (manualPlans && window.wbZcodePlans?.open) {
                    window.wbZcodePlans.open({ id: account.id, name: account.name })
                  } else {
                    void signSingle(account.id, 'checkin')
                  }
                }}
              >
                {manualPlans ? t('套餐明细') : busy('checkin')
                  ? t(daily?.queryOnly ? '查询中…' : '签到中…') : t(checkinActionLabel(account))}
              </Button>
            )}
        </span>
      </TableCell>
    </TableRow>
  )
}

function ProviderRow({ group }: { group: ProviderGroup }): React.ReactElement {
  const [expanded, setExpanded] = React.useState(false)
  const hasAccounts = group.totalCount > 0
  return (
    <CheckinRow provider={group.id} name={group.label} description={group.description}
      note={group.uiHooks?.includes('umid_component') ? <CheckinUmid /> : undefined}
      expandable={hasAccounts} expanded={expanded} onExpandedChange={setExpanded}
      summary={<>
        <span className="ck-row-count">
          {hasAccounts ? `${group.totalCount} 账号 · ${group.doneCount} 已签` : '—'}
        </span>
        <GroupBadge group={group} />
      </>}>
      <AccountTable group={group} />
    </CheckinRow>
  )
}

/* ─── new ─────────────────────────────── */

/**
 * new 。
 *
 * need up ， not —— this by is 「」，。
 * in `store.onboarding`（， not use ）。
 */
function OnboardingRow({ row }: { row: OnboardingEntry }): React.ReactElement {
  const cache = onboardingOf(row.id)
  const [expanded, setExpanded] = React.useState(false)
  const tasks = cache.tasks
  const unclaimed = cache.settled ? 0 : tasks.filter(task => !task.done && !task.blocked).length
  const status = cache.status

  return (
    <CheckinRow provider={row.provider} name={row.name}
      expandable={tasks.length > 0} expanded={expanded} onExpandedChange={setExpanded}
      description={<>
        {cache.settled
          ? t('一次性福利已结清，无需重复查询')
          : status === 'loaded'
            ? `已领 ${tasks.filter(task => task.done).length}/${tasks.length} · 累计 ${cache.earned} 积分`
            : status === 'loading' ? '正在查询…' : '尚未查询任务状态'}
        {cache.settledAt ? ` · ${t('结算于 {time}', { time: formatTime(cache.settledAt) })}`
          : cache.checkedAt ? ` · 查询于 ${formatTime(cache.checkedAt)}` : ''}
      </>}
      summary={cache.settled
        ? <Badge variant="success" shape="tag">已领取</Badge>
        : status === 'loaded' && unclaimed > 0
          ? <Badge variant="warning" shape="tag">{unclaimed} 项待领</Badge>
          : status === 'loaded'
            ? <Badge variant="outline" shape="tag">{t('暂无可领取福利')}</Badge>
            : status === 'error'
              ? <Badge variant="destructive" shape="tag">查询失败</Badge>
              : null}
      note={status === 'error' && cache?.error ? `查询失败：${cache.error}` : undefined}
      actions={<>
        <Button
          size="sm"
          variant="outline"
          disabled={status === 'loading' || cache?.claiming === true}
          onClick={() => void queryOnboarding(row.id, true)}
        >
          {status === 'loading' ? '查询中…' : '查询任务'}
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={status !== 'loaded' || cache.claiming || cache.settled || (row.provider !== 'officeace' && unclaimed === 0)}
          onClick={() => void claimOnboarding(row.id)}
        >
          {cache?.claiming ? '领取中…' : unclaimed > 0 ? `一键领取（${unclaimed}）` : '领取'}
        </Button>
      </>}>
      {tasks.map(task => (
        <div className="ck-task-line" key={task.key}>
          <span className={task.done ? 'ck-task-tick done' : 'ck-task-tick'}>{task.done ? '✓' : '·'}</span>
          <span className="ck-task-title">{task.title}</span>
          <span className="ck-task-pts">+{task.points}</span>
          <span className="ck-task-st">
            {task.claiming
              ? <Badge variant="warning" shape="tag">领取中</Badge>
              : task.done
                ? <Badge variant="success" shape="tag">已领取</Badge>
                : task.blocked
                  ? <Badge variant="outline" shape="tag">暂不可领</Badge>
                  : task.error
                    ? <Badge variant="destructive" shape="tag">{task.error}</Badge>
                    : <Badge variant="brand" shape="tag">可领取</Badge>}
          </span>
        </div>
      ))}
    </CheckinRow>
  )
}

/* ─── （ZCode ）────────────────── */

/**
 * （ when ） in id。
 *
 * up by 「」， by —— and `claimedPlanIdsToday`
 * 1 （ when will in up 「」）。
 */
function claimedPlansToday(claimPlans: Record<string, number> | null | undefined): string[] {
  if (!claimPlans || typeof claimPlans !== 'object') return []
  const today = new Date(Date.now() + 8 * 3600 * 1000).toISOString().slice(0, 10)
  return Object.entries(claimPlans)
    .filter(([, at]) => {
      const stamp = Number(at)
      if (!Number.isFinite(stamp) || stamp <= 0) return false
      return new Date(stamp + 8 * 3600 * 1000).toISOString().slice(0, 10) === today
    })
    .map(([planId]) => planId)
}

/**
 * 1 （ZCode when ）。
 *
 * this 1 ****： and in that （`wbZcodePlans`）。
 * to client after can —— in iframe not to that ， can 
 * postMessage ， no response when 1 「」。
 */
function PlanRow({ row }: { row: PlanEntry }): React.ReactElement {
  const claimed = claimedPlansToday(row.claimPlans)
  const hasClaimed = claimed.length > 0
  const desc = hasClaimed
    ? '点「套餐明细」看领到了哪几份、什么时候生效；活动期内每天可再领'
    : row.claimAt
      ? `上次领取 ${formatTime(row.claimAt)} · 领取需通过滑块验证码`
      : '限时体验套餐（start-plan），活动期内每天一份 · 领取需滑块验证码'
  return (
    <div className="ck-welfare-row">
      <ProviderLogo id={row.provider || 'zcode'} label="ZCode" />
      <div className="ck-prov-info">
        <div className="ck-prov-name">
          {row.name || row.id}
          {hasClaimed
            ? (
              <Badge
                variant="success"
                shape="tag"
                title={`本地台账里今天（北京时间）领过的套餐：${claimed.join('、')}`}
              >
                今天已领 {claimed.length} 份
              </Badge>
            )
            : <Badge variant="outline" shape="tag" title="本地台账里今天还没领过；活动期内每天发一份新套餐">今天未领</Badge>}
        </div>
        <div className="ck-prov-desc">{desc}</div>
      </div>
      <div className="ck-prov-right">
        <Button
          size="sm"
          variant="outline"
          onClick={() => window.wbZcodePlans?.open?.({ id: row.id, name: row.name })}
        >
          {hasClaimed ? '套餐明细' : '去领取'}
        </Button>
      </div>
    </div>
  )
}

/* ─── （WorkBuddy ）────────────── */

/**
 * 1 。
 *
 * ：（ / ）、（ + ）、。
 * need 1 up ， up is 「」 by ，。
 *
 * 「 1 」 use （ `runGrowth` ）： 1 request will 
 * API 60 seconds when 。
 */
function GrowthRow({ row }: { row: GrowthEntry }): React.ReactElement {
  const cache = growthOf(row.id)
  const tasks = cache.tasks ?? []
  const runnable = tasks.filter(task => growthTaskRunnable(task, 'full'))
  const pending = runnable.filter(task => !growthTaskCompleted(task))
  const loaded = cache.status === 'loaded' && tasks.length > 0
  const [expanded, setExpanded] = React.useState(false)
  const running = cache.running || cache.runningSchool

  return (
    <CheckinRow provider={row.provider || 'workbuddy'} name={row.name || row.id}
      expandable={loaded} expanded={expanded} onExpandedChange={setExpanded}
      description={<>
        {cache.status === 'loaded'
          ? `${runnable.length} 项任务 · ${pending.length} 项可完成`
          : '查询任务树并一键点亮 / 领奖（含开学季与猫猫旅行）'}
        {cache.checkedAt ? ` · 查询于 ${formatTime(cache.checkedAt)}` : ''}
      </>}
      summary={cache.status === 'loaded'
        ? pending.length > 0
          ? <Badge variant="warning" shape="tag">{pending.length} 项可完成</Badge>
          : <Badge variant="success" shape="tag">全部完成</Badge>
        : cache.status === 'error'
          ? <Badge variant="destructive" shape="tag">查询失败</Badge>
          : null}
      note={cache.status === 'error' ? `查询失败：${cache.error}` : undefined}
      actions={<>
        <Button
          size="sm"
          variant="outline"
          disabled={cache.status === 'loading' || running}
          onClick={() => void queryGrowth(row.id)}
        >
          {cache.status === 'loading' ? '查询中…' : '查询任务'}
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={cache.status !== 'loaded' || running}
          title="接单 + 点亮 + 领奖（只处理已实测沉淀的任务）"
          onClick={() => void runGrowth(row.id, 'full')}
        >
          {cache.running ? '执行中…' : '一键完成'}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={cache.status !== 'loaded' || running}
          title="只领取已完成任务的奖励（幂等，重复点不会重复入账）"
          onClick={() => void runGrowth(row.id, 'claim')}
        >
          领取
        </Button>
      </>}>
      <div className="ck-fold-note">
        {cache.energy !== null && cache.energy !== undefined ? `能量 ${cache.energy}` : ''}
        {cache.streak !== null && cache.streak !== undefined ? ` · 连登 ${cache.streak} 天` : ''}
        {cache.energy === null && cache.streak === null ? '（能量 / 连登读数本次未取到）' : ''}
      </div>
      {tasks.map(task => {
        const result = cache.results.find(item => item.code === task.code)
        return (
          <div className="ck-task-line" key={task.code}>
            <span className={task.status === 'claimed' ? 'ck-task-tick done' : 'ck-task-tick'}>
              {task.status === 'claimed' ? '✓' : '·'}
            </span>
            <span title={task.description || task.source || task.code}>
              {task.title || task.code}
              {task.target > 1 ? ` (${task.current}/${task.target})` : ''}
            </span>
            {result ? <span className="ck-task-pts">{result.message}</span> : null}
            <span className="ck-task-st">
              {task.status === 'claimed'
                ? <Badge variant="success" shape="tag">已领取</Badge>
                : growthTaskRunnable(task, 'claim')
                  ? <Badge variant="brand" shape="tag">可领取</Badge>
                  : task.unforgeable
                    ? <Badge variant="outline" shape="tag">需真实操作</Badge>
                    : !task.supported
                      ? <Badge variant="outline" shape="tag">不支持</Badge>
                      : task.locked
                        ? <Badge variant="outline" shape="tag">未解锁</Badge>
                        : <Badge variant="warning" shape="tag">待完成</Badge>}
            </span>
          </div>
        )
      })}
      {/* and ： 1 by （ not ） */}
      <div className="ck-grow-extra">
        <Button
          size="sm"
          variant="outline"
          disabled={running}
          onClick={() => void runSchool(row.id, false)}
        >
          {cache.runningSchool ? '执行中…' : '开学季任务'}
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={running}
          onClick={() => void runSchool(row.id, true)}
        >
          开学季 + 抽奖
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={running}
          onClick={() => void runTravelOnce(row.id)}
        >
          猫猫旅行一趟
        </Button>
      </div>
    </CheckinRow>
  )
}

/* ─── ─────────────────────────────────── */

export function CenterTab({ center }: { center: CheckinCenter }): React.ReactElement {
  const { daily, extras } = center
  const hasAnything = daily.providers.length > 0 || daily.outOfScope.length > 0
    || extras.onboarding.length > 0 || extras.plans.length > 0 || extras.growth.length > 0
  const section = store.centerSection
  const runningAll = store.runningAll || center.auto?.running === true

  return (
    <div className="ck-main">
      {hasAnything ? <StatsGrid center={center} /> : null}
      <div className="ck-tabs">
        <SegmentedControl
          options={CHECKIN_SECTION_OPTIONS}
          value={section}
          onValueChange={centerSection => patch({ centerSection })}
          aria-label={t('签到功能分类')}
          className="max-w-full flex-wrap"
        />
      </div>
      {section === 'daily' && daily.providers.length === 0 ? <EmptyState /> : null}

      {section === 'daily' && daily.providers.length > 0 ? (
        <Card render={<section />} className="panel">
          <CardHeader className="panel-head">
            <h2>每日签到</h2>
            <span className="panel-sub">点击行展开账号明细 · 单账号签到不受范围限制</span>
            <Button
              size="sm"
              variant="default"
              className="ml-auto flex-none"
              disabled={runningAll}
              onClick={() => void runAllCheckin()}
            >
              {runningAll ? '签到中…' : '立即全部签到'}
            </Button>
          </CardHeader>
          <div>
            {daily.providers.map(group => <ProviderRow key={group.id} group={group} />)}
            {/* is this ****（ OutOfScopeNote ） */}
            {daily.outOfScope.length > 0 ? <OutOfScopeNote items={daily.outOfScope} /> : null}
          </div>
        </Card>
      ) : null}

      {section === 'onboarding' ? <Card render={<section />} className="panel">
        <CardHeader className="panel-head">
          <h2>新手福利</h2>
          <span className="panel-sub">{t('一次性福利 · 已领取状态保存在网关，领完无需重复查询 · 点击行展开任务清单')}</span>
          {extras.onboarding.length > 0 ? (
            <Button
              size="sm"
              variant="outline"
              className="ml-auto flex-none"
              onClick={() => {
                for (const row of extras.onboarding) {
                  void queryOnboarding(row.id)
                }
              }}
            >
              全部查询
            </Button>
          ) : null}
        </CardHeader>
        {extras.onboarding.length > 0
          ? <div>{extras.onboarding.map(row => <OnboardingRow key={row.id} row={row} />)}</div>
          : <Empty className="ck-empty">没有支持新手福利的账号。</Empty>}
      </Card> : null}

      {section === 'plans' ? <Card render={<section />} className="panel">
        <CardHeader className="panel-head">
          <h2>活动福利</h2>
          <span className="panel-sub">限时体验套餐 · 领取需滑块验证码 · 逐份查看与领取</span>
        </CardHeader>
        {extras.plans.length > 0
          ? <div>{extras.plans.map(row => <PlanRow key={row.id} row={row} />)}</div>
          : <Empty className="ck-empty">没有可领套餐的账号（ZCode 未添加）。</Empty>}
      </Card> : null}

      {section === 'growth' ? <Card render={<section />} className="panel">
        <CardHeader className="panel-head">
          <h2>成长任务</h2>
          <span className="panel-sub">WorkBuddy 国内版 · 查询任务树后一键点亮 / 领奖 · 含开学季与猫猫旅行 · 点击行展开明细</span>
        </CardHeader>
        {extras.growth.length > 0
          ? <div>{extras.growth.map(row => <GrowthRow key={row.id} row={row} />)}</div>
          : <Empty className="ck-empty">没有支持成长任务的账号（需要 WorkBuddy 国内版）。</Empty>}
      </Card> : null}
    </div>
  )
}
