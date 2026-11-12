/**
 * can ** use **（ and CLI ， `probe-provider.mjs`）。
 *
 * use is 1 `run({ options, model, catalog, chat })`，return
 * `{ status, evidence?, error?, reason? }`：
 *   - `status`：`pass` / `fail` / `skip` / `info`
 * - `evidence`： can （ 1 need ，`--json` when ）
 * - `error` / `reason`： to 1 
 *
 * （Why this is required、 that is optional）
 * `.agents/skills/provider-adaptation/SKILL.md` 6 ：agent client for 1 
 * available is 「 + use + 」， its is 。
 *
 * `dependsOn` before use no （must to tool_calls）。
 */

/** use ：parameter， as schema not use 。 */
const WEATHER_TOOL = {
  type: 'function',
  function: {
    name: 'get_weather',
    description: '查询指定城市的当前天气',
    parameters: {
      type: 'object',
      properties: { city: { type: 'string', description: '城市名，例如 北京' } },
      required: ['city'],
    },
  },
}

const TIME_TOOL = {
  type: 'function',
  function: {
    name: 'get_local_time',
    description: '查询指定城市的当前本地时间',
    parameters: {
      type: 'object',
      properties: { city: { type: 'string', description: '城市名，例如 上海' } },
      required: ['city'],
    },
  },
}

/**
 * 64×64 PNG（178 ），， not 。
 *
 * use data URL not is can ： not this need this can ，
 * not this request to 。 unsupported up will 「 not 」，
 * not will as to and can none 413。
 */
const RED_PNG_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAAAeUlEQVR4nO3PQQkAMAzAwCqpf1ETMxF7HINABFzm7H7dcEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFj13PLIEAOXyUUwAAAABJRU5ErkJggg=='

/* ─── response（ ） ───────── */

/** `choices[0]`； no has then null。 */
function firstChoice(body) {
  return body?.choices?.[0] ?? null
}

/**
 * 。
 *
 * up to ；（Anthropic ） to ， this 
 * —— 「 has no has 」， not 。
 */
function contentOf(body) {
  const content = firstChoice(body)?.message?.content
  if (typeof content === 'string') return content
  if (Array.isArray(content)) {
    return content.map(part => (typeof part === 'string' ? part : part?.text ?? '')).join('')
  }
  return ''
}

function toolCallsOf(body) {
  const calls = firstChoice(body)?.message?.tool_calls
  return Array.isArray(calls) ? calls : []
}

/**
 * 。
 *
 * need ：`reasoning_content`（ OpenAI ）、`reasoning`
 * （）、 and for `{content:[{text}]}` / `{summary:[{text}]}`
 * （Responses ）。 will `reasoning_content`，
 * request can to 。
 */
function reasoningOf(body) {
  const message = firstChoice(body)?.message
  if (!message) return ''
  const value = message.reasoning_content ?? message.reasoning
  if (typeof value === 'string') return value
  if (value && typeof value === 'object') {
    const parts = [...(value.content ?? []), ...(value.summary ?? [])]
    return parts.map(part => part?.text ?? '').join('')
  }
  return ''
}

/* ─── use ───────────────────────────────── */

export const CASES = {
  stream: {
    label: '流式 SSE',
    severity: 'required',
    async run({ options, model, chat }) {
      const result = await chat(options, {
        model,
        stream: true,
        messages: [{ role: 'user', content: '数到三，只输出数字，用逗号分隔。' }],
      })
      if (!result.ok) return { status: 'fail', error: result.error }
      const dataFrames = result.frames.filter(frame => frame.data)
      const content = dataFrames
        .map(frame => frame.data?.choices?.[0]?.delta?.content ?? '')
        .join('')
      if (dataFrames.length < 2) {
        return {
          status: 'fail',
          error: `只收到 ${dataFrames.length} 个数据帧（流式应当逐帧下发）`,
          evidence: { frames: dataFrames.length, ttfbMs: result.ttfbMs },
        }
      }
      return {
        status: 'pass',
        evidence: {
          frames: dataFrames.length,
          ttfbMs: result.ttfbMs,
          durationMs: result.durationMs,
          reply: content.slice(0, 120),
        },
      }
    },
  },

  tool_call: {
    label: '工具调用',
    severity: 'required',
    async run({ options, model, catalog, chat }) {
      // 1 use ；failure after 1 —— is not ，
      // failure not down （ SKILL 6 ）。
      const prompts = [
        '请调用 get_weather 工具查询北京的天气，不要用文字回答。',
        '必须调用 get_weather 工具，参数 city 填「北京」。禁止直接回答，只允许发起工具调用。',
      ]
      let last = null
      for (const prompt of prompts) {
        last = await chat(options, {
          model,
          stream: false,
          messages: [{ role: 'user', content: prompt }],
          tools: [WEATHER_TOOL],
          tool_choice: 'auto',
        })
        if (!last.ok) return { status: 'fail', error: last.error }
        const matched = toolCallsOf(last.body).find(call => call?.function?.name === 'get_weather')
        if (matched) {
          return {
            status: 'pass',
            evidence: {
              finishReason: firstChoice(last.body)?.finish_reason,
              name: matched.function?.name,
              arguments: String(matched.function?.arguments ?? '').slice(0, 200),
              callId: matched.id ?? '',
            },
          }
        }
      }
      const detail = {
        finishReason: firstChoice(last?.body)?.finish_reason,
        toolCalls: toolCallsOf(last?.body ?? {}).length,
        reply: contentOf(last?.body ?? {}).slice(0, 160),
        catalogDeclares: catalog?.supports_tool_call ?? null,
      }
      if (catalog?.supports_tool_call === false && !options.force) {
        // directory****unsupported use ： this not is 「 not 」， is 「 this not 
        // agent client」。 for required is failure —— as skip will 
        // 「required」， that is （ SKILL 0 ）。
        return {
          status: 'fail',
          error: '/v1/models 声明该模型不支持工具调用：agent 客户端（Claude Code / Codex 等）用不了这家',
          evidence: detail,
        }
      }
      return { status: 'fail', error: '上游没有返回 tool_calls（已重试一次）', evidence: detail }
    },
  },

  tool_roundtrip: {
    label: '工具结果回灌',
    severity: 'required',
    dependsOn: 'tool_call',
    async run({ options, model, chat }) {
      const first = await chat(options, {
        model,
        stream: false,
        messages: [{ role: 'user', content: '请调用 get_weather 工具查询北京的天气。' }],
        tools: [WEATHER_TOOL],
        tool_choice: 'auto',
      })
      if (!first.ok) return { status: 'fail', error: first.error }
      const call = toolCallsOf(first.body).find(item => item?.function?.name === 'get_weather')
      if (!call) {
        return { status: 'skip', reason: '第一轮没有拿到 tool_calls，无法回灌' }
      }
      // up no to id when 1 ：assistant.tool_calls and tool must for ，
      // id will up （ `protocol::history` will ，
      // that ， to ）。
      const callId = call.id || 'call_probe_0'
      const second = await chat(options, {
        model,
        stream: false,
        tools: [WEATHER_TOOL],
        messages: [
          { role: 'user', content: '请调用 get_weather 工具查询北京的天气。' },
          { role: 'assistant', content: '', tool_calls: [{ ...call, id: callId }] },
          {
            role: 'tool',
            tool_call_id: callId,
            content: '{"city":"北京","weather":"晴","temperature":26}',
          },
          { role: 'user', content: '根据上面的工具结果，用一句话告诉我北京天气。' },
        ],
      })
      if (!second.ok) {
        return {
          status: 'fail',
          error: `回灌后请求失败：${second.error}`,
          evidence: { firstCallId: callId },
        }
      }
      const reply = contentOf(second.body)
      const again = toolCallsOf(second.body)
      if (!reply.trim()) {
        return {
          status: 'fail',
          error: '回灌后没有正文（可能又发起了一次工具调用，或上游丢弃了 tool 消息）',
          evidence: {
            toolCalls: again.length,
            finishReason: firstChoice(second.body)?.finish_reason,
          },
        }
      }
      return {
        status: 'pass',
        evidence: { reply: reply.slice(0, 160), repeatedCalls: again.length, firstCallId: callId },
      }
    },
  },

  parallel_tools: {
    label: '并行工具调用',
    severity: 'optional',
    async run({ options, model, catalog, chat }) {
      const prompt =
        '请**同时**调用 get_weather 和 get_local_time 两个工具，参数 city 分别填「北京」和「上海」。' +
        '不要用文字回答，只发起这两个工具调用。'
      let last = null
      for (let attempt = 0; attempt < 2; attempt += 1) {
        last = await chat(options, {
          model,
          stream: false,
          messages: [{ role: 'user', content: prompt }],
          tools: [WEATHER_TOOL, TIME_TOOL],
          tool_choice: 'auto',
        })
        if (!last.ok) return { status: 'fail', error: last.error }
        const calls = toolCallsOf(last.body)
        if (calls.length >= 2) {
          return {
            status: 'pass',
            evidence: { calls: calls.length, names: calls.map(call => call?.function?.name) },
          }
        }
      }
      const calls = toolCallsOf(last?.body ?? {})
      const detail = {
        calls: calls.length,
        names: calls.map(call => call?.function?.name),
        finishReason: firstChoice(last?.body)?.finish_reason,
      }
      if (catalog?.supports_tool_call === false && !options.force) {
        return {
          status: 'fail',
          error: '/v1/models 声明该模型不支持工具调用',
          evidence: detail,
        }
      }
      if (calls.length === 1) {
        // return 1 call is **** （ can can ）， this is 
        // 「 no has and can 」 ， not is —— optional， not 。
        return { status: 'info', reason: '只返回 1 个 tool_call（模型可能分两轮返回）', evidence: detail }
      }
      return { status: 'fail', error: '没有返回 tool_calls', evidence: detail }
    },
  },

  image: {
    label: '图片输入',
    severity: 'optional',
    async run({ options, model, catalog, chat }) {
      // directory****unsupported when ： this when request，none up is 400
      // is returnempty， is 「directory for 」， no has new 。
      // `--force` used for「directory」—— that down 1 。
      if (catalog?.supports_images === false && !options.force) {
        return { status: 'skip', reason: '/v1/models 声明不支持图片输入（--force 可强测）' }
      }
      const result = await chat(options, {
        model,
        stream: false,
        messages: [
          {
            role: 'user',
            content: [
              { type: 'text', text: '这张图片是什么颜色？只回答颜色名。' },
              { type: 'image_url', image_url: { url: RED_PNG_DATA_URL } },
            ],
          },
        ],
      })
      if (!result.ok) return { status: 'fail', error: result.error }
      const reply = contentOf(result.body)
      if (!reply.trim()) {
        return { status: 'fail', error: '图片请求没有返回正文' }
      }
      return {
        status: 'pass',
        evidence: {
          reply: reply.slice(0, 120),
          catalogDeclares: catalog?.supports_images ?? null,
        },
      }
    },
  },

  reasoning: {
    label: '思考 / 推理',
    severity: 'optional',
    async run({ options, model, catalog, chat }) {
      // and image 1 ：directoryunsupported when （`--force` ）。
      if (catalog?.supports_reasoning === false && !options.force) {
        return { status: 'skip', reason: '/v1/models 声明不支持思考（--force 可强测）' }
      }
      const result = await chat(options, {
        model,
        stream: false,
        reasoning_effort: 'high',
        messages: [
          { role: 'user', content: '一个笼子里有鸡和兔共 10 只，脚共 28 只。鸡和兔各几只？只给出答案。' },
        ],
      })
      if (!result.ok) return { status: 'fail', error: result.error }
      const reasoning = reasoningOf(result.body)
      if (reasoning.trim()) {
        return {
          status: 'pass',
          evidence: { chars: reasoning.length, preview: reasoning.slice(0, 120) },
        }
      }
      return {
        status: 'fail',
        error: '响应里没有 reasoning_content / reasoning 字段（模型可能只回正文）',
        evidence: { reply: contentOf(result.body).slice(0, 120) },
      }
    },
  },

  json_schema: {
    label: '结构化输出',
    severity: 'optional',
    async run({ options, model, chat }) {
      const result = await chat(options, {
        model,
        stream: false,
        messages: [
          {
            role: 'user',
            content: '用 JSON 描述北京的天气，包含 city（字符串）与 temperature（数字）两个字段。',
          },
        ],
        response_format: {
          type: 'json_schema',
          json_schema: {
            name: 'weather_report',
            schema: {
              type: 'object',
              properties: { city: { type: 'string' }, temperature: { type: 'number' } },
              required: ['city', 'temperature'],
              additionalProperties: false,
            },
          },
        },
      })
      if (!result.ok) return { status: 'fail', error: result.error }
      const reply = contentOf(result.body).trim()
      // has up will JSON in ```json （ need json_schema），
      // —— not 「unsupported」。
      const fenced = reply.match(/```(?:json)?\s*([\s\S]*?)```/)
      const candidate = (fenced ? fenced[1] : reply).trim()
      try {
        const parsed = JSON.parse(candidate)
        if (typeof parsed?.city !== 'string' || typeof parsed?.temperature !== 'number') {
          return { status: 'fail', error: 'JSON 能解析但字段不符合 schema', evidence: { parsed } }
        }
        return { status: 'pass', evidence: { parsed } }
      } catch {
        return {
          status: 'fail',
          error: '响应正文不是可解析的 JSON',
          evidence: { reply: reply.slice(0, 200) },
        }
      }
    },
  },

  usage: {
    label: 'usage 上报',
    severity: 'optional',
    async run({ options, model, chat }) {
      const result = await chat(options, {
        model,
        stream: true,
        stream_options: { include_usage: true },
        messages: [{ role: 'user', content: '说一句「你好」。' }],
      })
      if (!result.ok) return { status: 'fail', error: result.error }
      let usage = null
      for (const frame of result.frames) {
        if (frame.data?.usage && typeof frame.data.usage === 'object') {
          usage = frame.data.usage
        }
      }
      const total = Number(usage?.total_tokens ?? 0)
      if (!usage) {
        return {
          status: 'fail',
          error: '没有任何帧带 usage（请求已带 stream_options.include_usage）',
          evidence: { frames: result.frames.length },
        }
      }
      if (!(total > 0)) {
        // is ** already **（ModelScope usage is ，
        // in usage-only ； not include_usage as ）。
        return {
          status: 'fail',
          error: 'usage 全为零（上游可能只在特定帧给真值，或压根没统计）',
          evidence: { usage },
        }
      }
      return {
        status: 'pass',
        evidence: {
          promptTokens: usage.prompt_tokens ?? null,
          completionTokens: usage.completion_tokens ?? null,
          totalTokens: total,
        },
      }
    },
  },

  context_window: {
    label: '上下文窗口',
    severity: 'optional',
    async run({ catalog }) {
      // this 1 ** not up request**： is directory， and its use not ，
      // as info —— directory no not is 。
      if (catalog?.max_input_tokens == null) {
        return {
          status: 'info',
          reason: '/v1/models 没有声明 max_input_tokens（不影响转发，但客户端无法估算截断）',
        }
      }
      return {
        status: 'pass',
        evidence: {
          maxInputTokens: catalog.max_input_tokens,
          maxOutputTokens: catalog.max_output_tokens ?? null,
        },
      }
    },
  },
}

/** ：required in before ，`dependsOn` to before use 。 */
export const CASE_ORDER = [
  'stream',
  'tool_call',
  'tool_roundtrip',
  'parallel_tools',
  'image',
  'reasoning',
  'json_schema',
  'usage',
  'context_window',
]
