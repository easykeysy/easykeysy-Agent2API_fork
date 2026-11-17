import { accountReadTools } from './assistant-read-accounts'
import { logReadTools } from './assistant-read-logs'
import { reportReadTools } from './assistant-read-report'
import { diagnosticText } from './assistant-read-data'
import type { AssistantReadApi, AssistantReadTool, ToolArgs, Translate } from './assistant-read-types'

export type { AssistantReadApi, AssistantReadTool, QueryParams } from './assistant-read-types'

export function createAssistantReadTools(api: AssistantReadApi, t: Translate): AssistantReadTool[] {
  return [...accountReadTools(api, t), ...reportReadTools(api, t), ...logReadTools(api, t)].map(tool => ({
    ...tool,
    run: async (args: ToolArgs) => {
      try {
        return await tool.run(args)
      } catch (error) {
        throw new Error(diagnosticText(error instanceof Error ? error.message : String(error), t))
      }
    },
  }))
}

export function assistantReadMeta(t: Translate) {
  return [
    { id: 'list_accounts', label: t('List accounts'), desc: t('Reads accounts, enabled states, priorities and reasons for hidden accounts, with filtering and pagination.') },
    { id: 'get_account_detail', label: t('Get account details'), desc: t('Reads a local snapshot of credentials, rate limits, connections, proxy settings and limiters.') },
    { id: 'get_routing_queue', label: t('Inspect routing queue'), desc: t('Shows candidate accounts, skip reasons and the current routing strategy for a model.') },
    { id: 'get_usage_report', label: t('Read usage report'), desc: t('Reads token usage, success rates, rankings and trends for production requests.') },
    { id: 'list_request_logs', label: t('Read request logs'), desc: t('Reads request status, latency, accounts, errors and retries without reading conversation content.') },
    { id: 'list_runtime_logs', label: t('Read runtime logs'), desc: t('Reads redacted summaries of runtime events, including credentials, plugins and scheduled tasks.') },
  ]
}
