import type { ToolArgs, Translate } from './assistant-read-types'
import { accountName, accountsOf, booleanArg, diagnosticText, expectObject, expectRows, fields, findAccount, number, object, page, pageArgs, PAGE_PARAMS, result, stringArg } from './assistant-read-data'
import { usageSummary } from './assistant-read-accounts'
import type { ImageToolContext } from './assistant-image-tools'
import { checkToolSignal } from './assistant-tool-execution'

type BalanceApi = {
  accounts: () => Promise<unknown>
  usageSnapshot: () => Promise<unknown>
  refreshBalance: (id: string) => Promise<unknown>
}

export function createBalanceTool(api: BalanceApi, t: Translate): {
  id: string; label: string; kind: 'read'; desc: string; params: Record<string, unknown>
  run: (args: ToolArgs, context?: ImageToolContext) => Promise<{ text: string; summary: string }>
} {
  return {
    id: 'list_account_balance', label: t('Query account balance'), kind: 'read',
    desc: 'Paginated read of the account\'s most recent cached balance and quota; unqueried is never treated as zero. Default is snapshot-only; refresh",=true Refreshes only this page\'s accounts live. It requests the upstream and may trigger the configured auto-disable rule for low balance, and confirmation is required per the permission mode. Each call returns total/hasMore/nextOffset; hasMore",=true keep paging then; paging does not refresh by default.',
    params: {
      type: 'object', additionalProperties: false,
      properties: {
        account: { type: 'string', description: 'Account ID or the unique display name; leave blank for all' },
        provider: { type: 'string', description: 'Provider ID, used to filter and disambiguate accounts with the same name' },
        refresh: { type: 'boolean', description: 'true Refresh this page in real time; default false, only reads the cache' },
        ...PAGE_PARAMS,
      },
    },
    run: async (args, context) => {
      checkToolSignal(context)
      const ref = stringArg(args, 'account', t)
      const provider = stringArg(args, 'provider', t)
      const refresh = booleanArg(args, 'refresh', t) === true
      const { offset, limit } = pageArgs(args, t)
      const { accounts } = accountsOf(await api.accounts(), t)
      const selected = ref ? [findAccount(accounts, ref, provider, t)]
        : accounts.filter(account => !provider || String(account.provider).toLowerCase() === provider.toLowerCase())
      const selectedPage = page(selected.slice(offset).map(account => fields(account, ['id'])), offset, limit, selected.length, 3000)
      if (refresh) {
        for (const item of selectedPage.items) {
          checkToolSignal(context)
          await api.refreshBalance(String(item.id))
        }
      }
      const snapshot = expectObject(await api.usageSnapshot(), 'usageSnapshot', t)
      const cache = expectRows(snapshot, 'results', 'usageSnapshot.results', t)
      const entries = selected.slice(offset, offset + selectedPage.returned).map(account => {
        const row = cache.find(row => row.id === account.id)
        return {
          ...fields(account, ['id', 'provider', 'enabled']), name: accountName(account),
          cached: !refresh, queried: Boolean(row), at: row ? number(row.at) : null,
          ...fields(row, ['code']), error: diagnosticText(row?.error, t),
          usage: row?.usage ? usageSummary(object(row.usage), t) : null,
          lastSuccessAt: row ? number(row.lastSuccessAt) : null,
          lastUsage: row?.lastUsage ? usageSummary(object(row.lastUsage), t) : null,
        }
      })
      return result({ asOf: Date.now(), accounts: page(entries, offset, limit, selected.length), refreshed: refresh }, t('{n} accounts', { n: selected.length }))
    },
  }
}
