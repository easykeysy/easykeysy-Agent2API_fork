const EXCHANGE_PATH = "/deepai_code/desktop_auth/exchange/"

export function loginNetwork(provider, authorization) {
  if (provider.id !== "deepaicode" || !authorization.url) return null
  const url = new URL(authorization.url)
  if (url.pathname !== "/deepai_code/desktop_auth/") return null
  return { exchangeUrl: new URL(EXCHANGE_PATH, url.origin).href, hint: "", error: "" }
}

export function observeLoginRequest(context, url, method) {
  const login = context.loginNetwork
  if (!login || method.toUpperCase() !== "POST" || url.href !== login.exchangeUrl) return false
  login.hint = "Browser authorization received. Retrieving account credentials…"
  login.error = ""
  return true
}

export function loginNetworkError(context) {
  if (context.signal.aborted || !context.loginNetwork) return
  context.loginNetwork.error = context.proxy
    ? "Browser authorization was received, but retrieving account credentials through the selected proxy failed. Check the proxy and sign in again."
    : "Browser authorization was received, but the gateway could not reach DeepAI directly. Fill from the system proxy or choose a working sign-in proxy, then sign in again."
}
