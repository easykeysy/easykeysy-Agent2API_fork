import type { AssistantReadApi, AssistantReadTool, Translate } from './assistant-read-types'
import {
  accountMeta, accountName, accountSummary, accountsOf, activeRateLimits, booleanArg,
  diagnosticText, enumArg, expectObject, expectRows, fields, findAccount, number,
  object, page, pageArgs, PAGE_PARAMS, result, rows, safeText, stringArg, strings, text,
} from './assistant-read-data'

const WINDOW_KEYS = ['reset', 'seconds', 'startsAt', 'endsAt', 'utcOffsetMinutes']
const QUOTA_KEYS = [
  'id', 'label', 'provider', 'channel', 'planId', 'planName', 'unit', 'remaining', 'total', 'used',
  'remainingPercent', 'startsAt', 'endsAt', 'resetsAt', 'unlimited', 'localEstimate', 'localOnly',
]

function quota(value: unknown) {
  const source = object(value)
  return {
    ...fields(source, QUOTA_KEYS), models: strings(source.models),
    window: source.window ? fields(source.window, WINDOW_KEYS) : null,
  }
}

function limiterGroup(value: Record<string, unknown>, state: Record<string, unknown>, t: Translate) {
  return {
    ...fields(value, ['enabled', 'action', 'target']),
    state: { ...fields(state, ['hit', 'account']), models: strings(state.models) },
    conditions: rows(value.conditions).map((condition, index) => {
      const reading = object(rows(state.conditions)[index])
      const modelValues: Record<string, number> = {}
      for (const [model, raw] of Object.entries(object(reading.modelValues))) {
        const count = number(raw)
        if (count !== null) modelValues[safeText(model)] = count
      }
      return {
        ...fields(condition, ['join', 'sourceId', 'metric', 'comparison', 'threshold']),
        models: strings(condition.models), source: condition.source ? quota(condition.source) : null,
        window: condition.window ? fields(condition.window, WINDOW_KEYS) : null,
        reading: {
          ...fields(reading, ['known', 'value', 'hit', 'local', 'windowStart', 'resetsAt']),
          reason: diagnosticText(reading.reason, t), models: strings(reading.models), modelValues,
        },
      }
    }),
  }
}

function proxySummary(value: unknown, t: Translate) {
  if (!value) return { source: 'direct' }
  const proxy = object(value)
  const config = object(proxy.config)
  return {
    ...fields(proxy, ['source', 'label', 'protocol', 'host', 'port', 'total', 'enabled', 'available']),
    ...fields(config, ['proxyId', 'all']),
    proxyIds: strings(config.proxyIds), rotate: fields(proxy.rotate, ['all', 'total', 'usable']),
    error: diagnosticText(proxy.error, t),
  }
}

export function usageSummary(value: unknown, t: Translate) {
  const usage = object(value)
  return {
    ...fields(usage, ['totalLeft', 'planLeft', 'bonusLeft', 'available', 'availableView', 'unit', 'unlimited', 'empty']),
    emptyText: diagnosticText(usage.emptyText, t),
    statisticsError: diagnosticText(usage.statisticsError, t),
    benefitError: diagnosticText(usage.benefitError, t),
    wallets: page(rows(usage.wallets).map(wallet => fields(wallet, [
      'type', 'displayName', 'balance', 'balanceView', 'unit', 'total', 'used', 'remaining',
    ])), 0, 10, rows(usage.wallets).length, 2000),
    windows: page(rows(usage.windows).map(window => fields(window, ['key', 'label', 'remainingPercent', 'resetsAt'])), 0, 10, rows(usage.windows).length, 2000),
  }
}

export function accountReadTools(api: AssistantReadApi, t: Translate): AssistantReadTool[] {
  const listAccounts: AssistantReadTool = {
    id: 'list_accounts', label: t('List accounts'), kind: 'read',
    desc: 'Read the list of accounts visible on this machine: exact account ID, display name, provider, enabled state, global priority, credential presence, and whether a re-login is needed. Call this before answering "which accounts are there", "which account is disabled", or "which account is rate-limited right now". Disabled or credential-less accounts still appear in the list; accounts hidden because a plugin is unavailable show only a count and reason, not deletion. rateLimited only means the upstream rate-limit cooldown has not ended — it is not a local limiter hit. position is the configured order, not a guarantee that real requests use it; for actual candidates use get_routing_queue. Pagination: when hasMore is true, keep reading with nextOffset; do not draw conclusions from a single page.",',
    params: {
      type: 'object', additionalProperties: false,
      properties: {
        provider: { type: 'string', description: 'Provider ID; leave blank= All' },
        keyword: { type: 'string', description: 'Account ID or a substring of the page display name; leave blank= All' },
        enabled: { type: 'boolean', description: 'true = Enabled, false= Disabled; leave blank= All' },
        rateLimited: { type: 'boolean', description: 'true = Still in the upstream rate-limit cooldown, false= Not in cooldown; leave blank= All' },
        ...PAGE_PARAMS,
      },
    },
    run: async args => {
      const provider = stringArg(args, 'provider', t, 128).toLowerCase()
      const keyword = stringArg(args, 'keyword', t).toLowerCase()
      const enabled = booleanArg(args, 'enabled', t)
      const limited = booleanArg(args, 'rateLimited', t)
      const { offset, limit } = pageArgs(args, t)
      const { snapshot, accounts } = accountsOf(await api.accounts(), t)
      const now = Date.now()
      const filtered = accounts.filter(account => {
        if (provider && text(account.provider).toLowerCase() !== provider) return false
        if (keyword && ![text(account.id), accountName(account)].join(' ').toLowerCase().includes(keyword)) return false
        if (enabled !== undefined && account.enabled !== enabled) return false
        return limited === undefined || (activeRateLimits(account, now, t).length > 0) === limited
      })
      const data = page(filtered.slice(offset).map(account => accountSummary(account, t, now)), offset, limit, filtered.length)
      return result({ asOf: now, visibleTotal: accounts.length, ...accountMeta(snapshot), accounts: data }, t('{n} accounts', { n: filtered.length }))
    },
  }

  const getAccountDetail: AssistantReadTool = {
    id: 'get_account_detail', label: t('Get account details'), kind: 'read',
    desc: 'Read the public configuration and local cached state of a given account: credential-validity fields, re-login reasons, the upstream rate limit currently in effect, active connections and the concurrency cap, proxy binding summary, automatic balance-query settings, balance snapshot, quota sources, and limiter rules/hit readings. First get the account ID from list_accounts; when a display name is not unique, add provider or use the ID. This is read-only on the local cache — it does not refresh credentials and does not send a live balance query. A missing reading in the cache means unknown, not zero balance. The credential-expiry field may only indicate a short-lived access token; do not conclude that the long-lived credential has expired from that field alone — needsRelogin is the actual flag for a detected re-login need. Use section Default for the overview; use the corresponding section with pagination to see the full rate limits or limiters; when hasMore is set, keep fetching.",',
    params: {
      type: 'object', additionalProperties: false,
      properties: {
        account: { type: 'string', description: 'Account ID or the uniquely matching page display name; prefer the ID' },
        provider: { type: 'string', description: 'Restrict to a provider ID, used to disambiguate accounts with the same name; leave blank= All' },
        section: { type: 'string', enum: ['overview', 'rateLimits', 'limiters', 'quotaSources'], description: 'overview = configuration and cache summary; the rest read the paged details of that segment' },
        ...PAGE_PARAMS,
      },
      required: ['account'],
    },
    run: async args => {
      const ref = stringArg(args, 'account', t)
      const provider = stringArg(args, 'provider', t, 128)
      const section = enumArg(args, 'section', ['overview', 'rateLimits', 'limiters', 'quotaSources'], 'overview', t)
      const { offset, limit } = pageArgs(args, t)
      const { snapshot, accounts } = accountsOf(await api.accounts(), t)
      const account = findAccount(accounts, ref, provider, t)
      const id = text(account.id)
      const now = Date.now()
      const payload: Record<string, unknown> = { asOf: now, account: accountSummary(account, t, now) }
      const rateLimits = activeRateLimits(account, now, t)
      if (section === 'rateLimits') {
        payload.rateLimits = page(rateLimits.slice(offset), offset, limit, rateLimits.length)
      } else {
        const cache = expectObject(await api.usageSnapshot(), 'usageSnapshot', t)
        const usageRows = expectRows(cache, 'results', 'usageSnapshot.results', t)
        const statesByAccount = expectObject(cache.limiterStates, 'usageSnapshot.limiterStates', t)
        const sourcesByAccount = expectObject(cache.quotaSources, 'usageSnapshot.quotaSources', t)
        const states = expectRows(statesByAccount, id, 'usageSnapshot.limiterStates.account', t)
        const groups = expectRows(account, 'limiterGroups', 'account.limiterGroups', t)
        const sources = expectRows(sourcesByAccount, id, 'usageSnapshot.quotaSources.account', t)
        payload.cachedAt = number(cache.tokenAt)
        if (section === 'quotaSources') {
          payload.quotaSources = page(sources.slice(offset).map(quota), offset, limit, sources.length)
        } else if (section === 'limiters') {
          payload.limiters = page(groups.slice(offset).map((group, index) => limiterGroup(group, object(states[offset + index]), t)), offset, limit, groups.length)
        } else {
          const connections = expectObject(await api.connections(), 'connections', t)
          const counts = expectObject(connections.counts, 'connections.counts', t)
          const usageRow = usageRows.find(row => text(row.id) === id)
          payload.account = {
            ...object(payload.account),
            ...fields(account, ['expiresAt', 'tokenExpiresAt', 'tokenExpireAt', 'hasRefreshToken', 'canRefresh', 'maxConcurrent', 'source', 'edition', 'zcodePlan']),
            needsReloginReason: diagnosticText(account.needsReloginReason, t),
            currentAccount: snapshot.currentAccountId === id,
            connections: number(counts[id]) ?? 0,
            proxy: proxySummary(account.proxy, t),
            usageQuery: fields(account.usageQuery, ['enabled', 'interval', 'retries']),
            lowBalance: fields(account.lowBalance, ['mode', 'threshold']),
          }
          payload.balance = usageRow ? {
            cached: true, at: number(usageRow.at), ...fields(usageRow, ['code']),
            error: diagnosticText(usageRow.error, t), usage: usageRow.usage ? usageSummary(usageRow.usage, t) : null,
            lastSuccessAt: number(usageRow.lastSuccessAt),
            lastUsage: usageRow.lastUsage ? usageSummary(usageRow.lastUsage, t) : null,
          } : { cached: true, at: null, usage: null }
          payload.rateLimits = page(rateLimits, 0, 10)
          payload.limiterSummary = {
            rules: groups.length, statesKnown: true,
            matched: states.map((state, index) => ({ index, hit: state.hit === true, account: state.account === true, models: strings(state.models) })).filter(state => state.hit),
            quotaSources: sources.length,
          }
        }
      }
      return result(payload, safeText(accountName(account)))
    },
  }

  return [listAccounts, getAccountDetail]
}
