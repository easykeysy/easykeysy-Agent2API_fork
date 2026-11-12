export type ModelOption = {
  name: string
  supportsImages: boolean
}

/** as /api/session.models（ and /v1/models ），directory。 */
export function modelOptionsFrom(publicModels: unknown[]): ModelOption[] {
  const seen = new Set<string>()
  const options: ModelOption[] = []
  for (const value of publicModels) {
    if (!value || typeof value !== 'object' || Array.isArray(value)) continue
    const item = value as Record<string, unknown>
    if (item.kind === 'image' || item.supports_image_generation === true) continue
    const name = typeof item.id === 'string' ? item.id.trim() : ''
    const key = name.toLowerCase()
    if (!name || seen.has(key)) continue
    seen.add(key)
    options.push({ name, supportsImages: item.supports_images === true })
  }
  return options
}

export function selectedModel(options: ModelOption[], name: string | null | undefined): ModelOption | undefined {
  const requested = name?.trim().toLowerCase()
  return requested ? options.find(option => option.name.toLowerCase() === requested) : undefined
}
