import { createHash } from "node:crypto"

export const AGY_PROVIDER = "agy-cli"
export const AGY_PACKAGE = "@magpie-community/opencode-agy-cli-auth"
let localProviderName = AGY_PROVIDER

export function rememberProviderName(provider) {
  if (isLocalProvider(provider)) localProviderName = provider.name
}

export function localAccountName() { return localProviderName }
export const AGY_INSTRUCTIONS = "ACP uses the official agy CLI on the gateway computer, not the device running this browser or phone. Install agy 1.5.0 or later first: https://antigravity.google/docs/cli. For a custom installation path, set AGY_PATH in the gateway's startup environment.\nClick Sign in, authorize on Google's page, then paste the authorization code or full page URL unchanged. If agy is already signed in, it connects directly. The official client keeps Google's credentials; Agent2API stores only a local connection marker. Each gateway uses agy's current account, not multiple independent copies."

export function isLocalProvider(provider) {
  return provider.id === AGY_PROVIDER && provider.packageName === AGY_PACKAGE
}

function collectOptions(options, target, depth = 0) {
  if (!Array.isArray(options) || depth > 6) return
  for (const option of options) {
    if (typeof option?.value === "string") target.add(option.value)
    if (Array.isArray(option?.options)) collectOptions(option.options, target, depth + 1)
  }
}

export function normalizeLocalModels(provider, models) {
  if (!isLocalProvider(provider)) return models
  const offered = new Set(provider.localCatalog?.models.map(model => model.modelId).filter(id => typeof id === "string") || [])
  for (const option of provider.localCatalog?.configOptions || []) {
    if (option.id === "model" || option.category === "model") collectOptions(option.options, offered)
  }
  return Object.fromEntries(Object.entries(models).filter(([id, model]) => {
    const raw = model.api?.id || id.replace(/^agy-cli\//, "")
    return offered.has(raw) || [...offered].some(value => value.replace(/-(?:minimal|low|medium|high|xhigh|max)$/, "") === raw)
  }).map(([id, model]) => {
    const key = id.startsWith("agy-cli/") ? id : `agy-cli/${id}`
    return [key, { ...model, name: `${model.name || id} · ACP`, api: { ...model.api, id: model.api?.id || id },
      capabilities: { ...model.capabilities, toolcall: model.capabilities?.toolcall ?? model.tool_call ?? true,
        input: { ...model.capabilities?.input, image: model.capabilities?.input?.image === true && provider.localCatalog?.images === true } } }]
  }))
}

export function localRequest(provider, params, body) {
  if (!isLocalProvider(provider)) return
  if (typeof body.model === "string") body.model = body.model.replace(/^agy-cli\//, "")
  if (params.body?.reasoning_effort !== undefined) body.reasoning_effort = params.body.reasoning_effort
  const session = params.session || body.metadata?.user_id || ""
  const headers = params.headers || {}
  const client = [headers.authorization || "", headers["x-api-key"] || "", headers["x-goog-api-key"] || ""]
  const discriminator = JSON.stringify([provider.id, params.account, client, session])
  body.metadata = { ...body.metadata, user_id: createHash("sha256").update(discriminator).digest("hex") }
}

export function localErrorPolicy(provider) {
  return isLocalProvider(provider) ? { stop: true, reason: "A failed ACP session is not retried automatically or switched to another connection method." } : {}
}
