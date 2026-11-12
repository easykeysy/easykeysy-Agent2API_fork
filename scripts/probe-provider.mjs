#!/usr/bin/env node
/**
 * can ： for **** 1 provider 。
 *
 * ── this ──────────────────────────────────────
 * 「 for ，get / / to no 」 this ， not will 、
 * up not 。 from **** by client ， to 
 * can ； `.agents/skills/provider-adaptation/SKILL.md`
 * 6 。
 *
 * ── Why not is up ──────────────────────────────
 * need is ****：request、、SSE 、error、usage 
 * in 。 up can 「 up 」， not 「 this for 」。
 * this `/v1/chat/completions`， and client 1 —— 
 * ** will **。
 *
 * ── before ： 1 provider Key ──────────────────
 * 1 can can in has ， not provider when request will by to ，
 * none。 when will `/v1/models` 「 this by this 」，
 * for not up （ 2）， not will 1 。
 *
 * use ：
 *   node scripts/probe-provider.mjs --provider workbuddy --model glm-5.3 --key sk-dev-xxx
 *   node scripts/probe-provider.mjs --cases stream,tool_call --json
 *
 * ：0 = no has requiredfailure；1 = has requiredfailure；2 = use / before error。
 */

import process from 'node:process'
import { CASES, CASE_ORDER } from './probe-cases.mjs'

/* ─── parameter ───────────────────────────────── */

const USAGE = `
提供商能力探测 —— 对着本机网关逐项验证一家 provider 的适配完整度

用法：
  node scripts/probe-provider.mjs --provider <id> --model <name> [选项]

必填：
  --provider <id>      provider id（如 workbuddy / modelscope / atomcode）
  --model <name>       要探测的模型名（必须是 /v1/models 里属于该家的名字）

选项：
  --base <url>         网关地址，默认 http://127.0.0.1:3065
                       也可用环境变量 AGENT2API_BASE
  --key <key>          网关 Key（建议只允许目标 provider）
                       也可用环境变量 AGENT2API_KEY
  --cases <a,b,...>    只跑指定用例，默认全部
  --timeout <sec>      单次请求超时，默认 ${120}
  --force              目录声明不支持时也照测
  --json               输出机器可读的 JSON
  --list               列出全部用例后退出
  --help               显示本帮助

用例（* = 必需项，失败时脚本以 1 退出）：
  * stream           流式 SSE 能逐帧下发
  * tool_call        tools 声明被接受，返回 tool_calls
  * tool_roundtrip   工具结果回灌后能继续生成
    parallel_tools   一次返回多个 tool_calls
    image            图片输入被接受
    reasoning        思考字段能透出
    json_schema      结构化输出可用
    usage            终帧带非零 usage
    context_window   目录声明的上下文窗口（只读 /v1/models，不发上游）

示例：
  node scripts/probe-provider.mjs --provider atomcode --model GLM-5 --key sk-dev-atom
  node scripts/probe-provider.mjs --provider workbuddy --model glm-5.3 --cases stream,tool_call --json
`.trim()

/** request default when （seconds）。 is ， to 。 */
const DEFAULT_TIMEOUT_SEC = 120

/** use / before error： 1 and 2 （ and 「failure」）。 */
function fail(message) {
  console.error(`\n✗ ${message}\n`)
  process.exitCode = 2
  throw new Error('__exit__')
}

function parseArgs(argv) {
  const options = {
    base: process.env.AGENT2API_BASE ?? 'http://127.0.0.1:3065',
    key: process.env.AGENT2API_KEY ?? '',
    provider: '',
    model: '',
    cases: null,
    timeoutSec: DEFAULT_TIMEOUT_SEC,
    force: false,
    json: false,
    list: false,
    help: false,
  }
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]
    const next = () => {
      const value = argv[index + 1]
      if (value === undefined || value.startsWith('--')) {
        fail(`参数 ${arg} 缺少取值`)
      }
      index += 1
      return value
    }
    switch (arg) {
      case '--base': options.base = next(); break
      case '--key': options.key = next(); break
      case '--provider': options.provider = next(); break
      case '--model': options.model = next(); break
      case '--cases': options.cases = next().split(',').map(s => s.trim()).filter(Boolean); break
      case '--timeout': options.timeoutSec = Number(next()); break
      case '--force': options.force = true; break
      case '--json': options.json = true; break
      case '--list': options.list = true; break
      case '--help': case '-h': options.help = true; break
      default: fail(`未知参数 ${arg}（用 --help 看用法）`)
    }
  }
  if (!Number.isFinite(options.timeoutSec) || options.timeoutSec <= 0) {
    fail('--timeout 必须是正数（秒）')
  }
  options.base = options.base.replace(/\/+$/, '')
  return options
}

/* ─── HTTP ───────────────────────────────── */

function headersOf(options) {
  const headers = { 'content-type': 'application/json' }
  if (options.key) headers.authorization = `Bearer ${options.key}`
  return headers
}

async function getJson(options, path) {
  try {
    const res = await fetch(`${options.base}${path}`, {
      headers: headersOf(options),
      signal: AbortSignal.timeout(options.timeoutSec * 1000),
    })
    const text = await res.text()
    if (!res.ok) return { ok: false, status: res.status, text }
    try {
      return { ok: true, status: res.status, body: JSON.parse(text), text }
    } catch {
      return { ok: false, status: res.status, text }
    }
  } catch (error) {
    return { ok: false, status: 0, text: String(error?.message ?? error) }
  }
}

/** from error 1 can （OpenAI `{error:{message}}` ）。 */
function errorText(text) {
  try {
    const parsed = JSON.parse(text)
    const message = parsed?.error?.message ?? parsed?.message
    if (typeof message === 'string' && message) return message
  } catch {
    /* JSON： */
  }
  return String(text ?? '').slice(0, 600)
}

/**
 * SSE：return、 when and 。
 *
 * use `getReader()` not is `for await`：Node 18 ReadableStream 
 * ，reader is 。 by empty（`\n\n`）， and SSE 1 ；
 * down 1 is `data: {json}`， this not data 。
 */
async function readSse(res, startedAt) {
  const reader = res.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let raw = ''
  let ttfbMs = null
  const frames = []
  for (;;) {
    const { done, value } = await reader.read()
    if (done) break
    if (ttfbMs === null) ttfbMs = Date.now() - startedAt
    const chunk = decoder.decode(value, { stream: true })
    raw += chunk
    buffer += chunk
    let boundary = buffer.indexOf('\n\n')
    while (boundary !== -1) {
      const block = buffer.slice(0, boundary)
      buffer = buffer.slice(boundary + 2)
      for (const line of block.split('\n')) {
        if (!line.startsWith('data:')) continue
        const payload = line.slice(5).trim()
        if (!payload) continue
        if (payload === '[DONE]') {
          frames.push({ done: true })
          continue
        }
        try {
          frames.push({ data: JSON.parse(payload) })
        } catch {
          frames.push({ raw: payload })
        }
      }
      boundary = buffer.indexOf('\n\n')
    }
  }
  return { frames, ttfbMs, raw }
}

/**
 * 1 for request。
 *
 * `stream: false` when up SSE JSON（`choices[0].message`
 * tool_calls / reasoning_content / usage）—— use this ，
 * and etc 。 has `stream` and `usage` use need need 。
 */
async function chat(options, body) {
  const startedAt = Date.now()
  let res
  try {
    res = await fetch(`${options.base}/v1/chat/completions`, {
      method: 'POST',
      headers: headersOf(options),
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(options.timeoutSec * 1000),
    })
  } catch (error) {
    return { ok: false, error: `请求失败：${error?.message ?? error}` }
  }
  const durationMs = Date.now() - startedAt
  if (body.stream) {
    if (!res.ok) {
      const text = await res.text()
      return { ok: false, status: res.status, error: `${res.status} ${errorText(text)}`, durationMs }
    }
    const { frames, ttfbMs, raw } = await readSse(res, startedAt)
    return { ok: true, status: res.status, frames, ttfbMs, raw, durationMs }
  }
  const text = await res.text()
  if (!res.ok) {
    return { ok: false, status: res.status, error: `${res.status} ${errorText(text)}`, durationMs }
  }
  try {
    return { ok: true, status: res.status, body: JSON.parse(text), durationMs }
  } catch {
    return { ok: false, status: res.status, error: '响应不是 JSON', durationMs }
  }
}

/* ─── before check ───────────────────────────── */

/**
 * /v1/models，「 this by this 」。
 *
 * not ， in has when request will by to ，
 * none —— that not （ will to 「 this can use 」 error）。
 */
async function preflight(options) {
  const health = await getJson(options, '/health')
  if (!health.ok) {
    fail(
      `连不上网关 ${options.base}（${health.text ?? `HTTP ${health.status}`}）。\n` +
        '  确认服务在跑：桌面端启动应用，或服务端 / Docker 已就绪。',
    )
  }
  const models = await getJson(options, '/v1/models')
  if (!models.ok || !Array.isArray(models.body?.data)) {
    fail(`读取 /v1/models 失败（HTTP ${models.status}）：${errorText(models.text)}`)
  }
  const items = models.body.data
  const matched = items.filter(item => String(item?.id ?? '').toLowerCase() === options.model.toLowerCase())
  if (matched.length === 0) {
    const sample = items.slice(0, 8).map(item => item.id).join('、')
    fail(
      `/v1/models 里没有模型 ${options.model}。\n` +
        `  当前可见：${sample || '（空）'}${items.length > 8 ? ' …' : ''}\n` +
        '  先刷新清单：面板「模型管理」页点「获取模型」，或 POST /api/models/refresh。',
    )
  }
  const item = matched[0]
  const owner = String(item?.owned_by ?? '')
  if (options.provider && owner && owner !== options.provider) {
    fail(
      `模型 ${options.model} 由 ${owner} 承载，不是 --provider ${options.provider}。\n` +
        '  换一个该家自己的模型名，或确认 --provider 填的是 provider id（不是展示名）。',
    )
  }
  return {
    catalog: {
      id: item.id,
      owned_by: owner,
      max_input_tokens: item.max_input_tokens ?? null,
      max_output_tokens: item.max_output_tokens ?? null,
      supports_tool_call: item.supports_tool_call ?? null,
      supports_images: item.supports_images ?? null,
      supports_video: item.supports_video ?? null,
      supports_reasoning: item.supports_reasoning ?? null,
    },
  }
}

/* ─── ───────────────────────────────── */

const STATUS_MARK = { pass: '✓', fail: '✗', skip: '–', info: 'i' }

/** evidence 1 can need （ for ，）。 */
function summarize(evidence) {
  if (!evidence || typeof evidence !== 'object') return ''
  const parts = []
  for (const [key, value] of Object.entries(evidence)) {
    if (value === undefined || value === null) continue
    let text = typeof value === 'string' ? value : JSON.stringify(value)
    if (text.length > 60) text = `${text.slice(0, 57)}…`
    parts.push(`${key}=${text}`)
  }
  return parts.join(' ')
}

function detailOf(result) {
  return result.error ?? result.reason ?? summarize(result.evidence)
}

/* ─── ─────────────────────────────── */

async function main() {
  const options = parseArgs(process.argv.slice(2))
  if (options.help) {
    console.log(USAGE)
    return
  }
  if (options.list) {
    for (const id of CASE_ORDER) {
      const item = CASES[id]
      console.log(`${item.severity === 'required' ? '*' : ' '} ${id.padEnd(16)} ${item.label}`)
    }
    console.log('\n* = 必需项（失败时脚本以 1 退出）')
    return
  }
  if (!options.provider) fail('缺少 --provider（provider id，如 workbuddy）')
  if (!options.model) fail('缺少 --model（要探测的模型名）')

  const selected = options.cases ?? CASE_ORDER
  for (const id of selected) {
    if (!CASES[id]) fail(`未知用例 ${id}。可用：${CASE_ORDER.join('、')}`)
  }

  if (!options.key) {
    console.error(
      '⚠️  未提供 --key：无法钉住 provider。若同名模型在多家中存在，请求可能落到别家，结论不可归因。\n',
    )
  }

  const { catalog } = await preflight(options)

  if (!options.json) {
    console.log(`\n探测 ${options.provider} / ${options.model}  @ ${options.base}`)
    console.log(
      `目录声明：工具=${catalog.supports_tool_call} 图片=${catalog.supports_images} ` +
        `思考=${catalog.supports_reasoning} 上下文=${catalog.max_input_tokens ?? '—'}\n`,
    )
  }

  const results = []
  for (const id of selected) {
    const item = CASES[id]
    if (item.dependsOn) {
      const dependency = results.find(result => result.case === item.dependsOn)
      if (!dependency || dependency.status !== 'pass') {
        const skipped = { case: id, status: 'skip', reason: `前置用例 ${item.dependsOn} 未通过` }
        results.push(skipped)
        if (!options.json) console.log(`  – ${item.label.padEnd(10)} ${detailOf(skipped)}`)
        continue
      }
    }
    // in ： to to file when `\r` will down 。
    const interactive = !options.json && process.stdout.isTTY
    if (interactive) process.stdout.write(`  … ${item.label.padEnd(10)}\r`)
    let result
    try {
      result = await item.run({ options, model: options.model, catalog, chat })
    } catch (error) {
      result = { status: 'fail', error: `脚本异常：${error?.message ?? error}` }
    }
    results.push({ case: id, ...result })
    if (!options.json) {
      const mark = STATUS_MARK[result.status] ?? '?'
      console.log(`  ${mark} ${item.label.padEnd(10)} ${detailOf(result)}`)
    }
  }

  const requiredFailures = results.filter(
    result => CASES[result.case].severity === 'required' && result.status === 'fail',
  )
  const optionalFailures = results.filter(
    result => CASES[result.case].severity !== 'required' && result.status === 'fail',
  )

  if (options.json) {
    console.log(
      JSON.stringify(
        {
          provider: options.provider,
          model: options.model,
          base: options.base,
          catalog,
          results,
          summary: {
            pass: results.filter(result => result.status === 'pass').length,
            fail: results.filter(result => result.status === 'fail').length,
            skip: results.filter(result => result.status === 'skip').length,
            info: results.filter(result => result.status === 'info').length,
            requiredFailures: requiredFailures.map(result => result.case),
            optionalFailures: optionalFailures.map(result => result.case),
          },
        },
        null,
        2,
      ),
    )
  } else {
    const counts = results.reduce(
      (acc, result) => ({ ...acc, [result.status]: (acc[result.status] ?? 0) + 1 }),
      {},
    )
    console.log(
      `\n合计：通过 ${counts.pass ?? 0} · 失败 ${counts.fail ?? 0} · ` +
        `跳过 ${counts.skip ?? 0} · 提示 ${counts.info ?? 0}`,
    )
    if (requiredFailures.length > 0) {
      console.log(`\n✗ 必需项未通过：${requiredFailures.map(result => result.case).join('、')}`)
      console.log('  这家的「LLM 请求」不能算接完 —— 见 SKILL 第 6 节的验收口径。')
    } else if (optionalFailures.length > 0) {
      console.log(`\n提示：可选项未通过 ${optionalFailures.map(result => result.case).join('、')}（不阻断验收）`)
    } else {
      console.log('\n✓ 必需项全部通过。')
    }
    console.log('')
  }

  process.exitCode = requiredFailures.length > 0 ? 1 : 0
}

main().catch(error => {
  // `fail()` already ， this （）。
  if (error?.message !== '__exit__') {
    console.error(`\n✗ 脚本异常：${error?.stack ?? error}\n`)
    process.exitCode = 2
  }
})
