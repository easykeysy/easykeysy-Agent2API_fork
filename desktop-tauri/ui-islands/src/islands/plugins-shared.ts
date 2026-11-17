import { t } from '../i18n'
import { githubErrorMessage } from '../../../ui-shared/updates'
import type { PluginMutation } from './plugins-types'

export type PluginBridge = {
  __TAURI_INTERNALS__?: {
    invoke?: (command: string, args?: unknown) => Promise<unknown>
  }
  wbApp?: {
    toast?: (message: string, kind?: 'err' | 'ok') => void
    refresh?: () => Promise<boolean>
    refreshPluginPages?: (payload?: PluginMutation) => Promise<void> | void
    readonly currentPage?: string
  }
}

export function bridge(): PluginBridge {
  return window as unknown as PluginBridge
}

export function errorMessage(error: unknown): string {
  return githubErrorMessage(error, t, 'plugin')
}

export function toast(message: string, kind?: 'err' | 'ok'): void {
  bridge().wbApp?.toast?.(errorMessage(message), kind)
}

export async function api<T>(method: string, path: string, body?: unknown): Promise<T> {
  const invoke = bridge().__TAURI_INTERNALS__?.invoke
  if (!invoke) throw new Error(t('Desktop runtime unavailable (Tauri not initialized)'))
  const result = await invoke('api_request', {
    request: { method, path, body: body === undefined ? null : body },
  })
  if (result && typeof result === 'object' && 'success' in result) {
    const envelope = result as { success?: boolean; data?: unknown; error?: string }
    if (envelope.success === false) throw new Error(envelope.error || t('Request failed'))
    return envelope.data as T
  }
  return result as T
}

export function formatTime(value: number): string {
  return value ? new Date(value).toLocaleString() : '—'
}

export function formatSize(bytes: number): string {
  if (!bytes) return ''
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  if (bytes >= 1024) return `${Math.round(bytes / 1024)} KB`
  return `${bytes} B`
}

export function platformLabel(target: string): string {
  const labels: Record<string, string> = {
    'x86_64-pc-windows-msvc': t('Windows (x64)'),
    'aarch64-pc-windows-msvc': t('Windows (ARM64)'),
    'x86_64-apple-darwin': t('macOS (Intel)'),
    'aarch64-apple-darwin': t('macOS (Apple Silicon)'),
    'x86_64-unknown-linux-gnu': t('Linux (amd64)'),
    'aarch64-unknown-linux-gnu': t('Linux (arm64)'),
  }
  return labels[target] || target
}

export function readBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onerror = () => reject(new Error(t('Failed to read the file')))
    reader.onload = () => {
      const text = typeof reader.result === 'string' ? reader.result : ''
      const index = text.indexOf('base64,')
      resolve(index >= 0 ? text.slice(index + 7) : text)
    }
    reader.readAsDataURL(file)
  })
}

export function pluginName(id: string, name?: string | null): string {
  if (id === 'agent2api-providers') return t('Provider Enhancement')
  if (id === 'agent2api-web-providers') return t('Web Providers')
  return name || id
}

export function pluginDescription(id: string, description?: string | null): string {
  if (id === 'agent2api-providers') {
    return t('Adds AI providers to the gateway, with account sign-in and import, model requests, credential renewal, model catalogs, balance queries and check-in. Available capabilities vary by provider.')
  }
  if (id === 'agent2api-web-providers') {
    return t('Adds web providers to Agent2API, with Zhipu Qingyan model requests, guest and signed-in accounts, credential renewal, model catalogs, points queries, check-in and task rewards.')
  }
  return description || t('The author has not provided a plugin description.')
}
