/**
 * Agent2API before —— 。
 *
 * ── ────────────────────────────────────────────
 * use 「 in 」：`t('')` is 。 is ** JSON**，
 * 1 、 by （domain）file， in i18n-src/<locale>/<domain>.json。
 * this all「 use can in 」， to JSON 
 * （ new 1 empty ""， etc ）， and 1 1 、 not already has 。
 *
 * ── ────────────────────────────────────────────────────
 * ① `t('...')` / `t("...")` / `wbI18n.t('...')` —— ；
 * ② HTML file `data-i18n="..."` and `data-i18n-title` / `-placeholder`
 * / `-tip` / `-aria-label` **** and by ① t() 。
 * ：HTML file（data-i18n + t()）， and ；
 * to is HTML after ， use not will 。
 * 、 this ** not **， in as warning 
 * （ not to ， can that to need not need ）。
 *
 * ── ──────────────────────────────────────────────────
 * file → in down （DOMAIN_RULES）， by DOMAIN_ORDER 「」：
 * 1 in file when ， in before that ； already JSON 
 * not （ will has empty）。
 *
 * ── use ──────────────────────────────────────────────────────
 * node scan.mjs JSON 
 * node scan.mjs --check ：（ to no JSON）/ empty / ，
 * 1 has then 1（ CI / pre-commit ）
 *
 * Note: 「」， from not delete JSON 、 from not already has —— is ，
 * not this 。（JSON has 、 not to ） in --check ，
 * by is is this 。
 */

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const HERE = path.dirname(fileURLToPath(import.meta.url)) // desktop-tauri/i18n
const DESKTOP = path.resolve(HERE, '..') // desktop-tauri
const UI = path.join(DESKTOP, 'ui')
const ISLANDS_SRC = path.join(DESKTOP, 'ui-islands', 'src')
const UI_KIT_SRC = path.join(DESKTOP, 'ui-kit', 'src')
const SHARED_SRC = path.join(DESKTOP, 'ui-shared')
/**
 * directory（`desktop-tauri/mobile/src`）。
 *
 * is （ Vite + React）， `ui/m/` —— that is after 
 * bundle， no ， no has 。 this ， and ui-islands 1 。
 */
const MOBILE_SRC = path.join(DESKTOP, 'mobile', 'src')
const SRC_DIR = path.join(HERE, 'i18n-src')

/** directory（ not directory： not to ， not need need ） */
const LOCALES = ['en', 'zh-Hant', 'ja', 'ko', 'pt-BR']

/**
 * is ** 1 **：manifest.js by ， by 「」。
 * will ， need not need 。
 */
const DOMAIN_ORDER = [
  'common', 'login', 'accounts', 'account-settings', 'add-account', 'models', 'routing', 'agent', 'tools', 'settings', 'assistant', 'update',
  'keys', 'proxies', 'docs', 'report', 'checkin', 'tasks', 'logs', 'requests',
  'tables', 'port',
]

/**
 * file → 。files is ，prefixes is file before 。
 * in file 'common' and in warning（ new file）。
 */
const DOMAIN_RULES = [
  { domain: 'common', files: ['index.html', 'm.html', 'desktop-shell.tsx', 'desktop-titlebar.tsx', 'desktop-tooltip.tsx', 'island-registry.ts', 'clipboard.js', 'filter-memory.js', 'app.js', 'confirm-dialog.tsx', 'input.tsx', 'input-control.tsx', 'segmented.tsx', 'index.tsx', 'markdown.js', 'icons.js', 'App.tsx', 'main.tsx', 'Shell.tsx', 'NavBar.tsx', 'Icon.tsx', 'ProviderLogo.tsx', 'ui.tsx', 'PullToRefresh.tsx', 'AppUpdatePanel.tsx', 'CommunitySheet.tsx', 'i18n.ts', 'client.ts', 'gateway.ts', 'format.ts', 'providers.ts', 'providerFacts.ts', 'usePolling.ts', 'pwa.ts', 'usePwa.ts', 'appUpdate.ts', 'community.ts', 'servers.ts', 'auth.ts', 'store.ts', 'routes.ts'] },
  { domain: 'login', files: ['login.html', 'login-page.tsx', 'panel-captcha.js'] },
  { domain: 'account-settings', files: ['accounts-settings-dialog.tsx', 'accounts-settings-fields.tsx'] },
  { domain: 'accounts', files: ['accounts-page.tsx', 'accounts-hidden-notice.tsx', 'HiddenAccountsNotice.tsx', 'accounts-refresh.ts', 'accounts-update.ts', 'AccountsPage.tsx', 'AccountSheet.tsx', 'AccountsFilterSheet.tsx', 'accounts-shared.ts', 'accounts-columns.ts', 'accounts-store.ts', 'accounts-panels.tsx', 'accounts-queue-drag.ts', 'accounts-dialogs.tsx', 'accounts-batch-dialog.tsx', 'accounts-zcode-plan.tsx', 'accounts-proxy-form.tsx', 'accounts-dialog-limiter.tsx', 'accounts-dialog-usage.tsx', 'accounts-credit-detail.tsx', 'credit-expiry.ts', 'credential-expiry.ts', 'usage-readings.ts', 'usage-query-state.ts', 'accounts-usage.ts', 'accounts-usage-cache.ts', 'accounts-limit-detail.tsx', 'accounts-data.ts', 'accounts-domain.ts', 'accounts-group-settings.tsx', 'usage.ts', 'zcode-plans-modal.tsx', 'balance-query.tsx'], prefixes: ['accounts-limiter-', 'accounts-usage-', 'accounts-export', 'accounts-authorization'] },
  { domain: 'add-account', files: ['add-dynamic-provider.tsx', 'add-custom-provider.tsx', 'custom-provider-ui.js', 'providers.js', 'preset-providers.js', 'web-login.js', 'sms-login.js', 'autoclaw-oauth.js', 'aliyun-captcha.js', 'zcode-claim.js', 'zcode-captcha-pool.js', 'codearts-welfare.js'], prefixes: ['add-account-', 'add-provider-', 'add-doubaowork-'] },
  { domain: 'models', files: ['ModelsPage.tsx', 'model-capability.ts', 'model-capability-dialog.tsx', 'model-test-dialog.tsx', 'model-test-result.tsx'], prefixes: ['models-'] },
  { domain: 'routing', prefixes: ['routing-'] },
  // 「Agent」： AI client config（client / config / ）
  { domain: 'agent', files: ['AgentPage.tsx'], prefixes: ['agent-'] },
  // 「」： / （ / / up config / ）
  { domain: 'tools', files: ['ToolsPage.tsx'], prefixes: ['tools-'] },
  // 「set → 」（ system-info-pane.tsx and 
  // SystemInfoSheet.tsx use 1 ： / / path / ）。
  // settings —— is set 1 ， and set its 。
  { domain: 'settings', files: ['units.js', 'SettingsPage.tsx', 'MorePage.tsx', 'PwaNotice.tsx', 'ServersPage.tsx', 'LoginPage.tsx', 'system-info-pane.tsx', 'SystemInfoSheet.tsx', 'sidebar-layout.js'], prefixes: ['settings-', 'sidebar-', 'client-identification'] },
  // AI （「 can 」 1 + set「AI 」）：
  // for 、、set
  { domain: 'assistant', files: ['AssistantPage.tsx', 'AssistantHistory.tsx', 'AssistantModelPicker.tsx', 'assistant.ts', 'assistantTools.ts', 'MarkdownLite.tsx'], prefixes: ['assistant-'] },
  { domain: 'update', files: ['upgrade-panel.js', 'GithubAccessSettings.tsx', 'RequiredUpdateGate.tsx', 'updates.ts', 'github-access-editor.tsx'], prefixes: ['update-'] },
  { domain: 'keys', files: ['keys-page.tsx', 'KeysPage.tsx', 'model-api-key.ts', 'model-api-key-field.tsx', 'model-api-key-memory.ts', 'use-model-api-keys.ts'] },
  { domain: 'proxies', files: ['ProxiesPage.tsx', 'proxies-page.tsx', 'proxies-columns.ts', 'proxies-state.ts', 'proxies-dialogs.tsx', 'proxy-editor-sheet.tsx', 'proxy-retirement-dialog.tsx', 'sponsor.ts'] },
  { domain: 'docs', files: ['docs-page.tsx', 'DocsPage.tsx', 'DocsProviderData.ts', 'provider-docs-view.tsx'], prefixes: ['docs-provider-', 'docs-api-'] },
  { domain: 'report', files: ['report-page.tsx', 'report-charts.ts', 'ReportPage.tsx', 'HomePage.tsx', 'chart-data.ts', 'chart-labels.tsx', 'chart-tooltips.tsx', 'report-recharts.tsx', 'source-stats.tsx', 'api-key-detail.tsx', 'daily-trend-panel.tsx', 'SourceUsageSection.tsx', 'MobileReportTrend.tsx'] },
  { domain: 'checkin', files: ['checkin.ts', 'zcode.ts', 'zcode-plans.ts', 'zcode-claim-verification.js', 'CheckinPage.tsx', 'ZcodeCaptchaPanel.tsx'], prefixes: ['checkin-', 'Checkin'] },
  { domain: 'tasks', files: ['tasks-panel.tsx', 'TasksPage.tsx'] },
  { domain: 'logs', files: ['logs.ts', 'LogsPage.tsx'], prefixes: ['logs-'] },
  { domain: 'requests', files: ['RequestsPage.tsx', 'requests-page.tsx', 'request-detail.tsx', 'request-raw-panes.tsx', 'request-hover.tsx', 'request-clear-modal.tsx', 'conversation-preview.js', 'source-filters.tsx', 'request-cells.tsx', 'request-phase.ts'] },
  { domain: 'tables', files: ['table-columns.js', 'table-col-settings.tsx', 'table-shell.tsx'] },
  { domain: 'port', files: ['port-panel.tsx'] },
  // （ 1 ）： in this 1 file， not （），
  // 1 will i18n-src empty JSON —— common， and 
  // 。「」 in sidebar-layout.js ， settings 
  // （ that then already ）。
  { domain: 'common', files: ['plugins.ts', 'PluginsPage.tsx', 'PluginCards.tsx', 'PluginSettingsSheet.tsx'], prefixes: ['plugins-'] },
]

/* ─── file ─────────────────────────────── */

/**
 * excluding：i18n 。 is 「 / 」 （ to is t(key) this 
 * use ）， not ， will 1 none warning。
 */
const EXCLUDED = new Set([
  path.join(UI, 'i18n.js'),
  path.join(ISLANDS_SRC, 'i18n.ts'),
  path.join(UI_KIT_SRC, 'i18n.ts'),
])

/** ui/ （ not ：directory is and ， not is ） */
function uiScripts() {
  return fs.readdirSync(UI, { withFileTypes: true })
    .filter(entry => entry.isFile() && entry.name.endsWith('.js'))
    .map(entry => path.join(UI, entry.name))
    .filter(file => !EXCLUDED.has(file))
}

/** ui-islands/src down all .ts / .tsx（） */
function islandSources() {
  const out = []
  const walk = dir => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name)
      if (entry.isDirectory()) walk(full)
      else if (/\.tsx?$/.test(entry.name)) out.push(full)
    }
  }
  walk(ISLANDS_SRC)
  if (fs.existsSync(SHARED_SRC)) {
    for (const entry of fs.readdirSync(SHARED_SRC, { withFileTypes: true })) {
      if (entry.isFile() && (entry.name === 'logs.ts' || entry.name === 'credit-expiry.ts' || entry.name === 'credential-expiry.ts' || entry.name === 'usage-readings.ts' || entry.name === 'usage-query-state.ts' || entry.name === 'zcode-plans.ts' || entry.name === 'use-model-api-keys.ts' || entry.name === 'updates.ts' || entry.name === 'proxy-retirement-dialog.tsx' || /^(assistant-|agent-|docs-api-|model-api-key|client-identification).*\.tsx?$/.test(entry.name))) {
        out.push(path.join(SHARED_SRC, entry.name))
      }
    }
  }
  return out.filter(file => !EXCLUDED.has(file))
}

/**
 * mobile/src down all .ts / .tsx（）。
 *
 * directory not in when returnempty： is after ， / old no has ，
 * not this as 1 optionaldirectory。
 */
function mobileSources() {
  if (!fs.existsSync(MOBILE_SRC)) return []
  const out = []
  const walk = dir => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name)
      if (entry.isDirectory()) walk(full)
      else if (/\.tsx?$/.test(entry.name)) out.push(full)
    }
  }
  walk(MOBILE_SRC)
  return out.filter(file => !EXCLUDED.has(file))
}

function componentSources() {
  const out = []
  const walk = dir => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name)
      if (entry.isDirectory()) walk(full)
      else if (/\.tsx?$/.test(entry.name)) out.push(full)
    }
  }
  if (fs.existsSync(UI_KIT_SRC)) walk(UI_KIT_SRC)
  return out.filter(file => !EXCLUDED.has(file))
}

/** ：、 before ； not in return null（ use common + warning） */
function domainOf(file) {
  if (file.startsWith(`${UI_KIT_SRC}${path.sep}`)) return 'common'
  const base = path.basename(file)
  for (const rule of DOMAIN_RULES) {
    if (rule.files && rule.files.includes(base)) return rule.domain
  }
  for (const rule of DOMAIN_RULES) {
    if (rule.prefixes && rule.prefixes.some(prefix => base.startsWith(prefix))) return rule.domain
  }
  return null
}

/* ─── ───────────────────────── */

/** JS （\n / \t / \uXXXX / \u{...} / etc ） */
function unescapeJs(raw) {
  return raw.replace(
    /\\(?:u\{([0-9a-fA-F]+)\}|u([0-9a-fA-F]{4})|x([0-9a-fA-F]{2})|([\s\S]))/g,
    (_match, code, u4, x2, ch) => {
      if (code) return String.fromCodePoint(parseInt(code, 16))
      if (u4) return String.fromCharCode(parseInt(u4, 16))
      if (x2) return String.fromCharCode(parseInt(x2, 16))
      switch (ch) {
        case 'n': return '\n'
        case 't': return '\t'
        case 'r': return '\r'
        case 'b': return '\b'
        case 'f': return '\f'
        case 'v': return '\v'
        case '0': return '\0'
        default: return ch // \' \" \\ \/ 等一律还原成字符本身
      }
    },
  )
}

/** HTML （ ， HTML use ） */
function unescapeHtml(raw) {
  return raw.replace(/&(#x?[0-9a-fA-F]+|amp|lt|gt|quot|apos|#39);/g, (match, entity) => {
    if (entity === 'amp') return '&'
    if (entity === 'lt') return '<'
    if (entity === 'gt') return '>'
    if (entity === 'quot') return '"'
    if (entity === 'apos' || entity === '#39') return "'"
    if (entity[0] === '#') {
      const code = entity[1] === 'x' || entity[1] === 'X'
        ? parseInt(entity.slice(2), 16)
        : parseInt(entity.slice(1), 10)
      return Number.isFinite(code) ? String.fromCodePoint(code) : match
    }
    return match
  })
}

/* ─── ─────────────────────────────────── */

/** file：t('...') / wbI18n.t('...') ； as warning */
function extractFromCode(source, file, warnings) {
  const keys = []
  const patterns = [
    // wbI18n. before use （， not up down limit）
    /wbI18n\s*\.\s*t\s*\(\s*(['"])((?:\\.|(?!\1)[^\\\n])*)\1/g,
    // t(...)： before not can is /， in `split(`、`x.t(` this 
    /(?<![\w$.])t\s*\(\s*(['"])((?:\\.|(?!\1)[^\\\n])*)\1/g,
  ]
  for (const re of patterns) {
    for (const match of source.matchAll(re)) {
      const key = unescapeJs(match[2])
      if (key) keys.push(key)
    }
  }
  // ： `t(` ${n} `)`、 `t(label)` etc —— not to ，。
  // need / / / in ： this `t()` empty use not will 。
  const loose = /(?<![\w$.])(?:wbI18n\s*\.\s*)?t\s*\(\s*(?=[A-Za-z_$`(\u4e00-\u9fff])/g
  for (const match of source.matchAll(loose)) {
    const line = source.slice(0, match.index).split('\n').length
    warnings.push(`${path.relative(DESKTOP, file)}:${line} 非字面量 t(...) 首参（模板串 / 变量），键未提取，需人工处理`)
  }
  return keys
}

/** HTML file：data-i18n* （ HTML ， not ； t() by extractFromCode ） */
function extractFromHtml(source, file, warnings) {
  const keys = []
  const clean = source.replace(/<!--[\s\S]*?-->/g, '')
  // when data-i18n and data-i18n-title / -placeholder / -tip / -aria-label
  const re = /\bdata-i18n(?:-[a-z-]+)?\s*=\s*"([^"]*)"/g
  for (const match of clean.matchAll(re)) {
    const key = unescapeHtml(match[1]).trim()
    if (key) keys.push(key)
  }
  // data-i18n（none） in when will textContent ， not —— 
  const bare = /\bdata-i18n(?:-[a-z-]+)?(?=[\s>/])/g
  for (const match of clean.matchAll(bare)) {
    const line = clean.slice(0, match.index).split('\n').length
    warnings.push(`${path.relative(DESKTOP, file)}:${line} 裸 data-i18n（无显式值），运行时才拿 textContent 当键，建议补上显式值`)
  }
  return keys
}

/** total：{ domain: Set<key> } + warnings */
function scan() {
  const perDomain = new Map(DOMAIN_ORDER.map(domain => [domain, new Set()]))
  const warnings = []
  const files = [
    ...uiScripts(),
    path.join(UI, 'index.html'),
    path.join(UI, 'login.html'),
    // （`/m` in ）。 is data-i18n and t()，
    // and index.html 1 。
    path.join(UI, 'm.html'),
    ...islandSources(),
    path.join(SHARED_SRC, 'provider-docs-view.tsx'),
    ...componentSources(),
    ...mobileSources(),
  ]

  for (const file of files) {
    if (!fs.existsSync(file)) continue
    const base = path.basename(file)
    let domain = domainOf(file)
    if (!domain) {
      domain = 'common'
      warnings.push(`${path.relative(DESKTOP, file)} 未登记域归属，已暂归 common（请在 scan.mjs 的 DOMAIN_RULES 里登记）`)
    }
    const source = fs.readFileSync(file, 'utf8')
    // HTML file：data-i18n + t()/wbI18n.t() 。
    // HTML （ and extractFromHtml 1 ）， not will ；
    // ， to Set 。
    const keys = base.endsWith('.html')
      ? [
        ...extractFromHtml(source, file, warnings),
        ...extractFromCode(source.replace(/<!--[\s\S]*?-->/g, ''), file, warnings),
      ]
      : extractFromCode(source, file, warnings)
    for (const key of keys) perDomain.get(domain).add(key)
  }
  return { perDomain, warnings }
}

/* ─── （） ───────────────────── */

/** has i18n-src：return { owner: Map<key, domain>, json: Map<`${locale}/${domain}`, object> } */
function readExisting() {
  const owner = new Map()
  const json = new Map()
  for (const locale of LOCALES) {
    const dir = path.join(SRC_DIR, locale)
    if (!fs.existsSync(dir)) continue
    for (const entry of fs.readdirSync(dir).filter(name => name.endsWith('.json'))) {
      const domain = entry.slice(0, -'.json'.length)
      let data = {}
      try {
        data = JSON.parse(fs.readFileSync(path.join(dir, entry), 'utf8'))
      } catch {
        // JSON not ： will none， this to empty for and 
        process.stderr.write(`[scan] 警告：${locale}/${entry} 不是合法 JSON，按空对象处理\n`)
        data = {}
      }
      json.set(`${locale}/${domain}`, data)
      // in 「」； already has JSON 
      for (const key of Object.keys(data)) {
        if (!owner.has(key)) owner.set(key, domain)
      }
    }
  }
  return { owner, json }
}

/**
 * to by 「 already has ， its by 」 down 。
 * return { assigned: Map<domain, string[]>, unowned: Map<key, domain> }
 */
function assign(perDomain, owner) {
  const assigned = new Map()
  const takenByNew = new Map() // 本次新分配的键 → 域，用于同批内去重
  for (const domain of DOMAIN_ORDER) {
    const keys = []
    for (const key of perDomain.get(domain)) {
      const existing = owner.get(key)
      if (existing) {
        if (existing === domain) keys.push(key) // 已在这个域，保持
        continue // 已属别的域：不重复写
      }
      if (takenByNew.has(key)) continue // 同批别的域先拿了
      takenByNew.set(key, domain)
      keys.push(key)
    }
    assigned.set(domain, keys)
  }
  return { assigned, takenByNew }
}

/* ─── and ─────────────────────────── */

/** by Unicode （ is BMP in ， etc default， this ） */
function sortKeys(keys) {
  return [...keys].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
}

function writeJson(file, object) {
  const ordered = {}
  for (const key of sortKeys(Object.keys(object))) ordered[key] = object[key]
  fs.mkdirSync(path.dirname(file), { recursive: true })
  fs.writeFileSync(file, `${JSON.stringify(ordered, null, 2)}\n`, 'utf8')
}

/* ─── ───────────────────────────────── */

const ONLY_DOMAIN = process.argv.find(arg => arg.startsWith('--domain='))?.slice('--domain='.length)
if (ONLY_DOMAIN && !DOMAIN_ORDER.includes(ONLY_DOMAIN)) throw new Error(`Unknown i18n domain: ${ONLY_DOMAIN}`)
const ACTIVE_DOMAINS = ONLY_DOMAIN ? [ONLY_DOMAIN] : DOMAIN_ORDER

function runScan() {
  const { perDomain, warnings } = scan()
  const { owner, json } = readExisting()
  const { assigned } = assign(perDomain, owner)

  let totalAdded = 0
  for (const domain of ACTIVE_DOMAINS) {
    const keys = assigned.get(domain)
    if (keys.length === 0) continue
    for (const locale of LOCALES) {
      const mapKey = `${locale}/${domain}`
      const data = { ...(json.get(mapKey) || {}) }
      let added = 0
      for (const key of keys) {
        if (!Object.prototype.hasOwnProperty.call(data, key)) {
          data[key] = ''
          added += 1
        }
      }
      if (added > 0 || !json.has(mapKey)) {
        writeJson(path.join(SRC_DIR, locale, `${domain}.json`), data)
        totalAdded += added
      }
    }
  }

  // 
  process.stdout.write('\n[scan] 各域键数：\n')
  let total = 0
  for (const domain of DOMAIN_ORDER) {
    const keys = assigned.get(domain)
    if (keys.length === 0) continue
    total += keys.length
    process.stdout.write(`  ${domain.padEnd(12)} ${String(keys.length).padStart(4)}\n`)
  }
  process.stdout.write(`  ${'合计'.padEnd(12)} ${String(total).padStart(4)}\n`)
  process.stdout.write(`[scan] 本次新增键 ${totalAdded} 条（× ${LOCALES.length} 语言）\n`)
  if (warnings.length > 0) {
    process.stdout.write(`\n[scan] warning（${warnings.length} 条，需人工处理）：\n`)
    for (const warning of warnings) process.stdout.write(`  - ${warning}\n`)
  } else {
    process.stdout.write('[scan] 无 warning\n')
  }
}

function runCheck() {
  const { perDomain, warnings } = scan()
  const { owner, json } = readExisting()
  const { assigned } = assign(perDomain, owner)

  // to （used for）
  const allScanned = new Set()
  for (const domain of DOMAIN_ORDER) for (const key of assigned.get(domain)) allScanned.add(key)

  let missingTotal = 0
  process.stdout.write('\n[check] 各语言 / 各域的缺失与空值：\n')
  for (const locale of LOCALES) {
    let localeMissing = 0
    let localeEmpty = 0
    for (const domain of ACTIVE_DOMAINS) {
      const expected = assigned.get(domain)
      if (expected.length === 0) continue
      const data = json.get(`${locale}/${domain}`) || {}
      const missing = expected.filter(key => !Object.prototype.hasOwnProperty.call(data, key))
      const empty = Object.keys(data).filter(key => data[key] === '').length
      if (missing.length > 0 || empty > 0) {
        process.stdout.write(`  ${locale}/${domain}: 缺 ${missing.length}，空值 ${empty}\n`)
        if (missing.length > 0) for (const key of missing) process.stdout.write(`      miss: ${key}\n`)
      }
      localeMissing += missing.length
      localeEmpty += empty
    }
    missingTotal += localeMissing
    process.stdout.write(`  ${locale.padEnd(8)} 缺失键 ${localeMissing}，空值键 ${localeEmpty}\n`)
  }

  // ：JSON has 、 not to 
  const orphans = []
  for (const locale of LOCALES) {
    for (const domain of ACTIVE_DOMAINS) {
      const data = json.get(`${locale}/${domain}`)
      if (!data) continue
      for (const key of Object.keys(data)) {
        if (!allScanned.has(key)) orphans.push(`${locale}/${domain}: ${key}`)
      }
    }
  }
  if (orphans.length > 0) {
    process.stdout.write(`\n[check] 孤儿键（JSON 有、源码扫不到）${orphans.length} 条：\n`)
    for (const orphan of orphans) process.stdout.write(`  - ${orphan}\n`)
  }

  if (warnings.length > 0) {
    process.stdout.write(`\n[check] warning（${warnings.length} 条）：\n`)
    for (const warning of warnings) process.stdout.write(`  - ${warning}\n`)
  }

  if (missingTotal > 0) {
    process.stdout.write(`\n[check] 失败：有 ${missingTotal} 处缺失键（先跑 node scan.mjs 补齐骨架）\n`)
    process.exitCode = 1
  } else {
    process.stdout.write('\n[check] 通过：所有语言的所有域都没有缺失键（空值属预期，等翻译）\n')
  }
}

// in 「 node scan.mjs」 when ： build.mjs import when not can 1 
const invokedDirectly = process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url
if (invokedDirectly) {
  if (process.argv.includes('--check')) runCheck()
  else runScan()
}

// build.mjs use 1 / 
export { LOCALES, DOMAIN_ORDER }
