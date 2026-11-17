/**
 * 提供商能力探测的**用例定义**（与 CLI 外壳分离，见 `probe-provider.mjs`）。
 *
 * 每个用例是一个 `run({ options, model, catalog, chat })`，返回
 * `{ status, evidence?, error?, reason? }`：
 *   - `status`：`pass` / `fail` / `skip` / `info`
 *   - `evidence`：可复核的读数（渲染成一行摘要，`--json` 时原样输出）
 *   - `error` / `reason`：给人看的一句话
 *
 * 判定口径（为什么这些是必需项、那些只是可选项）见
 * `.agents/skills/provider-adaptation/SKILL.md` 第 6 节：agent 客户端对一家的
 * 最小可用集是「流式 + 工具调用 + 工具结果回灌」，其余是加分项。
 *
 * `dependsOn` 表示前置用例没通过就跳过（工具回灌必须先拿到 tool_calls）。
 */

/** 探测用的工具声明：参数极简，避免模型因为 schema 复杂而不调用。 */
const WEATHER_TOOL = {
  type: 'function',
  function: {
    name: 'get_weather',
    description: 'Query the current weather of the specified city',
    parameters: {
      type: 'object',
      properties: { city: { type: 'string', description: 'City name, e.g. Beijing' } },
      required: ['city'],
    },
  },
}

const TIME_TOOL = {
  type: 'function',
  function: {
    name: 'get_local_time',
    description: 'Query the current local time of the specified city',
    parameters: {
      type: 'object',
      properties: { city: { type: 'string', description: 'City name, e.g. Shanghai' } },
      required: ['city'],
    },
  },
}

/**
 * 64×64 纯红 PNG（178 字节），自己生成，不依赖外部图床。
 *
 * 用 data URL 而不是可访问的图片地址：探测不该要求这台机器能访问公网图床，
 * 也不该把请求发到第三方。极小的尺寸让不支持视觉的上游也只会报「看不懂」，
 * 而不会因为体积超限给出与能力无关的 413。
 */
const RED_PNG_DATA_URL =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAAAeUlEQVR4nO3PQQkAMAzAwCqpf1ETMxF7HINABFzm7H7dcEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFj13PLIEAOXyUUwAAAABJRU5ErkJggg=='

/* ─── 响应读取（各家字段形态的兜底） ───────── */

/** `choices[0]`；没有则 null。 */
function firstChoice(body) {
  return body?.choices?.[0] ?? null
}

/**
 * 取正文。
 *
 * 绝大多数上游给字符串；少数（Anthropic 风格的分段）给数组，这里拼成纯文本
 * —— 探测只关心「有没有内容」，不关心分段结构。
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
 * 取思考文本。
 *
 * 三种形态都要认：`reasoning_content`（多数 OpenAI 兼容家）、`reasoning`
 * （字符串）、以及对象形态 `{content:[{text}]}` / `{summary:[{text}]}`
 * （Responses 风格）。聚合器会把流式思考折进 `reasoning_content`，所以
 * 非流式请求也能读到。
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

/* ─── 用例 ───────────────────────────────── */

export const CASES = {
  stream: {
    label: 'Streaming SSE',
    severity: 'required',
    async run({ options, model, chat }) {
      const result = await chat(options, {
        model,
        stream: true,
        messages: [{ role: 'user', content: 'Count to three, output only the numbers, separated by commas.' }],
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
    label: 'Tool calling',
    severity: 'required',
    async run({ options, model, catalog, chat }) {
      // 第一次用常规提示；失败后换更硬的约束再试一次 —— 模型偶尔就是不调工具，
      // 单次失败不足以下结论（见 SKILL 第 6 节）。
      const prompts = [
        'Please use the get_weather tool to look up the weather in Beijing; do not answer in text.',
        'You must call the get_weather tool with the city parameter set to "Beijing". Answering directly is forbidden; only initiating the tool call is allowed.',
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
        // 目录**明确声明**不支持工具调用：这不是「测不出来」，而是「这家做不了
        // agent 客户端」。对必需项来说它仍是失败 —— 记为 skip 会让整体显示
        // 「必需项全部通过」，那是误导（见 SKILL 第 0 节的红线）。
        return {
          status: 'fail',
          error: '/v1/models Declares that the model does not support tool calling: agent clients (Claude Code / Codex, etc.) cannot use this provider",',
          evidence: detail,
        }
      }
      return { status: 'fail', error: 'Upstream returned no tool_calls (retried once)', evidence: detail }
    },
  },

  tool_roundtrip: {
    label: 'Tool result backfill',
    severity: 'required',
    dependsOn: 'tool_call',
    async run({ options, model, chat }) {
      const first = await chat(options, {
        model,
        stream: false,
        messages: [{ role: 'user', content: 'Please use the get_weather tool to look up the weather in Beijing.' }],
        tools: [WEATHER_TOOL],
        tool_choice: 'auto',
      })
      if (!first.ok) return { status: 'fail', error: first.error }
      const call = toolCallsOf(first.body).find(item => item?.function?.name === 'get_weather')
      if (!call) {
        return { status: 'skip', reason: 'The first round got no tool_calls; cannot backfill' }
      }
      // 上游没给 id 时自己造一个：assistant.tool_calls 与 tool 消息必须配对，
      // 缺 id 会被严格上游判成畸形历史（网关侧 `protocol::history` 会尝试修，
      // 但那属于兜底，探测应当给出规范形态）。
      const callId = call.id || 'call_probe_0'
      const second = await chat(options, {
        model,
        stream: false,
        tools: [WEATHER_TOOL],
        messages: [
          { role: 'user', content: 'Please use the get_weather tool to look up the weather in Beijing.' },
          { role: 'assistant', content: '', tool_calls: [{ ...call, id: callId }] },
          {
            role: 'tool',
            tool_call_id: callId,
            content: '{"city":"Beijing","weather":"Sunny","temperature":26}ather":"Sunny","temperature":26}',
          },
          { role: 'user', content: 'Based on the tool results above, tell me the weather in Beijing in one sentence.' },
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
          error: 'After the backfill there was no body (it may have issued another tool call, or the upstream dropped the tool message)',
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
    label: 'Parallel tool calls',
    severity: 'optional',
    async run({ options, model, catalog, chat }) {
      const prompt =
        'Please call both the get_weather and get_local_time tools at the same time, with city set to "Beijing" and "Shanghai" respectively.",' +
        'Do not answer in text; only issue these two tool calls.'
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
          error: '/v1/models Declare that this model does not support tool calling',
          evidence: detail,
        }
      }
      if (calls.length === 1) {
        // 单次返回一个 call 是**允许**的（模型可能打算分两轮），所以这只是
        // 「没有并行能力」的读数，不是适配缺陷 —— 可选项，不阻断验收。
        return { status: 'info', reason: 'Only 1 total tool_call was returned (the model may return it in two rounds)', evidence: detail }
      }
      return { status: 'fail', error: 'No tool_calls returned', evidence: detail }
    },
  },

  image: {
    label: 'Image input',
    severity: 'optional',
    async run({ options, model, catalog, chat }) {
      // 目录**明确声明**不支持时直接跳过：这时候再发图片请求，无论上游是 400
      // 还是静默返回空正文，结论都只是「目录说得对」，没有新信息。
      // `--force` 用于「怀疑目录写错了」—— 那种情况下才值得真发一次。
      if (catalog?.supports_images === false && !options.force) {
        return { status: 'skip', reason: '/v1/models Declares no image input support (--force can override the test)' }
      }
      const result = await chat(options, {
        model,
        stream: false,
        messages: [
          {
            role: 'user',
            content: [
              { type: 'text', text: 'What color is this image? Answer with the color name only.' },
              { type: 'image_url', image_url: { url: RED_PNG_DATA_URL } },
            ],
          },
        ],
      })
      if (!result.ok) return { status: 'fail', error: result.error }
      const reply = contentOf(result.body)
      if (!reply.trim()) {
        return { status: 'fail', error: 'The image request did not return a body' }
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
    label: 'Reasoning / 推理',
    severity: 'optional',
    async run({ options, model, catalog, chat }) {
      // 与 image 同一处置：目录声明不支持时直接跳过（`--force` 才真发）。
      if (catalog?.supports_reasoning === false && !options.force) {
        return { status: 'skip', reason: '/v1/models Declares no thinking support (--force can override the test)' }
      }
      const result = await chat(options, {
        model,
        stream: false,
        reasoning_effort: 'high',
        messages: [
          { role: 'user', content: 'A cage holds 10 chickens and rabbits together, 28 legs in total. How many of each are there? Give the answer only.",' },
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
        error: 'The response has no reasoning_content / reasoning field (the model may have returned only the body)",',
        evidence: { reply: contentOf(result.body).slice(0, 120) },
      }
    },
  },

  json_schema: {
    label: 'Structured output',
    severity: 'optional',
    async run({ options, model, chat }) {
      const result = await chat(options, {
        model,
        stream: false,
        messages: [
          {
            role: 'user',
            content: 'Describe the weather in Beijing as JSON, with two fields: city (string) and temperature (number).",',
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
      // 有些上游会把 JSON 包在 ```json 代码块里（即使要求了 json_schema），
      // 剥掉再解析 —— 包代码块本身不构成「不支持结构化输出」。
      const fenced = reply.match(/```(?:json)?\s*([\s\S]*?)```/)
      const candidate = (fenced ? fenced[1] : reply).trim()
      try {
        const parsed = JSON.parse(candidate)
        if (typeof parsed?.city !== 'string' || typeof parsed?.temperature !== 'number') {
          return { status: 'fail', error: 'JSON It parses, but the fields do not match the schema', evidence: { parsed } }
        }
        return { status: 'pass', evidence: { parsed } }
      } catch {
        return {
          status: 'fail',
          error: 'The response body is not parseable JSON',
          evidence: { reply: reply.slice(0, 200) },
        }
      }
    },
  },

  usage: {
    label: 'usage Report',
    severity: 'optional',
    async run({ options, model, chat }) {
      const result = await chat(options, {
        model,
        stream: true,
        stream_options: { include_usage: true },
        messages: [{ role: 'user', content: 'Say "hello".' }],
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
          error: 'None of the frames carried usage (the request already included stream_options.include_usage)",',
          evidence: { frames: result.frames.length },
        }
      }
      if (!(total > 0)) {
        // 全零是**真实的已知故障形态**（ModelScope 的每帧 usage 都是零，
        // 真值只在 usage-only 收尾块里；不注入 include_usage 就恒为零）。
        return {
          status: 'fail',
          error: 'usage All zero (the upstream may only give real values on certain frames, or may not have counted at all)',
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
    label: 'Context window',
    severity: 'optional',
    async run({ catalog }) {
      // 这一项**不发上游请求**：它读的是目录声明，与其余用例性质不同，
      // 因此缺值记为 info —— 目录没声明不是适配缺陷。
      if (catalog?.max_input_tokens == null) {
        return {
          status: 'info',
          reason: '/v1/models max_input_tokens is not declared (forwarding is unaffected, but clients cannot estimate truncation)",',
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

/** 执行顺序：必需项在前，`dependsOn` 指向前面的用例。 */
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
