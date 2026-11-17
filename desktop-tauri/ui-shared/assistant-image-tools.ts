import type { AssistantTextFile } from './assistant-files'
import type { Translate } from './assistant-read-types'

export type ImageAsset = { id: string; url: string; mime: string; width: number; height: number }
export type ImageToolContext = {
  session: string
  images: string[]
  /** 最近一条带附件的用户消息里的 JSON / .info 文件（账号导入工具用） */
  files?: AssistantTextFile[]
  confirmed: boolean
  apiKey?: string
  apiKeyName?: string
  signal?: AbortSignal
}
export type ImageToolResult = { text: string; summary: string; images: ImageAsset[] }
export type ImageToolApi = (path: string, body: unknown, signal?: AbortSignal, apiKey?: string) => Promise<unknown>

export type AssistantImageTool = {
  id: string
  label: string
  kind: 'run'
  desc: string
  params: Record<string, unknown>
  plan: (args: Record<string, unknown>) => { lead: string; table: { head: string[]; rows: string[][] }; note: string }
  run: (args: Record<string, unknown>, context?: ImageToolContext) => Promise<ImageToolResult>
}

const properties = {
  prompt: { type: 'string', description: 'The full generation or editing requirement, including what should be kept.', maxLength: 32000 },
  model: { type: 'string', description: 'list_models the returned kind=image Model ID or an enabled mapping name; do not use a chat model.' },
  n: { type: 'integer', minimum: 1, maximum: 10, description: 'Generation count, default 1; Antigravity Only 1. is supported' },
  size: { type: 'string', description: 'auto or WIDTHxHEIGHT, e.g. 1024x1024; the exact supported range is up to the provider.' },
  quality: { type: 'string', description: 'Only pass a quality the provider supports; the default is auto. Antigravity does not support explicit quality tiers.",' },
  output_format: { type: 'string', enum: ['png', 'jpeg', 'webp'], description: 'Image output encoding; the upstream encoding is kept by default.' },
}

function text(value: unknown): string { return typeof value === 'string' ? value.trim() : '' }

function reference(value: unknown, context: ImageToolContext, t: Translate): string {
  const input = text(value)
  const match = /^attachment:(\d+)$/.exec(input)
  if (match) {
    const index = Number(match[1]) - 1
    if (index < 0 || index >= context.images.length) throw new Error(t('The reference image number does not exist'))
    return context.images[index]
  }
  if (/^assistant-image:[a-f0-9]{64}$/.test(input)) return input
  if (/^https?:\/\//i.test(input)) return input
  throw new Error(t('Use an attachment number, a generated image ID, or an HTTP(S) link for reference images'))
}

export function imageAssets(raw: unknown): ImageAsset[] {
  if (!Array.isArray(raw)) return []
  return raw.flatMap(value => {
    const item = value && typeof value === 'object' ? value as Record<string, unknown> : {}
    const id = text(item.id)
    const mime = text(item.mime)
    if (!/^[a-f0-9]{64}$/.test(id) || !['image/png', 'image/jpeg', 'image/webp'].includes(mime)) return []
    return [{ id, url: `/api/assistant/images/files/${id}`, mime, width: Number(item.width) || 0, height: Number(item.height) || 0 }]
  })
}

export function imageContext(messages: Array<{ role: string; images?: unknown }>): string[] {
  for (let index = messages.length - 1; index >= 0; index--) {
    const message = messages[index]
    if (message.role === 'user' && Array.isArray(message.images) && message.images.length) {
      return message.images.filter((image): image is string => typeof image === 'string')
    }
    const assets = imageAssets(message.images)
    if (assets.length) return assets.map(image => `assistant-image:${image.id}`)
  }
  return []
}

export function createAssistantImageTools(api: ImageToolApi, t: Translate): AssistantImageTool[] {
  return ([false, true] as const).map(editing => ({
    id: editing ? 'edit_images' : 'generate_images',
    label: editing ? t('Edit images') : t('Generate images'),
    kind: 'run',
    desc: editing
      ? 'Edits an image with natural language or generates a new image from a reference, via the standard Images editing pipeline. First use list_models with Select supports_image_editing",=true  model (Loomy, Antigravity, ModelScope or WorkBuddy). For images you can pass the current reference image number such as attachment:1, or an assistant-image:ID returned by a previous image tool; when omitted, the most recently uploaded or generated image is used. If there is no image, ask the user to upload one. Only PNG/JPEG/WebP reference images are supported; masks, input_fidelity, and streaming are not. This really consumes quota, and uncertain results are not resent automatically. Images returned by the tool are already displayed in the UI — just report the result; do not invent local file paths.",'
      : 'Generates an image from text, via the standard Images generation pipeline. First use list_models with Select kind",=image, supports_image_generation=true  model (Little Raccoon, Loomy, Antigravity, ModelScope or WorkBuddy). This really consumes quota and generates 1 image by default; increase the count only when the user asks. Uncertain results are not resent automatically. Images returned by the tool are already displayed in the UI — just report the result; do not invent local file paths.",',
    params: {
      type: 'object',
      properties: {
        ...properties,
        ...(editing ? { images: { type: 'array', minItems: 1, maxItems: 16, items: { type: 'string' }, description: 'Reference image numbers such as attachment:1, assistant-image:ID, or an HTTP(S) URL; when empty, the most recent set of images is used.",' } } : {}),
      },
      required: ['prompt', 'model'],
      additionalProperties: false,
    },
    plan: args => ({
      lead: editing ? t('Edit reference images using the prompt.') : t('Generate images from the prompt.'),
      table: { head: [t('参数'), t('内容')], rows: [[t('Model'), text(args.model)], [t('Prompt'), text(args.prompt)], [t('The count'), String(args.n ?? 1)]] },
      note: t('This sends a real upstream request and spends quota'),
    }),
    run: async (args, context) => {
      if (!context?.session) throw new Error(t('The image tool has no current session'))
      if (!context.apiKey) throw new Error(t('Select an API Key first'))
      if (context.signal?.aborted) throw new DOMException('Aborted', 'AbortError')
      const unknown = Object.keys(args).find(key => !Object.hasOwn(properties, key) && !(editing && key === 'images'))
      if (unknown) throw new Error(t('The image tool received an unsupported parameter: {name}', { name: unknown }))
      const request: Record<string, unknown> = {}
      for (const key of Object.keys(properties)) if (args[key] !== undefined) request[key] = args[key]
      if (!text(request.prompt) || !text(request.model)) throw new Error(t('The image tool requires a prompt and an image model'))
      if (editing) {
        if (args.images !== undefined && (!Array.isArray(args.images) || !args.images.length)) {
          throw new Error(t('The reference image list cannot be empty'))
        }
        const inputs = args.images === undefined ? context.images : (args.images as unknown[]).map(value => reference(value, context, t))
        if (!inputs.length) throw new Error(t('Upload a reference image or generate an image first'))
        request.images = inputs.map(image => ({ image_url: image }))
      }
      const raw = await api(`/api/assistant/images/${editing ? 'edits' : 'generations'}`, {
        session: context.session, confirmed: context.confirmed, request,
      }, context.signal, context.apiKey)
      const response = raw && typeof raw === 'object' ? raw as Record<string, unknown> : {}
      const images = imageAssets(response.data)
      if (!images.length) throw new Error(t('The image tool returned no displayable images'))
      const result = { ...response, data: images.map(image => ({ ...image, ref: `assistant-image:${image.id}` })), displayed: true }
      return { text: JSON.stringify(result), summary: t('Generated {n} images', { n: images.length }), images }
    },
  }))
}

export function assistantImageMeta(t: Translate) {
  return [
    { id: 'generate_images', label: t('Generate images'), desc: t('Generate images from text and display them in the conversation.'), tag: 'cost' as const },
    { id: 'edit_images', label: t('Edit images'), desc: t('Edit uploaded or generated images using multiple references.'), tag: 'cost' as const },
  ]
}
