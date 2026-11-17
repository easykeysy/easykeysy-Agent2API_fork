import { AGY_INSTRUCTIONS, isLocalProvider } from "./local-provider.mjs"
import { applies } from "./login-prompts.mjs"

const WPS_SITE = "https://lingxi.wps.cn/"
const LOCAL_CLIENT_HINT = "Reads the signed-in client on the gateway machine, not the computer displaying the panel. You can also paste credentials instead."

export function loginMethods(provider) {
  const original = provider.hooks.auth?.methods ?? []
  const methods = original.map((method, methodIndex) => ({ ...method, methodIndex }))
  if (provider.id === "wps-lingxi") {
    const paste = methods.find(method => method.type === "oauth" && method.prompts?.some(prompt => prompt.key === "cookie"))
    for (const method of methods) {
      method.browserMode = method.prompts?.length ? "none" : /导入|客户端/.test(method.label) ? "none" : "plugin"
      if (method.browserMode === "plugin") {
        method.label = "Sign in using the plugin's browser"
        method.instructions = "The plugin starts a separate browser on the gateway machine and reads the sign-in result. It does not use the system's default browser."
      } else if (!method.prompts?.length) method.localOnly = true
    }
    if (paste && typeof paste.authorize === "function") {
      methods.unshift({
        methodIndex: original.length, type: "oauth", label: "Sign in in the embedded window", embeddedOnly: true,
        desktopOnly: true, cookieCapture: "wps-lingxi", browserMode: "choice",
        instructions: "Sign in to WPS in the app's embedded window. Credentials are passed only to the original plugin for verification and saved to this gateway.",
        async authorize(_inputs, options) {
          return { url: WPS_SITE, method: "code", instructions: this.instructions,
            async callback(cookie) {
              const authorization = await paste.authorize({ cookie }, options)
              if (authorization?.type === "failed") return authorization
              if (typeof authorization?.callback !== "function") throw new Error("WPS The plugin did not return a credential verification flow")
              return authorization.callback(undefined, options)
            },
          }
        },
      })
    }
  } else if (["intern-inkstone", "mclaw"].includes(provider.id)) {
    // 这两家的登录与续期依赖插件自己的浏览器环境，不能替换成宿主开页。
    for (const method of methods) if (method.type === "oauth" && !method.embeddedOnly
      && (!method.browserMode || method.browserMode === "choice")) method.browserMode = "plugin"
  } else if (provider.id === "agnes-code") {
    for (const method of methods) if (method.type === "oauth") {
      method.localOnly = true
      method.browserMode = "none"
      method.instructions = LOCAL_CLIENT_HINT
    }
  }
  for (const method of methods) {
    if (isLocalProvider(provider) && method.type === "oauth") {
      method.label = "Google sign-in (ACP)"
      method.browserMode = "external"
      method.instructions = AGY_INSTRUCTIONS
    }
    if (provider.id === "deepaicode" && method.type === "oauth") {
      method.instructions = "After the browser shows sign-in complete, the gateway still needs to retrieve account credentials online. If your browser or Magpie uses a system proxy, click “Fill from system proxy” below first. Direct connections do not use the system proxy."
    }
    if (method.type === "api" && (method.keyless || provider.id === "deepaicode" && method.prompts?.some(prompt => prompt.key === "dir"))) {
      method.instructions = LOCAL_CLIENT_HINT
    }
  }
  return methods
}

export function promptOptional(provider, method, prompt) {
  if (typeof prompt.optional === "boolean") return prompt.optional
  if (typeof prompt.required === "boolean") return !prompt.required
  if (provider.id === "deepaicode" && method.prompts?.some(item => item.key === "dir")) return true
  if (provider.id === "cloudflare-workers-ai" && prompt.key === "accountId") return false
  if (typeof prompt.validate === "function") {
    try {
      const result = prompt.validate("")
      if (result && typeof result.then === "function") { void result.catch(() => {}); return false }
      return !result
    } catch { return false }
  }
  return /^(username|tenant)$/i.test(prompt.key)
}

export function promptSecret(prompt) {
  if (prompt.type === "password") return true
  if (typeof prompt.secret === "boolean") return prompt.secret
  return prompt.type !== "select" && /key|token|secret|password|cookie|credential|sid/i.test(prompt.key)
}

export async function validateInputs(method, inputs) {
  for (const prompt of method.prompts ?? []) {
    if (!applies(prompt, inputs)) continue
    if (typeof prompt.validate !== "function") continue
    const error = await prompt.validate(inputs[prompt.key] ?? "")
    if (typeof error === "string" && error) throw new Error(error)
  }
}

const loopback = host => ["127.0.0.1", "localhost", "[::1]"].includes(host)

export function authorizationPolicy(provider, method, authorization) {
  const url = authorization.url ? new URL(authorization.url) : null
  const targets = []
  if (url && authorization.method === "auto") {
    for (const key of ["redirect_uri", "redirect_url", "callback_url", "auth_callback_url", "return_url"]) {
      try {
        const target = new URL(url.searchParams.get(key))
        if (target.protocol === "http:" && loopback(target.hostname) && target.port && !target.username && !target.password) targets.push(target)
      } catch {}
    }
    if (provider.id === "deepaicode" && url.pathname === "/deepai_code/desktop_auth/") {
      const port = url.searchParams.get("port")
      if (/^\d+$/.test(port) && +port > 0 && +port < 65536) {
        const target = new URL(`http://127.0.0.1:${port}/auth-callback`)
        target.searchParams.set("state", url.searchParams.get("state") || "")
        targets.push(target)
      }
    }
    if (provider.id === "joycode" && url.hostname === "joycode.jd.com") {
      const port = url.searchParams.get("authPort")
      if (/^\d+$/.test(port) && +port > 0 && +port < 65536) {
        const target = new URL(`http://127.0.0.1:${port}/`)
        for (const key of ["authKey", "authPort"]) target.searchParams.set(key, url.searchParams.get(key) || "")
        targets.push(target)
      }
    }
  }
  const hosts = provider.id === "wps-lingxi" ? ["wps.cn", "kdocs.cn"]
    : provider.id === "joycode" ? ["jd.com", "jd.com.cn"] : []
  return {
    targets,
    ui: {
      browserMode: authorization.opened === true ? "plugin" : method.browserMode || "choice",
      manualCallback: !method.cookieCapture && (authorization.method === "code" || targets.length > 0),
      callbackInput: authorization.method === "code" && !method.cookieCapture ? "code" : "url",
      cookieCapture: method.cookieCapture || null,
      loopbackTargets: targets.map(target => target.href),
    },
    allowedHosts: targets.length ? [] : hosts,
  }
}

export function checkedCallback(targets, raw) {
  const url = new URL(raw)
  const target = targets.find(target => url.protocol === "http:" && loopback(url.hostname)
    && url.port === target.port && url.pathname === target.pathname && !url.username && !url.password)
  if (!target) throw new Error("The callback address does not belong to this login's local receiver")
  for (const [key, value] of target.searchParams) {
    if (key === "state" && !url.searchParams.has(key)) continue
    if (url.searchParams.get(key) !== value) throw new Error("The callback address does not match the current login task")
  }
  url.hostname = target.hostname
  return url.href
}
