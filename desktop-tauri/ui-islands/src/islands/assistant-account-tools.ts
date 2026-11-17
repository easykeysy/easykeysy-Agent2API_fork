import { t } from '../i18n'
import type { AssistantTool } from './assistant-tools'
import { accountName, accountsOf, findAccount as resolveAccount } from '../../../ui-shared/assistant-read-data'
import { checkToolSignal, providerGroups, runToolBatch } from '../../../ui-shared/assistant-tool-execution'

type AccountRow = Record<string, unknown>
type AccountBridge = {
  getAccounts: () => Promise<unknown>
  updateAccount: (id: string, patch: Record<string, unknown>) => Promise<unknown>
  reorderAccounts: (queue: string[]) => Promise<unknown>
}

function bridge(): AccountBridge {
  const api = (window as unknown as { workbuddyDesktop?: AccountBridge }).workbuddyDesktop
  if (!api) throw new Error(t('Desktop bridge unavailable (this page must be opened inside the Agent2API client)'))
  return api
}

function str(value: unknown, max = 256): string {
  return typeof value === 'string' ? value.trim().slice(0, max) : ''
}
function num(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined
}
function list(value: unknown): AccountRow[] {
  return Array.isArray(value) ? value.filter((item): item is AccountRow => Boolean(item) && typeof item === 'object' && !Array.isArray(item)) : []
}
function refreshApp(): void {
  const app = (window as unknown as { wbApp?: { refresh?: () => void } }).wbApp
  void app?.refresh?.()
}

export async function readAccounts(): Promise<AccountRow[]> {
  const read = bridge().getAccounts
  if (typeof read !== 'function') throw new Error(t('This client is missing the read interface: {name}', { name: 'getAccounts' }))
  return accountsOf(await read(), t).accounts
}

export function accountLabel(account: AccountRow): string {
  return accountName(account) || t('Unnamed account')
}

export function findAccount(accounts: AccountRow[], ref: string): AccountRow {
  return resolveAccount(accounts, ref, '', t)
}

const setProviderOrder: AssistantTool = {
  id: 'set_provider_order', label: t('Reorder providers'), kind: 'write',
  desc: 'Adjusts the **whole-group position** (which group number) of a given provider in the account global queue, deciding which provider is tried first at routing time. All of that provider\'s accounts move as one block. Positions are counted from 1.",',
  params: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: 'Provider id' },
      position: { type: 'number', description: 'The target group position (1= at the very front)' },
    },
    required: ['provider', 'position'],
  },
  plan: args => ({
    lead: t('About to move the whole "{provider}" group to position {position}:', {
      provider: str(args.provider) || t('(not specified)'), position: String(num(args.position) ?? '?'),
    }),
    note: t('The queue is submitted as a whole table at once (the backend renumbers priorities in the new order), so a move also shifts the relative order of other groups.'),
  }),
  run: async (args, context) => {
    checkToolSignal(context)
    const provider = str(args.provider).toLowerCase()
    const position = num(args.position)
    if (!provider || position === undefined) throw new Error(t('Incomplete parameters (provider / position required)'))
    const accounts = await readAccounts()
    const groups = providerGroups(accounts)
    const from = groups.findIndex(group => group.provider === provider)
    if (from < 0) throw new Error(t('No accounts found for provider "{provider}"', { provider }))
    const to = Math.max(0, Math.min(groups.length - 1, Math.round(position) - 1))
    const [moved] = groups.splice(from, 1)
    groups.splice(to, 0, moved)
    checkToolSignal(context)
    await bridge().reorderAccounts(groups.flatMap(group => group.ids))
    refreshApp()
    return {
      text: JSON.stringify({ ok: true, provider, from: from + 1, to: to + 1, moved: moved.ids.length }),
      summary: t('Moved to position {n}', { n: to + 1 }),
    }
  },
}

const setAccountOrder: AssistantTool = {
  id: 'set_account_order', label: t('Adjust account priority'), kind: 'write',
  desc: 'Move a given account to position N in the account global queue (1= at the very front). Position decides the primary/backup and failover order: the earlier, the sooner it is used.',
  params: {
    type: 'object',
    properties: {
      account: { type: 'string', description: 'The account name or account id' },
      position: { type: 'number', description: 'The target position (1= at the very front)' },
    },
    required: ['account', 'position'],
  },
  plan: args => ({
    lead: t('About to move account "{account}" to position {position}:', {
      account: str(args.account) || t('(not specified)'), position: String(num(args.position) ?? '?'),
    }),
    note: t('The queue is submitted as a whole table at once; the earlier the position, the sooner it is picked when routing.'),
  }),
  run: async (args, context) => {
    checkToolSignal(context)
    const ref = str(args.account, 128)
    const position = num(args.position)
    if (!ref || position === undefined) throw new Error(t('Incomplete parameters (account / position required)'))
    const accounts = await readAccounts()
    const target = findAccount(accounts, ref)
    const id = str(target.id)
    const ids = accounts.map(row => str(row.id)).filter(Boolean)
    const from = ids.indexOf(id)
    if (from < 0) throw new Error(t('The account is not in the current queue'))
    ids.splice(from, 1)
    const to = Math.max(0, Math.min(ids.length, Math.round(position) - 1))
    ids.splice(to, 0, id)
    checkToolSignal(context)
    await bridge().reorderAccounts(ids)
    refreshApp()
    return {
      text: JSON.stringify({ ok: true, account: accountLabel(target), from: from + 1, to: to + 1 }),
      summary: t('Moved to position {n}', { n: to + 1 }),
    }
  },
}

const setAccountEnabled: AssistantTool = {
  id: 'set_account_enabled', label: t('Enable / disable accounts'), kind: 'write',
  desc: 'Turns the forwarding eligibility of one or more accounts on or off. Disabling only removes them from the queue (balance can still be queried), but **if a line is left with only that account**, disabling cuts it off outright. Several can be changed at once.",',
  params: {
    type: 'object',
    properties: {
      items: {
        type: 'array', description: 'The account list to change',
        items: {
          type: 'object',
          properties: {
            account: { type: 'string', description: 'The account name or account id' },
            enabled: { type: 'boolean', description: 'true = Enable, false= Disable' },
          },
          required: ['account', 'enabled'],
        },
      },
    },
    required: ['items'],
  },
  plan: args => {
    const items = list(args.items)
    return {
      lead: t('About to change the enabled state of {n} accounts (disabling {k}):', { n: items.length, k: items.filter(item => item.enabled === false).length }),
      table: {
        head: [t('Accounts'), t('Change to')],
        rows: items.map(item => [str(item.account), item.enabled === false ? t('Disable') : t('enabled')]),
      },
      note: t('Disabling removes the account from the forwarding queue; if this route has no other usable account, requests will fail outright.'),
    }
  },
  run: async (args, context) => {
    checkToolSignal(context)
    const api = bridge()
    const accounts = await readAccounts()
    const result = await runToolBatch(list(args.items), context, async item => {
      if (typeof item.enabled !== 'boolean') throw new Error(t('Incomplete parameters (account / enabled required)'))
      const target = findAccount(accounts, str(item.account, 128))
      await api.updateAccount(str(target.id), { enabled: item.enabled })
      return `${item.enabled ? 'enabled' : 'Disable'} ${accountLabel(target)}`
    }, t)
    refreshApp()
    return result
  },
}

export const ASSISTANT_ACCOUNT_TOOLS: AssistantTool[] = [setProviderOrder, setAccountOrder, setAccountEnabled]
