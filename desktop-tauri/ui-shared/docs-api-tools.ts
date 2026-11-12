import { t } from '../ui-kit/src/i18n'
import { buildWebSearchApiDoc } from './tools-api-doc'
import type { ApiSection } from './docs-api-content'

/**
 * REST 。
 *
 * 「」 Skill / MCP can ； REST API is 
 * ， not and ， in this 。 use `tools-api-doc` in 
 * （ its ）：、 up ， not 。
 */
export function toolApiSections(origin: string): ApiSection[] {
  const locale = (window as unknown as { wbI18n?: { locale(): string } }).wbI18n?.locale() || 'zh-Hans'
  return [{
    id: 'tools-api',
    title: t('工具服务 API'),
    path: 'POST /api/web-search/search · POST /api/web-search/fetch',
    description: t('网络搜索与网页抓取的 REST 接口，使用网关 Key 鉴权；不属于 /v1 对话协议。'),
    paragraphs: [
      t('工具服务 API 是网关对外的固定接口，路径以 /api/web-search 开头；不属于 /v1 对话协议，也不是管理面板的管理接口，鉴权同样使用网关 Key。'),
    ],
    markdown: buildWebSearchApiDoc(locale, origin),
  }]
}
