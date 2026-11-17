import type { AssistantReadApi, AssistantReadTool, Translate } from './assistant-read-types'
import {
  accountMeta, accountsOf, diagnosticText, enumArg, expectObject, expectRows, fields, number, object, page, pageArgs,
  PAGE_PARAMS, result, rows, stringArg,
} from './assistant-read-data'

function routeEntry(value: Record<string, unknown>, t: Translate) {
  return {
    ...fields(value, ['id', 'name', 'provider', 'providerLabel', 'state', 'resetAt', 'priority']),
    reason: diagnosticText(value.reason, t),
  }
}

function dimension(value: Record<string, unknown>) {
  return fields(value, ['id', 'model', 'label', 'requests', 'success', 'failures', 'totalTokens', 'tokens', 'percentage',
    'successRate', 'promptTokens', 'completionTokens', 'cacheReadTokens', 'cacheRate', 'lastUsedAt',
    'tokenDetailsComplete', 'source', 'filterValue', 'state'])
}

function dateArg(args: Record<string, unknown>, key: string, t: Translate): string {
  const value = stringArg(args, key, t, 10)
  const date = new Date(`${value}T00:00:00Z`)
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || !Number.isFinite(date.getTime()) || date.toISOString().slice(0, 10) !== value) {
    throw new Error(t('Parameter {name} must be a valid date (YYYY-MM-DD)', { name: key }))
  }
  return value
}

export function reportReadTools(api: AssistantReadApi, t: Translate): AssistantReadTool[] {
  const getRoutingQueue: AssistantReadTool = {
    id: 'get_routing_queue', label: t('Inspect routing queue'), kind: 'read',
    desc: 'Read, for a given model (external model name / mapping alias), the candidate accounts under the current policy and the skipped accounts with reasons. Call this before answering "why wasn\'t this account used", "what other fallback routes exist", or "who can serve this model". When model is empty, the global queue that is not bound to a model is read. The result is a static rehearsal: it does not consume the rotation cursor and does not change session stickiness. Real requests are also affected by rotation/stickiness and state changes, so pickedId is not guaranteed to be the account for the next real request. ordered lists the candidates to try; blocked lists the skipped accounts; modelKnown",=false  means the model is not in the current routing catalog; the returned content must not be treated as available candidates for that model. Pagination: when hasMore is set, keep fetching with nextOffset.",',
    params: {
      type: 'object', additionalProperties: false,
      properties: {
        model: { type: 'string', description: 'The outward model name or mapping alias; leave blank= Global queue' },
        section: { type: 'string', enum: ['ordered', 'blocked', 'both'], description: 'ordered = candidates, blocked= Skipped, both= two segments (default); each segment paginates independently' },
        ...PAGE_PARAMS,
      },
    },
    run: async args => {
      const model = stringArg(args, 'model', t, 128)
      const section = enumArg(args, 'section', ['ordered', 'blocked', 'both'], 'both', t)
      const { offset, limit } = pageArgs(args, t)
      const [queue, accountData] = await Promise.all([api.routingQueue(model || undefined), api.accounts()])
      const data = expectObject(queue, 'routingQueue', t)
      const { snapshot, accounts } = accountsOf(accountData, t)
      const visible = new Set(accounts.map(account => String(account.id)))
      const ordered = expectRows(data, 'ordered', 'routingQueue.ordered', t).filter(row => visible.has(String(row.id)))
      const blocked = expectRows(data, 'blocked', 'routingQueue.blocked', t).filter(row => visible.has(String(row.id)))
      const payload: Record<string, unknown> = {
        asOf: Date.now(), preview: true,
        ...fields(data, ['strategy', 'sticky', 'model', 'modelKnown', 'lastRequestModel']),
        pickedId: fields(ordered[0], ['id']).id ?? '', hiddenAccounts: accountMeta(snapshot).hiddenAccounts,
      }
      if (section !== 'blocked') payload.ordered = page(ordered.slice(offset).map(row => routeEntry(row, t)), offset, limit, ordered.length, 9000)
      if (section !== 'ordered') payload.blocked = page(blocked.slice(offset).map(row => routeEntry(row, t)), offset, limit, blocked.length, 9000)
      return result(payload, t('{n} candidates, {m} skipped', { n: ordered.length, m: blocked.length }))
    },
  }

  const getUsageReport: AssistantReadTool = {
    id: 'get_usage_report', label: t('Read usage report'), kind: 'read',
    desc: 'Read the local production request report before answering "how many tokens did I use today", "which account consumed the most this week", "what is the success rate", or "what is the usage trend". Call this before any of those questions. Date boundaries for range follow the gateway machine\'s local calendar days; a custom range includes both endpoint days. Assistant conversations and model tests are excluded by backend convention — this is not the total upstream consumption ledger. For balance use list_account_balance.summary: the default gives an overview plus the top 10 of each dimension (with pagination metadata); for a complete ranking, pick providers/accounts/models/clients/apiKeys and paginate. apiKeyDetail queries that key\'s client and model distribution via filterValue and never returns the key value. dailyTrend/cacheTrend24h are available for trends on demand. successRate Yes 0",~1 ratio; model statistics use the actual upstream model name (older rows fall back to the requested model), not the mapping alias.',
    params: {
      type: 'object', additionalProperties: false,
      properties: {
        range: { type: 'string', enum: ['today', 'yesterday', '7', '30', 'month', 'all', 'custom'], description: 'Default today; custom must carry startDate/endDate' },
        startDate: { type: 'string', description: 'Custom range start date, YYYY-MM-DD (inclusive)' },
        endDate: { type: 'string', description: 'Custom range end date, YYYY-MM-DD (inclusive)' },
        section: { type: 'string', enum: ['summary', 'providers', 'accounts', 'models', 'clients', 'apiKeys', 'apiKeyDetail', 'dailyTrend', 'cacheTrend24h'], description: 'Default summary; other segments can be paged and read in full' },
        apiKey: { type: 'string', description: 'apiKeyDetail Required: use the filterValue from an apiKeys row (key:<id>/env/none/unknown), the key value is not sent' },
        ...PAGE_PARAMS,
      },
    },
    run: async args => {
      const range = enumArg(args, 'range', ['today', 'yesterday', '7', '30', 'month', 'all', 'custom'], 'today', t)
      const section = enumArg(args, 'section', ['summary', 'providers', 'accounts', 'models', 'clients', 'apiKeys', 'apiKeyDetail', 'dailyTrend', 'cacheTrend24h'], 'summary', t)
      const { offset, limit } = pageArgs(args, t)
      const dates = range === 'custom' ? { start: dateArg(args, 'startDate', t), end: dateArg(args, 'endDate', t) } : undefined
      if (dates && dates.start > dates.end) throw new Error(t('The start date cannot be later than the end date'))
      const apiKey = stringArg(args, 'apiKey', t, 128)
      if (section === 'apiKeyDetail' && !apiKey) throw new Error(t('Specify an API key identity to view its details'))
      const data = expectObject(await api.usageReport(range, dates, apiKey ? { apiKey } : undefined), 'usageReport', t)
      const overview = expectObject(data.overview, 'usageReport.overview', t)
      if (number(overview.requests) === null || number(overview.successful) === null || number(overview.tokens) === null) {
        throw new Error(t('Incomplete response: {name}', { name: 'usageReport.overview' }))
      }
      const requests = number(overview.requests) as number
      const successful = number(overview.successful) as number
      const payload: Record<string, unknown> = {
        asOf: Date.now(), excludesAssistantAndTests: true, calendar: 'gateway-local',
        ...fields(data, ['range', 'startDate', 'endDate']),
        overview: {
          ...fields(overview, ['requests', 'successful', 'tokens', 'activeDays', 'streak']),
          successRate: requests ? successful / requests : 0,
          topModel: overview.topModel ? dimension(object(overview.topModel)) : null,
        },
      }
      if (section === 'summary') {
        for (const key of ['providers', 'accounts', 'models', 'clients', 'apiKeys']) {
          const items = expectRows(data, key, `usageReport.${key}`, t)
          payload[key] = page(items.map(dimension), 0, 10, items.length, 3000)
        }
        const rates: Record<string, unknown> = {}
        const source = expectObject(data.cacheRates, 'usageReport.cacheRates', t)
        for (const key of ['last10m', 'last1h', 'last24h', 'last7d']) {
          if (source[key] !== undefined) rates[key] = fields(source[key], ['hitTokens', 'inputTokens', 'rate'])
        }
        payload.cacheRates = rates
      } else if (section === 'apiKeyDetail') {
        const detail = expectObject(data.apiKeyDetail, 'usageReport.apiKeyDetail', t)
        payload.apiKeyDetail = { ...fields(detail, ['filterValue', 'label', 'requests', 'totalTokens', 'detailsComplete']) }
        for (const key of ['clients', 'models']) {
          const items = expectRows(detail, key, `usageReport.apiKeyDetail.${key}`, t)
          object(payload.apiKeyDetail)[key] = page(items.slice(offset).map(dimension), offset, limit, items.length)
        }
      } else {
        const items = expectRows(data, section, `usageReport.${section}`, t)
        const projected = items.slice(offset).map(item => {
          if (section !== 'dailyTrend' && section !== 'cacheTrend24h') return dimension(item)
          const models = rows(item.models)
          return {
            ...fields(item, ['date', 'hour', 'tokens', 'requests', 'totalTokens', 'hitTokens', 'inputTokens', 'rate']),
            models: page(models.map(dimension), 0, 10, models.length, 2000),
            ...(section === 'dailyTrend' ? {
              clients: page(rows(item.clients).map(dimension), 0, 10, rows(item.clients).length, 2000),
              apiKeys: page(rows(item.apiKeys).map(dimension), 0, 10, rows(item.apiKeys).length, 2000),
            } : {}),
          }
        })
        payload[section] = page(projected, offset, limit, items.length)
      }
      return result(payload, t('{requests} requests, {tokens} tokens', { requests, tokens: number(overview.tokens) as number }))
    },
  }

  return [getRoutingQueue, getUsageReport]
}
