type Translate = (key: string) => string

export type SystemProxy = {
  source: 'custom'
  protocol: 'http' | 'socks5'
  host: string
  port: number
  username: string
  password: string
}

export function readSystemProxy(value: unknown, t: Translate): SystemProxy {
  const proxy = value && typeof value === 'object' ? (value as Record<string, unknown>).proxy : undefined
  if (proxy == null) throw new Error(t('No importable system proxy was found on the gateway machine. Enter one manually or choose a saved proxy.'))
  if (typeof proxy !== 'object' || Array.isArray(proxy)) throw new Error(t('The system proxy information is invalid. Enter it manually.'))
  const item = proxy as Record<string, unknown>
  if (item.source !== 'custom' || !['http', 'socks5'].includes(String(item.protocol))
    || typeof item.host !== 'string' || !item.host.trim()
    || typeof item.port !== 'number' || !Number.isInteger(item.port) || item.port < 1 || item.port > 65535
    || typeof item.username !== 'string' || typeof item.password !== 'string') {
    throw new Error(t('The system proxy information is invalid. Enter it manually.'))
  }
  return {
    source: 'custom', protocol: item.protocol as SystemProxy['protocol'], host: item.host,
    port: item.port, username: item.username, password: item.password,
  }
}
