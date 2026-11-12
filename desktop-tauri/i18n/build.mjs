/**
 * Agent2API before —— 。
 *
 * ── / ──────────────────────────────────────────────
 * desktop-tauri/i18n/i18n-src/<locale>/<domain>.json （scan.mjs + ）
 * desktop-tauri/ui/i18n/manifest.js （）
 * desktop-tauri/ui/i18n/<locale>/<domain>.js （ when ，）
 *
 * when is 「 1 1 .js、 window.wbI18nDict up Object.assign」：
 * head by manifest document.write this <script>，
 * is in 1 （units.js / app.js / ） before already 。
 * not islands/ui.js is —— is 、 by ， no need 
 * 「must has 1 React 」 ， after by update。
 *
 * ── as empty = no ──────────────────────────────────────────
 * empty** not **（ not is empty）： when t() not to ，
 * empty will 。 is 「 in 」， not will 、 not will empty。
 *
 * ── zh-Hant ：opencc + ──────────────────────────
 * not ， is use opencc-js cn→twp（、）
 * ， use zh-hant-glossary.json opencc （「→」 as 
 * 「」 this ）。i18n-src/zh-Hant empty is ，。
 * opencc-js not up when ： i18n-src/zh-Hant already has ， and in 
 * stderr OPENCC_MISSING。
 *
 * ── before ──────────────────────────────────────────
 * 1 new Function(src) check， 1 not ，
 * not 。
 */

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { DOMAIN_ORDER } from './scan.mjs'

const HERE = path.dirname(fileURLToPath(import.meta.url)) // desktop-tauri/i18n
const DESKTOP = path.resolve(HERE, '..')
const SRC_DIR = path.join(HERE, 'i18n-src')
const OUT_DIR = path.join(DESKTOP, 'ui', 'i18n')

/** need need （ not need need ： not to ，） */
const OUTPUT_LOCALES = ['en', 'zh-Hant', 'ja', 'ko', 'pt-BR']
const ONLY_DOMAIN = process.argv.find(arg => arg.startsWith('--domain='))?.slice('--domain='.length)
if (ONLY_DOMAIN && !DOMAIN_ORDER.includes(ONLY_DOMAIN)) throw new Error(`Unknown i18n domain: ${ONLY_DOMAIN}`)

/* ─── ───────────────────────────────── */

/** → JS ：JSON.stringify / / \n， up \u2028\u2029 */
function jsString(value) {
  return JSON.stringify(value)
    .replace(/\u2028/g, '\\u2028')
    .replace(/\u2029/g, '\\u2029')
}

/** by Unicode */
function sortKeys(keys) {
  return [...keys].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
}

/** 1 JSON（ is ）； not in or return null */
function readJson(file) {
  if (!fs.existsSync(file)) return null
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'))
  } catch (error) {
    process.stderr.write(`[build] 错误：${path.relative(DESKTOP, file)} 不是合法 JSON：${error.message}\n`)
    process.exit(1)
  }
}

/** 1 .js （， diff and for ） */
function dictFileContent(locale, domain, entries) {
  const header = `/* Agent2API ：${locale}/${domain} —— by desktop-tauri/i18n/build.mjs ，； is i18n-src/*.json */`
  const boot = 'window.wbI18nDict = window.wbI18nDict || {};'
  if (entries.length === 0) return `${header}\n${boot}\n`
  const pairs = entries.map(([key, value]) => `${jsString(key)}: ${jsString(value)}`).join(', ')
  return `${header}\n${boot} Object.assign(window.wbI18nDict, { ${pairs} });\n`
}

/** （head file） */
function manifestContent() {
  const list = DOMAIN_ORDER.map(name => jsString(name)).join(', ')
  const header = '/* Agent2API —— by desktop-tauri/i18n/build.mjs ， */'
  return `${header}\nwindow.__WB_I18N_DOMAINS = [${list}];\n`
}

/* ─── ───────────────────────────── */

/** { keys: Map<domain, string[]>, values: Map<`${locale}/${domain}`, object> } */
function loadSources() {
  const keys = new Map(DOMAIN_ORDER.map(domain => [domain, new Set()]))
  const values = new Map()
  for (const locale of OUTPUT_LOCALES) {
    const dir = path.join(SRC_DIR, locale)
    if (!fs.existsSync(dir)) continue
    for (const name of fs.readdirSync(dir).filter(file => file.endsWith('.json'))) {
      const domain = name.slice(0, -'.json'.length)
      const data = readJson(path.join(dir, name)) || {}
      values.set(`${locale}/${domain}`, data)
      const bucket = keys.get(domain) || new Set()
      for (const key of Object.keys(data)) bucket.add(key)
      keys.set(domain, bucket)
    }
  }
  const sorted = new Map()
  for (const [domain, set] of keys) sorted.set(domain, sortKeys(set))
  return { keys: sorted, values }
}

/* ─── zh-Hant ─────────────────────────── */

/** opencc-js（cn→twp）。 not up return null and OPENCC_MISSING */
async function loadConverter() {
  try {
    const OpenCC = await import('opencc-js')
    return OpenCC.Converter({ from: 'cn', to: 'twp' })
  } catch {
    process.stderr.write('[build] OPENCC_MISSING：opencc-js 未安装，zh-Hant 只输出 i18n-src/zh-Hant 里已有人工译文的键\n')
    return null
  }
}

/**
 * 1 「 → 」 ：opencc 、、。
 *
 * Why need 「 for 」 1 ：opencc after 
 * already is ，（「」） in not ，empty。
 * for 「 in opencc down will 」（ → ）， this 
 * **** need to （ → ）—— this can to 。
 * by from to ， 1 。
 */
function makeHantGenerator(converter, glossary) {
  if (!converter) return null
  const rendered = Object.entries(glossary)
    .sort((a, b) => b[0].length - a[0].length || (a[0] < b[0] ? -1 : 1))
    .map(([source, target]) => [converter(source), target])
    .filter(([form, target]) => form && form !== target)
  return key => {
    let out = converter(key)
    for (const [form, target] of rendered) {
      if (out.includes(form)) out = out.split(form).join(target)
    }
    // in ： to （ not to ）
    if (Object.prototype.hasOwnProperty.call(glossary, key)) out = glossary[key]
    return out
  }
}

/* ─── ───────────────────────────────── */

async function main() {
  const glossary = readJson(path.join(HERE, 'zh-hant-glossary.json')) || {}
  const converter = await loadConverter()
  const toHant = makeHantGenerator(converter, glossary)

  const { keys, values } = loadSources()
  const produced = DOMAIN_ORDER.filter(domain => (!ONLY_DOMAIN || domain === ONLY_DOMAIN) && (keys.get(domain) || []).length > 0)

  // to and ， —— not 
  const outputs = []
  if (!ONLY_DOMAIN) outputs.push({ file: path.join(OUT_DIR, 'manifest.js'), content: manifestContent() })

  let hantGenerated = 0
  let hantManual = 0
  for (const locale of OUTPUT_LOCALES) {
    // and （ 1 file down this ）： head need 22 
    // <script>， in 1 。file —— by manifest 、
    // login/accounts ，。
    const combined = []
    for (const domain of produced) {
      const manual = values.get(`${locale}/${domain}`) || {}
      const entries = []
      for (const key of keys.get(domain)) {
        let value = manual[key]
        if (locale === 'zh-Hant') {
          if (typeof value === 'string' && value) {
            hantManual += 1
          } else if (toHant) {
            value = toHant(key)
            if (value) hantGenerated += 1
          }
        }
        if (typeof value === 'string' && value) entries.push([key, value])
      }
      outputs.push({ file: path.join(OUT_DIR, locale, `${domain}.js`), content: dictFileContent(locale, domain, entries) })
      combined.push(...entries)
    }
    // （--domain=） not and ： has this 1 ， will 
    // 。 after 1 。
    if (!ONLY_DOMAIN) {
      outputs.push({ file: path.join(OUT_DIR, 'combined', `${locale}.js`), content: dictFileContent(locale, 'combined', combined) })
    }
  }

  for (const { file, content } of outputs) {
    try {
      // （ not ）： is 「window up Object.assign」 this is 
      new Function(content)
    } catch (error) {
      process.stderr.write(`[build] 自校验失败：${path.relative(DESKTOP, file)} 无法被 JS 解析：${error.message}\n`)
      process.exit(1)
    }
  }

  // before file：directory down not this file（JSON already / not in ）
  for (const locale of OUTPUT_LOCALES) {
    const dir = path.join(OUT_DIR, locale)
    if (!fs.existsSync(dir)) continue
    for (const name of fs.readdirSync(dir).filter(file => file.endsWith('.js'))) {
      const domain = name.slice(0, -'.js'.length)
      if ((!ONLY_DOMAIN || domain === ONLY_DOMAIN) && !produced.includes(domain)) {
        fs.unlinkSync(path.join(dir, name))
        process.stdout.write(`[build] 清理过期词典：${locale}/${name}\n`)
      }
    }
  }

  for (const { file, content } of outputs) {
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, content, 'utf8')
  }

  // 
  process.stdout.write(`\n[build] 产出域 ${produced.length} 个：${produced.join(', ') || '（无）'}\n`)
  process.stdout.write(`[build] ${ONLY_DOMAIN ? ONLY_DOMAIN : 'manifest.js'} + ${OUTPUT_LOCALES.length} 个语言目录，共 ${outputs.length} 个文件\n`)
  if (converter) {
    process.stdout.write(`[build] zh-Hant：opencc 全量生成 ${hantGenerated} 条，人工译文覆盖 ${hantManual} 条\n`)
  } else {
    process.stdout.write(`[build] zh-Hant：仅人工译文 ${hantManual} 条（OPENCC_MISSING）\n`)
  }
}

await main()
