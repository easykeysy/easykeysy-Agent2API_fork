import { t } from '../ui-kit/src/i18n'

export const API_ENDPOINTS = [
  { id: 'api-base', method: '', path: '/v1' },
  { id: 'api-chat', method: 'POST', path: '/v1/chat/completions' },
  { id: 'api-responses', method: 'POST', path: '/v1/responses' },
  { id: 'api-messages', method: 'POST', path: '/v1/messages' },
  { id: 'api-count-tokens', method: 'POST', path: '/v1/messages/count_tokens' },
  { id: 'api-image-generations', method: 'POST', path: '/v1/images/generations' },
  { id: 'api-image-edits', method: 'POST', path: '/v1/images/edits' },
  { id: 'api-image-files', method: 'GET', path: '/v1/images/files/{id}' },
  { id: 'api-models', method: 'GET', path: '/v1/models' },
  { id: 'api-health', method: 'GET', path: '/health' },
] as const

export function apiEndpoints() {
  const chat = t('Chat protocols and context budgets')
  const images = t('Image generation and editing')
  const probes = t('Model list and health checks')
  const labels: Record<typeof API_ENDPOINTS[number]['id'], { name: string; note: string; group: string }> = {
    'api-base': {
      name: 'Base URL', group: '',
      note: t('Use this address for OpenAI-compatible clients; the official Anthropic SDK uses the gateway root address described below.'),
    },
    'api-chat': {
      name: 'OpenAI Chat Completions', group: chat,
      note: t('The default protocol for most OpenAI-compatible clients'),
    },
    'api-responses': {
      name: 'OpenAI Responses', group: chat,
      note: t('Codex / newer OpenAI SDKs; stateless forwarding with client-supplied history.'),
    },
    'api-messages': {
      name: 'Anthropic Messages', group: chat,
      note: t('Claude Code / Anthropic SDK; shares models and the account pool with the OpenAI endpoints.'),
    },
    'api-count-tokens': {
      name: t('Token count estimate'), group: chat,
      note: t('Local context budget estimate, returning estimated=true; not an exact upstream token count.'),
    },
    'api-image-generations': {
      name: t('Image generation'), group: images,
      note: t('Generate images from text; send JSON and receive non-streaming base64 data or download URLs.'),
    },
    'api-image-edits': {
      name: t('Image editing'), group: images,
      note: t('Reference-image editing; accepts multipart file uploads or JSON image references.'),
    },
    'api-image-files': {
      name: t('Temporary image download'), group: images,
      note: t('Only for temporary gateway URLs returned in responses; replace {id} with the actual download token.'),
    },
    'api-models': {
      name: t('List models'), group: probes,
      note: t('Authentication is not required; a valid Key filters the list to models that Key is authorized to use.'),
    },
    'api-health': {
      name: t('Health check'), group: probes,
      note: t('Authentication is not required; checks gateway reachability, not model or quota availability.'),
    },
  }
  return API_ENDPOINTS.map(endpoint => ({ ...endpoint, ...labels[endpoint.id] }))
}
