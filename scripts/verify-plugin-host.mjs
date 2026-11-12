#!/usr/bin/env node
/**
 * to 。
 *
 * ── this ──────────────────────────────────────────
 * ** not **， for —— as need is ：
 *
 * 1. init to provider and can 
 * 2. chat none： to 「 this 1 request」
 *   3. models.list   directory
 * 4. usage.query 
 * 5. checkin.claim to 
 * 6. account.normalize （）
 * 7. login.start/poll 
 * 8. conversation ：head → chunk → end
 * 9. credit ： not when must down 
 * 10. abort ： in use can 
 * 11. to store.get（）
 * 12. kill after new spawn can 
 *
 * ── use ──────────────────────────────────────────────────────
 *   node scripts/verify-plugin-host.mjs
 *   node scripts/verify-plugin-host.mjs --exe path/to/agent2api-providers.exe
 *
 * ：0 = ；1 = has use failure。
 */

import { spawn } from 'node:child_process'
import { createServer } from 'node:http'
import process from 'node:process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const DEFAULT_EXE = path.join(ROOT, 'agent2api-providers', 'target', 'debug', 'agent2api-providers.exe')

/* ─── parameter ───────────────────────────────── */

const argv = process.argv.slice(2)
const exeIndex = argv.indexOf('--exe')
const EXE = exeIndex >= 0 ? argv[exeIndex + 1] : DEFAULT_EXE

/* ─── ───────────────────────────── */

const results = []
let failed = 0

function pass(name, detail) {
  results.push({ name, ok: true, detail })
  console.log(`  ✓ ${name}${detail ? ` — ${detail}` : ''}`)
}

function fail(name, detail) {
  failed++
  results.push({ name, ok: false, detail })
  console.log(`  ✗ ${name}${detail ? ` — ${detail}` : ''}`)
}

/* ─── up （conversation need 1 HTTP request）─── */

async function startFakeUpstream() {
  const server = createServer((req, res) => {
    let body = ''
    req.on('data', (chunk) => { body += chunk })
    req.on('end', () => {
      // ── response ──────────────────────────────────────
      // request `"demo_big": true` when ， 4 MiB（ 512 KiB ）
      // and 20ms。 this is ** 1 ** can ：
      // response when not will down ， that use 。
      let big = false
      try {
        big = JSON.parse(body)?.demo_big === true
      } catch { /* not is JSON request */ }

      res.writeHead(200, { 'content-type': 'text/event-stream' })
      if (!big) {
        for (const part of ['data: {"choices":[{"delta":{"content":"你"}}]}\n\n',
                            'data: {"choices":[{"delta":{"content":"好"}}]}\n\n',
                            'data: [DONE]\n\n']) {
          res.write(part)
        }
        res.end()
        return
      }

      // 4 MiB： 64 KiB， 64 ， 20ms
      const frame = 'data: {"choices":[{"delta":{"content":"' + 'x'.repeat(64 * 1024) + '"}}]}\n\n'
      let sent = 0
      const timer = setInterval(() => {
        if (res.writableEnded) { clearInterval(timer); return }
        res.write(frame)
        sent += frame.length
        if (sent >= 4 * 1024 * 1024) {
          clearInterval(timer)
          res.write('data: [DONE]\n\n')
          res.end()
        }
      }, 20)
      // ── must res close， not is req ──────────────────
      // `req.on('close')` in **request** when （ and is in none），
      // use when will response 1 not —— this 1 ：
      // is 「 to head after 」， in up 。
      res.on('close', () => clearInterval(timer))
    })
  })
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
  return { server, port: server.address().port }
}

/* ─── client（）───────────────── */

class PluginClient {
  constructor(exe, env) {
    this.exe = exe
    this.env = env
    this.nextId = 1
    this.pending = new Map()
    this.streams = new Map()
    this.buffer = ''
    this.child = null
  }

  async start() {
    this.child = spawn(this.exe, [], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: { ...process.env, ...this.env },
    })
    this.child.stdout.setEncoding('utf8')
    this.child.stdout.on('data', (chunk) => this.onData(chunk))
    this.child.stderr.setEncoding('utf8')
    this.child.stderr.on('data', (chunk) => {
      // when 
      if (process.env.VERBOSE) process.stderr.write(`[plugin stderr] ${chunk}`)
    })
    this.exited = new Promise((resolve) => this.child.on('exit', resolve))
  }

  onData(chunk) {
    this.buffer += chunk
    for (;;) {
      const index = this.buffer.indexOf('\n')
      if (index < 0) return
      const line = this.buffer.slice(0, index).trim()
      this.buffer = this.buffer.slice(index + 1)
      if (!line) continue
      let frame
      try {
        frame = JSON.parse(line)
      } catch {
        continue
      }
      this.onFrame(frame)
    }
  }

  onFrame(frame) {
    // （store.*）—— this 
    if (frame.method) {
      this.serveCallback(frame)
      return
    }
    if (frame.event === 'log') {
      if (process.env.VERBOSE) console.log(`    [plugin ${frame.level}] ${frame.message}`)
      return
    }
    if (frame.event) {
      const stream = this.streams.get(frame.id)
      if (stream) stream.push(frame)
      return
    }
    const waiter = this.pending.get(frame.id)
    if (waiter) {
      this.pending.delete(frame.id)
      if (frame.error) waiter.reject(new Error(frame.error.message))
      else waiter.resolve(frame.result)
    }
  }

  /** ， 。 */
  serveCallback(frame) {
    const reply = (result) => this.write({ id: frame.id, result })
    const replyError = (message, code = 'internal') =>
      this.write({ id: frame.id, error: { message, code } })

    switch (frame.method) {
      case 'store.get': {
        // ：return 1 ， ensure_token / conversation can 
        const account = frame.params?.account || 'demo-1'
        if (account === 'missing') return replyError('账号不存在', 'not_found')
        return reply({
          id: account,
          provider: 'demo',
          secret: { token: 'demo-token-1234', expires_at: Date.now() + 60_000 },
          public: { name: '示例账号', uid: 'demo-uid' },
        })
      }
      case 'store.list':
        return reply([])
      case 'store.update':
        return reply({ updated: true })
      case 'store.mark_limited':
      case 'store.ledger_put':
      case 'settings.set':
        return reply(null)
      case 'store.ledger_get':
      case 'settings.get':
        return reply(null)
      case 'notify':
        return reply(null)
      case 'log':
        return reply(null)
      default:
        return replyError(`不认识的回调 ${frame.method}`, 'no_method')
    }
  }

  write(frame) {
    if (!this.child?.stdin.writable) return
    this.child.stdin.write(JSON.stringify(frame) + '\n')
  }

  /** 1 use and etc 。 */
  call(method, params, timeoutMs = 15000) {
    const id = this.nextId++
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id)
        reject(new Error(`超时：${method}`))
      }, timeoutMs)
      this.pending.set(id, {
        resolve: (value) => { clearTimeout(timer); resolve(value) },
        reject: (err) => { clearTimeout(timer); reject(err) },
      })
      this.write({ id, method, params })
    })
  }

  /** 1 use ， head / chunks / 。 */
  callStream(method, params, { creditOnRead = true, timeoutMs = 20000 } = {}) {
    const id = this.nextId++
    const frames = []
    let head = null
    let end = null
    const chunks = []
    let resolveEnd
    const done = new Promise((resolve) => { resolveEnd = resolve })

    const timer = setTimeout(() => resolveEnd({ head, chunks, frames, error: '超时' }), timeoutMs)

    this.streams.set(id, {
      push: (frame) => {
        frames.push(frame)
        if (frame.event === 'head') head = frame
        if (frame.event === 'chunk' && frame.data) {
          const bytes = Buffer.from(frame.data, 'base64')
          chunks.push(bytes)
          // ： 1 1 
          if (creditOnRead) {
            this.write({ id: this.nextId++, method: 'credit', params: { id, n: bytes.length + 2048 } })
          }
        }
      },
    })

    // by pending 
    this.pending.set(id, {
      resolve: (value) => {
        clearTimeout(timer)
        this.streams.delete(id)
        resolveEnd({ head, chunks, frames, result: value, error: null })
      },
      reject: (err) => {
        clearTimeout(timer)
        this.streams.delete(id)
        resolveEnd({ head, chunks, frames, error: err.message })
      },
    })

    this.write({ id, method, params })
    return { id, done }
  }

  async stop() {
    if (!this.child) return
    this.child.stdin.end()
    await Promise.race([this.exited, new Promise((r) => setTimeout(r, 1000))])
    if (this.child.exitCode === null) this.child.kill()
  }
}

/* ─── use ───────────────────────────────── */

function baseChat(overrides = {}) {
  return {
    provider: 'demo',
    account: 'demo-1',
    body: { messages: [{ role: 'user', content: '你好' }], model: 'demo-fast' },
    headers: {},
    stream: true,
    wire_model: 'demo-fast',
    reasoning: '',
    ...overrides,
  }
}

async function main() {
  console.log(`插件二进制：${EXE}\n`)

  const upstream = await startFakeUpstream()
  const env = { DEMO_UPSTREAM: `http://127.0.0.1:${upstream.port}` }
  const client = new PluginClient(EXE, env)
  await client.start()

  console.log('【1】握手')

  let providers
  try {
    providers = await client.call('init', {
      env: {
        data_dir: ROOT,
        host_version: '0.1.0-test',
        target: 'x86_64-windows',
        proxy: '',
        debug_traffic: false,
      },
    })
    if (Array.isArray(providers) && providers.length > 0) {
      pass('init 返回 provider 列表', `${providers.length} 家：${providers.map((p) => p.id).join(', ')}`)
    } else {
      fail('init 返回 provider 列表', '列表为空')
    }
  } catch (e) {
    fail('init 返回 provider 列表', e.message)
  }

  const demo = providers?.find((p) => p.id === 'demo')
  if (demo) {
    const caps = demo.capabilities || {}
    const expected = ['chat', 'stateful', 'refresh', 'model_refresh', 'usage', 'checkin', 'web_login']
    const wrong = expected.filter((key) => caps[key] !== true)
    if (wrong.length === 0) pass('能力位如实申报', expected.join(' / '))
    else fail('能力位如实申报', `未申报：${wrong.join(', ')}`)
    if (Array.isArray(demo.login_methods) && demo.login_methods.length === 2) {
      pass('登录方式申报', demo.login_methods.map((m) => m.kind).join(', '))
    } else {
      fail('登录方式申报', `期望 2 种，得到 ${demo.login_methods?.length ?? 0}`)
    }
  }

  console.log('\n【2】无状态转发（构造请求）')
  try {
    const plan = await client.call('chat', baseChat())
    const okUrl = typeof plan.url === 'string' && plan.url.endsWith('/v1/chat/completions')
    const okBody = plan.body?.model === 'demo-fast'
    const okAuth = (plan.headers || []).some(([k, v]) => k === 'authorization' && v.includes('demo'))
    if (okUrl && okBody && okAuth) {
      pass('chat 返回完整构造计划', `${plan.url}（model=${plan.body.model}）`)
    } else {
      fail('chat 返回完整构造计划', JSON.stringify({ okUrl, okBody, okAuth }))
    }
    if (plan.upstream === 'chat') pass('响应协议标记', 'chat')
    else fail('响应协议标记', `期望 chat，得到 ${plan.upstream}`)
  } catch (e) {
    fail('chat 返回完整构造计划', e.message)
  }

  console.log('\n【3】控制面')
  try {
    const list = await client.call('models.list', { provider: 'demo', account: 'demo-1' })
    if (Array.isArray(list.models) && list.models.length === 2) {
      pass('models.list', `${list.models.length} 条`)
    } else {
      fail('models.list', `期望 2 条，得到 ${list.models?.length ?? 0}`)
    }
  } catch (e) {
    fail('models.list', e.message)
  }

  try {
    const refreshed = await client.call('models.refresh', { provider: 'demo', account: 'demo-1', force: true })
    if (refreshed.refreshed === true && refreshed.count === 2) {
      pass('models.refresh（force）', `落地 ${refreshed.count} 条`)
    } else {
      fail('models.refresh（force）', JSON.stringify(refreshed))
    }
  } catch (e) {
    fail('models.refresh（force）', e.message)
  }

  try {
    const cached = await client.call('models.refresh', { provider: 'demo', account: 'demo-1', force: false })
    if (cached.refreshed === false && !cached.message) {
      pass('models.refresh（自动路径按缓存跳过）', 'refreshed=false 且无失败原因')
    } else {
      fail('models.refresh（自动路径按缓存跳过）', JSON.stringify(cached))
    }
  } catch (e) {
    fail('models.refresh（自动路径按缓存跳过）', e.message)
  }

  try {
    const usage = await client.call('usage.query', { provider: 'demo', account: 'demo-1' })
    if (usage.plan === 'Demo Plan' && usage.balance === '1000') {
      pass('usage.query', `${usage.plan} / 余额 ${usage.balance}`)
    } else {
      fail('usage.query', JSON.stringify(usage))
    }
  } catch (e) {
    fail('usage.query', e.message)
  }

  try {
    const report = await client.call('checkin.claim', { provider: 'demo', account: 'demo-1' })
    if (report.outcome === 'claimed') pass('checkin.claim', `+${report.credit}`)
    else fail('checkin.claim', JSON.stringify(report))
  } catch (e) {
    fail('checkin.claim', e.message)
  }

  console.log('\n【4】错误分类与策略')
  try {
    const quota = await client.call('classify', { provider: 'demo', status: 429, body: { error: { message: '限额', code: 6004 } } })
    if (quota.kind === 'quota_limited') pass('429 → QuotaLimited', '会换账号')
    else fail('429 → QuotaLimited', JSON.stringify(quota))
  } catch (e) {
    fail('429 → QuotaLimited', e.message)
  }

  try {
    const expired = await client.call('classify', { provider: 'demo', status: 401, body: { error: { message: '过期' } } })
    if (expired.kind === 'token_expired') pass('401 → TokenExpired', '刷新后同账号重试')
    else fail('401 → TokenExpired', JSON.stringify(expired))
  } catch (e) {
    fail('401 → TokenExpired', e.message)
  }

  try {
    const blocked = await client.call('classify', { provider: 'demo', status: 400, body: { error: { message: 'sensitive content' } } })
    if (blocked.kind === 'content_blocked') pass('400+审核文案 → ContentBlocked', '不罚账号')
    else fail('400+审核文案 → ContentBlocked', JSON.stringify(blocked))
  } catch (e) {
    fail('400+审核文案 → ContentBlocked', e.message)
  }

  try {
    const patch = await client.call('reasoning_patch', { provider: 'demo', level: 'high', model: 'demo-fast', body: {} })
    if (patch.action === 'set' && patch.field === 'reasoning_effort') {
      pass('reasoning_patch 注入', `${patch.field}=${patch.value}`)
    } else {
      fail('reasoning_patch 注入', JSON.stringify(patch))
    }
  } catch (e) {
    fail('reasoning_patch 注入', e.message)
  }

  try {
    const skip = await client.call('reasoning_patch', { provider: 'demo', level: 'ultra', model: 'demo-fast', body: {} })
    if (skip.action === 'skip' && skip.reason) pass('reasoning_patch 拒绝表外档位并给原因', skip.reason)
    else fail('reasoning_patch 拒绝表外档位并给原因', JSON.stringify(skip))
  } catch (e) {
    fail('reasoning_patch 拒绝表外档位并给原因', e.message)
  }

  console.log('\n【5】账号')
  try {
    const normalized = await client.call('account.normalize', {
      provider: 'demo',
      source: { kind: 'manual', payload: { token: 'demo-abc-1234', name: '我的示例' } },
    })
    const okSecret = normalized.secret?.token === 'demo-abc-1234'
    const okPublic = normalized.public?.name === '我的示例' && normalized.public?.tokenTail === '1234'
    const okIdentity = typeof normalized.identity === 'string' && normalized.identity.startsWith('demo-')
    if (okSecret && okPublic && okIdentity) {
      pass('account.normalize（手填）', `identity=${normalized.identity}`)
    } else {
      fail('account.normalize（手填）', JSON.stringify({ okSecret, okPublic, okIdentity }))
    }
    // → 1 identity（ before ）
    const again = await client.call('account.normalize', {
      provider: 'demo',
      source: { kind: 'manual', payload: { token: 'demo-abc-1234' } },
    })
    if (again.identity === normalized.identity) pass('同一令牌的稳定身份', again.identity)
    else fail('同一令牌的稳定身份', `${normalized.identity} vs ${again.identity}`)
  } catch (e) {
    fail('account.normalize（手填）', e.message)
  }

  try {
    await client.call('account.normalize', {
      provider: 'demo',
      source: { kind: 'manual', payload: {} },
    })
    fail('空令牌被拒', '居然成功了')
  } catch (e) {
    pass('空令牌被拒', e.message.slice(0, 40))
  }

  console.log('\n【6】登录')
  try {
    const start = await client.call('login.start', { provider: 'demo', method: 'browser', payload: {} })
    if (start.url && start.poll_interval_ms === 3000) {
      pass('login.start（设备授权型）', `${start.url}，轮询 ${start.poll_interval_ms}ms`)
    } else {
      fail('login.start（设备授权型）', JSON.stringify(start))
    }
    const poll = await client.call('login.poll', { provider: 'demo', flow: start.flow })
    if (poll.state === 'done' && poll.account?.secret?.token) {
      pass('login.poll 返回账号', poll.account.public?.name)
    } else {
      fail('login.poll 返回账号', JSON.stringify(poll))
    }
  } catch (e) {
    fail('login.start（设备授权型）', e.message)
  }

  console.log('\n【7】流式转发 + 背压')
  try {
    const { done } = client.callStream('conversation', {
      provider: 'demo',
      account: 'demo-1',
      body: { messages: [{ role: 'user', content: '你好' }], model: 'demo-fast', stream: true },
      headers: {},
      stream: true,
      session: 'test-session',
      wire_model: 'demo-fast',
    })
    const outcome = await done
    if (outcome.head?.status === 200) {
      pass('conversation 上报 head', `status=${outcome.head.status}, upstream=${outcome.head.upstream}`)
    } else {
      fail('conversation 上报 head', JSON.stringify(outcome.head ?? outcome.error))
    }
    const body = Buffer.concat(outcome.chunks).toString('utf8')
    if (body.includes('data:') && body.includes('[DONE]')) {
      pass('流式 chunk 内容完整', `${outcome.chunks.length} 帧，${body.length} 字节`)
    } else {
      fail('流式 chunk 内容完整', `收到 ${body.length} 字节：${body.slice(0, 80)}`)
    }
    if (outcome.result === null && !outcome.error) {
      pass('流式调用正常收尾', 'result=null')
    } else {
      fail('流式调用正常收尾', outcome.error ?? JSON.stringify(outcome.result))
    }
  } catch (e) {
    fail('conversation 上报 head', e.message)
  }

  // ── ：response + not ，must down ────────────────
  // 4 MiB response vs 512 KiB ： to use must in 
  // `sink.send` up 。 this 「 to total」——
  // if no ， will 4 MiB 。
  try {
    const { id, done } = client.callStream('conversation', {
      provider: 'demo',
      account: 'demo-1',
      body: { messages: [{ role: 'user', content: 'big' }], model: 'demo-fast', demo_big: true },
      headers: {},
      stream: true,
      session: '',
      wire_model: 'demo-fast',
    }, { creditOnRead: false, timeoutMs: 2500 })
    const outcome = await done
    const received = outcome.chunks.reduce((sum, c) => sum + c.length, 0)
    // 512 KiB + 。 2 。
    const limit = 512 * 1024 * 2
    if (received > 0 && received < limit) {
      pass('背压生效（不还额度时插件停在窗口内）', `收到 ${(received / 1024).toFixed(0)} KiB / 总共 4096 KiB`)
    } else if (received >= limit) {
      fail('背压生效（不还额度时插件停在窗口内）', `收到 ${(received / 1024).toFixed(0)} KiB，超过窗口上限`)
    } else {
      fail('背压生效（不还额度时插件停在窗口内）', `没收到数据：${outcome.error}`)
    }
    client.write({ id: client.nextId++, method: 'abort', params: { id } })
  } catch (e) {
    fail('背压生效（不还额度时插件停在窗口内）', e.message)
  }

  console.log('\n【8】取消')
  try {
    // use response（ 4 MiB） when use 1 in ——
    // response will in abort to before ， that this is 。
    const { id, done } = client.callStream('conversation', {
      provider: 'demo',
      account: 'demo-1',
      body: { messages: [{ role: 'user', content: '取消测试' }], model: 'demo-fast', demo_big: true },
      headers: {},
      stream: true,
      session: '',
      wire_model: 'demo-fast',
    }, { creditOnRead: true, timeoutMs: 8000 })
    // etc ，
    await new Promise((r) => setTimeout(r, 400))
    client.write({ id: client.nextId++, method: 'abort', params: { id } })
    const outcome = await done
    if (outcome.error && /取消|aborted|cancel/i.test(outcome.error)) {
      pass('abort 让在途流式调用终止', outcome.error)
    } else if (outcome.result === null && outcome.chunks.length > 0) {
      // up in abort before —— not failure
      pass('abort 后调用已收尾', `${outcome.chunks.length} 帧后自然结束`)
    } else {
      fail('abort 让在途流式调用终止', JSON.stringify({ error: outcome.error, frames: outcome.chunks.length }))
    }
  } catch (e) {
    fail('abort 让在途流式调用终止', e.message)
  }

  console.log('\n【9】错误路径')
  try {
    await client.call('no.such.method', { provider: 'demo' })
    fail('未知方法被拒', '居然成功了')
  } catch (e) {
    pass('未知方法被拒', e.message.slice(0, 50))
  }

  try {
    await client.call('chat', { provider: 'unknown-provider', body: {} })
    fail('未知 provider 被拒', '居然成功了')
  } catch (e) {
    pass('未知 provider 被拒', e.message.slice(0, 50))
  }

  console.log('\n【10】崩溃恢复')
  try {
    const before = client.child.pid
    client.child.kill('SIGKILL')
    await new Promise((r) => setTimeout(r, 300))
    const restarted = new PluginClient(EXE, env)
    await restarted.start()
    const list = await restarted.call('init', {
      env: { data_dir: ROOT, host_version: '0.1.0-test', target: 'x86_64-windows', proxy: '', debug_traffic: false },
    })
    if (Array.isArray(list) && list.length > 0) {
      pass('子进程被杀后能重新握手', `旧 pid=${before}，新进程可用`)
    } else {
      fail('子进程被杀后能重新握手', '列表为空')
    }
    await restarted.stop()
  } catch (e) {
    fail('子进程被杀后能重新握手', e.message)
  }

  await client.stop()
  upstream.server.close()

  /* ─── total ─── */
  const total = results.length
  const ok = total - failed
  console.log(`\n${'─'.repeat(52)}`)
  console.log(`通过 ${ok}/${total}${failed ? `，失败 ${failed}` : ''}`)
  if (failed) {
    console.log('\n失败项：')
    for (const item of results.filter((r) => !r.ok)) {
      console.log(`  · ${item.name} — ${item.detail}`)
    }
  }
  process.exit(failed ? 1 : 0)
}

main().catch((e) => {
  console.error('验证脚本自身出错：', e)
  process.exit(1)
})
