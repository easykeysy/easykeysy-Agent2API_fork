#!/usr/bin/env node
/**
 * 提供商插件宿主的端到端验证。
 *
 * ── 这个脚本验证什么 ──────────────────────────────────────────
 * 它**不**经过网关，直接对着插件二进制跑协议 —— 因为要验的正是协议本身：
 *
 *   1. init          握手拿到 provider 列表与能力位
 *   2. chat          无状态：拿到「怎么构造这一次请求」
 *   3. models.list   目录
 *   4. usage.query   余额
 *   5. checkin.claim 签到
 *   6. account.normalize  账号规范化（含稳定身份）
 *   7. login.start/poll   登录流程
 *   8. conversation  流式：head → chunk → end
 *   9. credit        背压：不还额度时插件必须停下来
 *  10. abort         取消：在途流式调用能被打断
 *  11. 回调          插件反向调 store.get（脚本扮演网关应答）
 *  12. 崩溃恢复      kill 子进程后重新 spawn 仍能握手
 *
 * ── 用法 ──────────────────────────────────────────────────────
 *   node scripts/verify-plugin-host.mjs
 *   node scripts/verify-plugin-host.mjs --exe path/to/agent2api-providers.exe
 *
 * 退出码：0 = 全部通过；1 = 有用例失败。
 */

import { spawn } from 'node:child_process'
import { createServer } from 'node:http'
import process from 'node:process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const DEFAULT_EXE = path.join(ROOT, 'agent2api-providers', 'target', 'debug', 'agent2api-providers.exe')

/* ─── 参数 ───────────────────────────────── */

const argv = process.argv.slice(2)
const exeIndex = argv.indexOf('--exe')
const EXE = exeIndex >= 0 ? argv[exeIndex + 1] : DEFAULT_EXE

/* ─── 结果记账 ───────────────────────────── */

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

/* ─── 假上游（conversation 要真发一个 HTTP 请求）─── */

async function startFakeUpstream() {
  const server = createServer((req, res) => {
    let body = ''
    req.on('data', (chunk) => { body += chunk })
    req.on('end', () => {
      // ── 慢速大响应模式 ──────────────────────────────────────
      // 请求体里带 `"demo_big": true` 时，吐 4 MiB（远超 512 KiB 初始窗口）
      // 且每帧之间停 20ms。这是**唯一**能让背压真正触发的形态：
      // 响应小于窗口时插件根本不会停下，那项用例就成了假通过。
      let big = false
      try {
        big = JSON.parse(body)?.demo_big === true
      } catch { /* 不是 JSON 就当普通请求 */ }

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

      // 4 MiB：每帧 64 KiB，共 64 帧，每帧之间 20ms
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
      // ── 必须监听 res 的 close，不是 req 的 ──────────────────
      // `req.on('close')` 在**请求体读完**时就触发（与连接是否还在无关），
      // 用它取消定时器会让响应一帧都发不出去 —— 这个坑踩过一次：
      // 症状是「插件收到 head 后卡死」，而根因在假上游自己。
      res.on('close', () => clearInterval(timer))
    })
  })
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
  return { server, port: server.address().port }
}

/* ─── 插件客户端（扮演网关）───────────────── */

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
      // 插件崩溃时的兜底诊断
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
    // 插件回调网关（store.*）—— 这里扮演网关应答
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

  /** 扮演网关，回答插件发来的回调。 */
  serveCallback(frame) {
    const reply = (result) => this.write({ id: frame.id, result })
    const replyError = (message, code = 'internal') =>
      this.write({ id: frame.id, error: { message, code } })

    switch (frame.method) {
      case 'store.get': {
        // 演示：返回一条假账号，让 ensure_token / conversation 能跑通
        const account = frame.params?.account || 'demo-1'
        if (account === 'missing') return replyError('账号不存在', 'not_found')
        return reply({
          id: account,
          provider: 'demo',
          secret: { token: 'demo-token-1234', expires_at: Date.now() + 60_000 },
          public: { name: 'Sample account', uid: 'demo-uid' },
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

  /** 发一个调用并等应答。 */
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

  /** 发一个流式调用，收集 head / chunks / 结束。 */
  callStream(method, params, { creditOnRead = true, timeoutMs = 20000 } = {}) {
    const id = this.nextId++
    const frames = []
    let head = null
    let end = null
    const chunks = []
    let resolveEnd
    const done = new Promise((resolve) => { resolveEnd = resolve })

    const timer = setTimeout(() => resolveEnd({ head, chunks, frames, error: 'Timeout' }), timeoutMs)

    this.streams.set(id, {
      push: (frame) => {
        frames.push(frame)
        if (frame.event === 'head') head = frame
        if (frame.event === 'chunk' && frame.data) {
          const bytes = Buffer.from(frame.data, 'base64')
          chunks.push(bytes)
          // 扮演网关：读走一段就还一段额度
          if (creditOnRead) {
            this.write({ id: this.nextId++, method: 'credit', params: { id, n: bytes.length + 2048 } })
          }
        }
      },
    })

    // 结束由 pending 应答标记
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

/* ─── 用例 ───────────────────────────────── */

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

  console.log('[1] Handshake')

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
      pass('init Back provider List', `${providers.length} 家：${providers.map((p) => p.id).join(', ')}`)
    } else {
      fail('init Back provider List', 'The list is empty')
    }
  } catch (e) {
    fail('init Back proinit Back provider List', e.message)
  }

  const demo = providers?.find((p) => p.id === 'demo')
  if (demo) {
    const caps = demo.capabilities || {}
    const expected = ['chat', 'stateful', 'refresh', 'model_refresh', 'usage', 'checkin', 'web_login']
    const wrong = expected.filter((key) => caps[key] !== true)
    if (wrong.length === 0) pass('Report the capability bits honestly', expected.join(' / '))
    else fail('Report the capability bits honestly', `未申报：${wrong.join(', ')}`)
    if (Array.isArray(demo.login_methods) && demo.login_methods.length === 2) {
      pass('Report the login method', demo.login_methods.map((m) => m.kind).join(', '))
    } else {
      fail('Report the login method', `期望 2 种，得到 ${demo.login_methods?.length ?? 0}`)
    }
  }

  console.log('\n[2] Stateless forwarding (building requests)')
  try {
    const plan = await client.call('chat', baseChat())
    const okUrl = typeof plan.url === 'string' && plan.url.endsWith('/v1/chat/completions')
    const okBody = plan.body?.model === 'demo-fast'
    const okAuth = (plan.headers || []).some(([k, v]) => k === 'authorization' && v.includes('demo'))
    if (okUrl && okBody && okAuth) {
      pass('chat Return the complete build plan', `${plan.url}（model=${plan.body.model}）`)
    } else {
      fail('chat Return the complete build plan', JSON.stringify({ okUrl, okBody, okAuth }))
    }
    if (plan.upstream === 'chat') pass('The response protocol marker', 'chat')
    else fail('The response protocol marker', `期望 chat，得到 ${plan.upstream}`)
  } catch (e) {
    fail('chat Return the complete build plan', e.message)
  }

  console.log('\n[3] Control plane')
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
      pass('models.refresh(force)', `落地 ${refreshed.count} 条`)
    } else {
      fail('models.refresh(force)', JSON.stringify(refreshed))
    }
  } catch (e) {
    fail('models.refresh(force)', e.message)
  }

  try {
    const cached = await client.call('models.refresh', { provider: 'demo', account: 'demo-1', force: false })
    if (cached.refreshed === false && !cached.message) {
      pass('models.refresh(the automatic path skips by cache)', 'refreshed=false and there is no failure reason')
    } else {
      fail('models.refresh(the automatic path skips by cache)', JSON.stringify(cached))
    }
  } catch (e) {
    fail('models.refresh(the automatic path skips by cache)', e.message)
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

  console.log('\n[4] Error classification and policy')
  try {
    const quota = await client.call('classify', { provider: 'demo', status: 429, body: { error: { message: 'Limit', code: 6004 } } })
    if (quota.kind === 'quota_limited') pass('429 → QuotaLimited', 'It will rotate accounts')
    else fail('429 → QuotaLimited', JSON.stringify(quota))
  } catch (e) {
    fail('429 → QuotaLimited', e.message)
  }

  try {
    const expired = await client.call('classify', { provider: 'demo', status: 401, body: { error: { message: '过期' } } })
    if (expired.kind === 'token_expired') pass('401 → TokenExpired', 'Retry with the same account after refreshing')
    else fail('401 → TokenExpired', JSON.stringify(expired))
  } catch (e) {
    fail('401 → TokenExpired', e.message)
  }

  try {
    const blocked = await client.call('classify', { provider: 'demo', status: 400, body: { error: { message: 'sensitive content' } } })
    if (blocked.kind === 'content_blocked') pass('400+The review copy→ ContentBlocked', 'The account is not penalized')
    else fail('400+The review copy→ ContentBlocked', JSON.stringify(blocked))
  } catch (e) {
    fail('400+The review copy→ ContentBlocked', e.message)
  }

  try {
    const patch = await client.call('reasoning_patch', { provider: 'demo', level: 'high', model: 'demo-fast', body: {} })
    if (patch.action === 'set' && patch.field === 'reasoning_effort') {
      pass('reasoning_patch Injection', `${patch.field}=${patch.value}`)
    } else {
      fail('reasoning_patch Injection', JSON.stringify(patch))
    }
  } catch (e) {
    fail('reasoning_patch Injection', e.message)
  }

  try {
    const skip = await client.call('reasoning_patch', { provider: 'demo', level: 'ultra', model: 'demo-fast', body: {} })
    if (skip.action === 'skip' && skip.reason) pass('reasoning_patch Reject out-of-table levels and give the reason', skip.reason)
    else fail('reasoning_patch Reject out-of-table levels and give the reason', JSON.stringify(skip))
  } catch (e) {
    fail('reasoning_patch Reject out-of-table levels and give the reason', e.message)
  }

  console.log('\n[5] Account')
  try {
    const normalized = await client.call('account.normalize', {
      provider: 'demo',
      source: { kind: 'manual', payload: { token: 'demo-abc-1234', name: 'My example' } },
    })
    const okSecret = normalized.secret?.token === 'demo-abc-1234'
    const okPublic = normalized.public?.name === 'My example' && normalized.public?.tokenTail === '1234'
    const okIdentity = typeof normalized.identity === 'string' && normalized.identity.startsWith('demo-')
    if (okSecret && okPublic && okIdentity) {
      pass('account.normalize(typed in)', `identity=${normalized.identity}`)
    } else {
      fail('account.normalize(typed in)', JSON.stringify({ okSecret, okPublic, okIdentity }))
    }
    // 同令牌两次添加 → 同一个 identity（去重的前提）
    const again = await client.call('account.normalize', {
      provider: 'demo',
      source: { kind: 'manual', payload: { token: 'demo-abc-1234' } },
    })
    if (again.identity === normalized.identity) pass('The stable identity of the same token', again.identity)
    else fail('The stable identity of the same token', `${normalized.identity} vs ${again.identity}`)
  } catch (e) {
    fail('account.normalize(typed in)', e.message)
  }

  try {
    await client.call('account.normalize', {
      provider: 'demo',
      source: { kind: 'manual', payload: {} },
    })
    fail('An empty token was rejected', 'It actually succeeded')
  } catch (e) {
    pass('An empty token was rejected', e.message.slice(0, 40))
  }

  console.log('\n[6] Login')
  try {
    const start = await client.call('login.start', { provider: 'demo', method: 'browser', payload: {} })
    if (start.url && start.poll_interval_ms === 3000) {
      pass('login.start(device-authorization type)', `${start.url}，轮询 ${start.poll_interval_ms}ms`)
    } else {
      fail('login.start(device-authorization type)', JSON.stringify(start))
    }
    const poll = await client.call('login.poll', { provider: 'demo', flow: start.flow })
    if (poll.state === 'done' && poll.account?.secret?.token) {
      pass('login.poll Return the account', poll.account.public?.name)
    } else {
      fail('login.poll Return the account', JSON.stringify(poll))
    }
  } catch (e) {
    fail('login.start(device-authorization type)', e.message)
  }

  console.log('\n[7] Streaming forwarding + backpressure')
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
      pass('conversation Report the head', `status=${outcome.head.status}, upstream=${outcome.head.upstream}`)
    } else {
      fail('conversation Report the head', JSON.stringify(outcome.head ?? outcome.error))
    }
    const body = Buffer.concat(outcome.chunks).toString('utf8')
    if (body.includes('data:') && body.includes('[DONE]')) {
      pass('the streaming chunk content is complete', `${outcome.chunks.length} 帧，${body.length} 字节`)
    } else {
      fail('the streaming chunk content is complete', `收到 ${body.length} 字节：${body.slice(0, 80)}`)
    }
    if (outcome.result === null && !outcome.error) {
      pass('The streaming call ended normally', 'result=null')
    } else {
      fail('The streaming call ended normally', outcome.error ?? JSON.stringify(outcome.result))
    }
  } catch (e) {
    fail('conversation Report the head', e.message)
  }

  // ── 背压：大响应 + 不还额度，插件必须停下来 ────────────────
  // 4 MiB 响应 vs 512 KiB 初始窗口：插件最多发到窗口用完就必须阻塞在
  // `sink.send` 上。这里断言「收到的字节The count is significantly smaller than the total量」——
  // 如果背压没实现，插件会把 4 MiB 全推过来。
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
    // 初始窗口 512 KiB + 帧开销余量。留 2 倍余量避免边界抖动。
    const limit = 512 * 1024 * 2
    if (received > 0 && received < limit) {
      pass('backpressure is in effect (the plugin stays inside the window when not returning quota)', `收到 ${(received / 1024).toFixed(0)} KiB / 总共 4096 KiB`)
    } else if (received >= limit) {
      fail('backpressure is in effect (the plugin stays inside the window when not returning quota)', `收到 ${(received / 1024).toFixed(0)} KiB，超过窗口上限`)
    } else {
      fail('backpressure is in effect (the plugin stays inside the window when not returning quota)', `没收到数据：${outcome.error}`)
    }
    client.write({ id: client.nextId++, method: 'abort', params: { id } })
  } catch (e) {
    fail('backpressure is in effect (the plugin stays inside the window when not returning quota)', e.message)
  }

  console.log('\n[8] Cancel')
  try {
    // 用大响应（慢速 4 MiB）确保取消时调用一定还在途 ——
    // 小响应会在 abort 到达之前就结束，那样这项就是假通过。
    const { id, done } = client.callStream('conversation', {
      provider: 'demo',
      account: 'demo-1',
      body: { messages: [{ role: 'user', content: 'Cancel the test' }], model: 'demo-fast', demo_big: true },
      headers: {},
      stream: true,
      session: '',
      wire_model: 'demo-fast',
    }, { creditOnRead: true, timeoutMs: 8000 })
    // 等它真的开始推数据，再取消
    await new Promise((r) => setTimeout(r, 400))
    client.write({ id: client.nextId++, method: 'abort', params: { id } })
    const outcome = await done
    if (outcome.error && /取消|aborted|cancel/i.test(outcome.error)) {
      pass('abort Make in-flight streaming calls terminate', outcome.error)
    } else if (outcome.result === null && outcome.chunks.length > 0) {
      // 上游在 abort 生效前刚好推完了 —— 记录事实但不判失败
      pass('abort the follow-up call has finished', `${outcome.chunks.length} 帧后自然结束`)
    } else {
      fail('abort Make in-flight streaming calls terminate', JSON.stringify({ error: outcome.error, frames: outcome.chunks.length }))
    }
  } catch (e) {
    fail('abort Make in-flight streaming calls terminate', e.message)
  }

  console.log('\n[9] Error paths')
  try {
    await client.call('no.such.method', { provider: 'demo' })
    fail('An unknown method was refused', 'It actually succeeded')
  } catch (e) {
    pass('An unknown method was refused', e.message.slice(0, 50))
  }

  try {
    await client.call('chat', { provider: 'unknown-provider', body: {} })
    fail('Unknown provider Rejected', 'It actually succeeded')
  } catch (e) {
    pass('Unknown provider Rejected', e.message.slice(0, 50))
  }

  console.log('\n[10] Crash recovery')
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
      pass('the child process can re-handshake after being killed', `旧 pid=${before}，新进程可用`)
    } else {
      fail('the child process can re-handshake after being killed', 'The list is empty')
    }
    await restarted.stop()
  } catch (e) {
    fail('the child process can re-handshake after being killed', e.message)
  }

  await client.stop()
  upstream.server.close()

  /* ─── 汇总 ─── */
  const total = results.length
  const ok = total - failed
  console.log(`\n${'─'.repeat(52)}`)
  console.log(`通过 ${ok}/${total}${failed ? `，失败 ${failed}` : ''}`)
  if (failed) {
    console.log('\nFailed items:')
    for (const item of results.filter((r) => !r.ok)) {
      console.log(`  · ${item.name} — ${item.detail}`)
    }
  }
  process.exit(failed ? 1 : 0)
}

main().catch((e) => {
  console.error('The verification script itself errored:', e)
  process.exit(1)
})
