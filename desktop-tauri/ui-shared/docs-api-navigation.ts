import { t } from '../ui-kit/src/i18n'
import type { ApiSection } from './docs-api-content'
import { apiEndpoints } from './docs-api-endpoints'
import { documentText, type PluginProviderDocumentation } from './provider-docs'

export type DocumentationEntry = {
  id: string
  title: string
  group: string
  description: string
  path?: string
  searchText: string[]
}

export type DocumentationGroup = { id: string; title: string; entries: DocumentationEntry[] }

export function documentationGroups(sections: ApiSection[], packs: PluginProviderDocumentation[]): DocumentationGroup[] {
  const addresses: DocumentationEntry = {
    id: 'addresses', title: t('API address'), group: '',
    description: t('View the current gateway\'s client addresses and endpoint paths, ready to copy and use.'),
    searchText: apiEndpoints().flatMap(endpoint => [endpoint.name, endpoint.path, endpoint.note, endpoint.group]),
  }
  const entries: DocumentationEntry[] = [addresses, ...sections.map(section => ({
    id: section.id, title: section.title, group: '', description: section.description, path: section.path,
    searchText: [section.path || '', ...(section.paragraphs || []), section.warning || '', ...(section.points || []),
      ...(section.fields?.flatMap(field => [field.name, field.type, field.description]) || []),
      ...(section.examples?.flatMap(example => [example.label, example.code]) || []),
      section.markdown || ''],
  }))]
  const definitions = [
    { id: 'guide', title: t('Getting started'), ids: ['addresses', 'start', 'models'] },
    { id: 'chat', title: t('Chat protocols'), ids: ['chat', 'tools', 'responses', 'messages'] },
    { id: 'images', title: t('Image generation and editing'), ids: ['image-generation', 'image-editing', 'image-results', 'image-sdk'] },
    { id: 'tools', title: t('Tool services API'), ids: ['tools-api'] },
    { id: 'troubleshooting', title: t('Troubleshooting and limits'), ids: ['errors'] },
  ]
  const groups = definitions.map(definition => ({
    id: definition.id, title: definition.title,
    entries: definition.ids.flatMap(id => {
      const entry = entries.find(item => item.id === id)
      return entry ? [{ ...entry, group: definition.title }] : []
    }),
  }))
  const assignedIds = new Set(groups.flatMap(group => group.entries.map(entry => entry.id)))
  const remaining = entries.filter(entry => !assignedIds.has(entry.id))
  if (remaining.length) {
    groups.push({
      id: 'reference', title: t('Detailed API reference'), entries: remaining.map(entry => ({ ...entry, group: t('Detailed API reference') })),
    })
  }
  if (packs.length) {
    const locale = (window as unknown as { wbI18n?: { locale(): string } }).wbI18n?.locale() || 'zh-Hans'
    groups.push({
      id: 'providers', title: t('Provider capabilities'), entries: [{
        id: 'providers', title: t('Provider capability matrix'), group: t('Provider capabilities'),
        description: t('Capabilities are declared by the current gateway or plugin version. They describe integration support, not account quota or current availability.'),
        searchText: packs.flatMap(pack => [pack.name, documentText(pack.description, locale),
          ...pack.providers.flatMap(provider => [provider.id, provider.label,
            ...provider.notes.map(note => documentText(note, locale)),
            ...Object.values(provider.capabilities).map(capability => documentText(capability.note, locale))])]),
      }],
    })
  }
  return groups
}

export function documentationMatches(entry: DocumentationEntry, query: string): boolean {
  const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean)
  if (!terms.length) return true
  const text = [entry.title, entry.group, entry.description, ...entry.searchText].join('\n').toLowerCase()
  return terms.every(term => text.includes(term))
}
