import { t } from '../ui-kit/src/i18n'
import { apiExampleCode, type ApiExample } from './docs-api-examples'
import { imageApiSections } from './docs-api-images'
import { toolApiSections } from './docs-api-tools'

export type ApiField = { name: string; type: string; description: string }
export type ApiSection = {
  id: string
  title: string
  path?: string
  description: string
  paragraphs?: string[]
  fields?: ApiField[]
  points?: string[]
  warning?: string
  examples?: ApiExample[]
  /** 整份 Markdown 文档（工具服务 API 这类长文用它，不走五语词典） */
  markdown?: string
}

export function apiReferenceSections(origin: string): ApiSection[] {
  const code = apiExampleCode(origin)
  const example = (label: string, value: string, language: ApiExample['language'] = 'bash'): ApiExample => ({ label, code: value, language })
  return [
    {
      id: 'start', title: t('Connection and authentication'),
      description: t('Determine the gateway address, Key, and model before choosing a protocol supported by your client.'),
      paragraphs: [
        t('This page describes the public /v1 endpoints, not the management panel\'s /api endpoints. All three chat protocols share the model catalog, model mappings, account pool, and gateway Keys, but use different request and response formats.'),
        t('Set the OpenAI SDK\'s base_url to the Base URL ending in /v1 in the API address chapter, not the full /chat/completions URL. For the official Anthropic SDK and Claude Code, set ANTHROPIC_BASE_URL to the gateway root without /v1; they append /v1/messages themselves. Other clients may append /v1 differently; follow their address-field instructions.'),
        t('The address list and request examples use the current gateway address. On another device, 127.0.0.1 refers to that device itself; replace it with an address that actually reaches the gateway.'),
        t('Request examples use Bash / Git Bash syntax. Replace YOUR_GATEWAY_KEY, CHAT_MODEL_ID, IMAGE_MODEL_ID, and placeholder image URLs. Response examples only illustrate structure; capability flags, token counts, and timestamps are not measurements from your current gateway.'),
      ],
      fields: [
        { name: 'Authorization', type: 'HTTP header', description: t('Use Authorization: Bearer YOUR_GATEWAY_KEY. Obtain an enabled Key from the Gateway Key page.') },
        { name: 'x-api-key', type: 'HTTP header', description: t('Alternatively, use x-api-key: YOUR_GATEWAY_KEY, commonly used by Anthropic clients. Choose either authentication header; do not use provider account tokens or panel login tokens.') },
        { name: 'Content-Type', type: 'HTTP header', description: t('Use application/json for JSON requests and multipart/form-data when uploading files for image editing. curl -F generates the boundary automatically; do not override this header manually.') },
        { name: 'anthropic-version', type: 'HTTP header', description: t('The Anthropic SDK adds its version header automatically; handwritten Messages requests can use 2023-06-01. /v1/models returns the Anthropic list format when it receives that header or x-api-key.') },
      ],
      points: [
        t('Model calls always require an enabled, usable gateway API Key, even for local use. Administrator login sessions only authorize panel management and cannot replace the Key for model calls.'),
        t('GET /health, GET /v1/models, and temporary gateway image downloads do not require a Key. Reading the model list before configuring a Key does not validate it; only an actual protected request with that Key can do so.'),
        t('Per-Key model and provider restrictions also apply to image generation and editing. Use an id from /v1/models or a configured mapping name, not a display name.'),
      ],
      examples: [example(t('Environment variables for the SDK example'), code.setup), example(t('Read authorized models'), code.models)],
    },
    {
      id: 'models', title: t('Model discovery and health checks'), path: 'GET /v1/models · GET /health',
      description: t('Select the correct model using capability fields; health checks only confirm gateway reachability.'),
      paragraphs: [
        t('The model list reflects current accounts, enabled models, and mappings. Without a Key, the public list is available. If a Key is provided, it must be valid and the list is filtered by its authorization scope; invalid Keys return 401.'),
        t('Without Anthropic-related headers, the model endpoint returns the OpenAI-style object=list, data array, and meta summary. With anthropic-version or x-api-key, it returns data, has_more, first_id, and last_id while retaining capability extensions.'),
        t('GET /health returns status=ok or degraded. Public-network secure mode exposes only simplified liveness information. HTTP 200 or status=ok does not mean every provider is online, accounts have quota, or image models are callable.'),
      ],
      fields: [
        { name: 'data[].id', type: 'string', description: t('The model value for business requests. owned_by identifies the catalog entry\'s owner, not necessarily the provider serving each request; identical model names and mappings may have multiple routes.') },
        { name: 'kind', type: 'chat | image', description: t('chat models use the three chat endpoints; image models use /v1/images/generations and /v1/images/edits. Do not submit image models to chat endpoints.') },
        { name: 'supports_images', type: 'boolean', description: t('Indicates image input support, not image generation. Chat models can interpret images, and editing models may also declare this field.') },
        { name: 'supports_image_generation', type: 'boolean', description: t('Identifies an image-generation catalog entry. Calls still require generation support in the installed plugin and an available authorized account.') },
        { name: 'supports_image_editing', type: 'boolean', description: t('Indicates reference-image editing support declared by the current catalog and plugin. Check it before editing; an image substring in the model name is not sufficient.') },
        { name: 'supports_tool_call / supports_reasoning', type: 'boolean', description: t('Indicate tool-calling and reasoning support respectively. Actual behavior also depends on the serving model, provider, and settings.') },
        { name: 'input_modalities / output_modalities', type: 'string[]', description: t('Describe input and output modalities such as text, image, and video. Missing fields do not imply support for other files, audio, or server-side upload endpoints.') },
        { name: 'max_input_tokens / max_output_tokens', type: 'number', description: t('Catalog-declared context and maximum output budgets; may be omitted. Token budgets are not account balances and do not override actual upstream limits.') },
      ],
      examples: [example(t('Example model list structure'), code.modelResponse, 'json'), example(t('Check gateway reachability'), code.health)],
    },
    {
      id: 'chat', title: 'OpenAI Chat Completions', path: 'POST /v1/chat/completions',
      description: t('General-purpose chat endpoint for most OpenAI-compatible clients.'),
      paragraphs: [t('Read choices[].message for non-streaming responses and choices[].delta for streams. Supply messages history with every request; the gateway does not restore a conversation from a previous response id.')],
      fields: [
        { name: 'model', type: 'string', description: t('Explicitly select a chat model id from the current catalog. If omitted, the gateway tries its default model, still subject to the Key\'s authorized scope.') },
        { name: 'messages', type: 'array', description: t('Required. Each item includes role and content. Common roles are system, user, and assistant; tool results use tool. Text may be a string; multimodal content uses an array of content blocks.') },
        { name: 'stream', type: 'boolean', description: t('Defaults to false for a complete JSON response; true returns text/event-stream. Add -N to curl for streaming output. Parse complete SSE events rather than assuming network chunks are event boundaries.') },
        { name: 'stream_options.include_usage', type: 'boolean', description: t('Request include_usage=true for streaming usage. Whether final usage is returned depends on the upstream; do not expect usage in every delta.') },
        { name: 'max_tokens / max_completion_tokens', type: 'integer', description: t('Maximum output tokens; supported fields and budgets depend on the model. Avoid conflicting limits. finish_reason may be length when the limit is reached.') },
        { name: 'temperature / top_p', type: 'number', description: t('Sampling parameters. Support and valid ranges depend on the provider and model; protocol compatibility does not imply support for every tuning parameter.') },
        { name: 'tools / tool_choice', type: 'array / string | object', description: t('Use a tool-capable model. See the Vision and tool calling chapter for function declarations, matched tool results and multimodal request examples.') },
        { name: 'reasoning_effort / response_format', type: 'string / object', description: t('Request reasoning effort and structured output respectively. Capabilities and values depend on the model; accepting a field does not prove the upstream applied it.') },
      ],
      points: [
        t('Successful Chat streams normally end with data: [DONE]. Handle delta.content, reasoning fields, and delta.tool_calls; tool arguments may need to be assembled across multiple deltas.'),
        t('usage.prompt_tokens, completion_tokens, and total_tokens measure usage, not a uniform monetary price. Values may be omitted or 0 when the upstream does not provide them.'),
        t('HTTP status cannot change after streaming starts. Even after receiving 200, check stream errors, finish reasons, and normal termination; a disconnected stream is not a successful result.'),
      ],
      examples: [example(t('Non-streaming chat request'), code.chat), example(t('Streaming chat request'), code.chatStream), example(t('Non-streaming chat response'), code.chatResponse, 'json')],
    },
    {
      id: 'tools', title: t('Vision and tool calling'),
      description: t('Image input interprets images; tool calling asks the client to execute actions. Both require model support.'),
      paragraphs: [
        t('For vision, use a chat model with supports_images=true. Chat uses text and image_url blocks, with upstream-supported HTTP(S) URLs or data URLs. Formats and sizes depend on the target model. The upload limits in the Reference-image editing chapter do not automatically apply to all chat models.'),
        t('Declare custom function tools in tools. When the model returns tool_calls, the client validates arguments, executes functions, and sends both the assistant tool calls and role=tool results in the next messages array. Match tool_call_id and include results for every parallel call.'),
        t('Responses pairs function_call with function_call_output; Anthropic pairs tool_use with tool_result. Protocol conversion handles these formats but does not execute custom functions for the client. Native upstream tools such as search depend on the provider, not a universal gateway capability.'),
      ],
      warning: t('The built-in AI assistant has generate_images and edit_images tools, enabled in assistant settings and executed according to confirmation mode. External API clients do not receive these tools automatically; they must declare tools and call the image endpoints themselves.'),
      examples: [example(t('Vision request with a chat model'), code.vision), example(t('Request body declaring a function tool'), code.tools, 'json'), example(t('Request body returning executed tool results'), code.toolResult, 'json')],
    },
    {
      id: 'responses', title: 'OpenAI Responses', path: 'POST /v1/responses',
      description: t('Compatible Responses requests and events for clients such as Codex; not server-side conversation storage.'),
      fields: [
        { name: 'model', type: 'string', description: t('Use the same chat model ids and Key authorization rules as Chat.') },
        { name: 'input', type: 'string | array', description: t('Use a string for a single turn or an array of message and tool items for multiple turns, including required history. Text blocks use input_text; image blocks use input_image.') },
        { name: 'instructions', type: 'string', description: t('System instructions corresponding to Chat\'s system message; include them again on each request when needed.') },
        { name: 'max_output_tokens / stream', type: 'integer / boolean', description: t('Control the output limit and streaming. stream defaults to false; true returns named Responses events.') },
        { name: 'tools / tool_choice', type: 'array / string | object', description: t('Use Responses-format function declarations. Return tool results as function_call_output with the matching call_id. Actual support depends on the model and provider.') },
        { name: 'reasoning.effort / text.format', type: 'string / object', description: t('Map to reasoning effort and structured-output requests; supported values depend on the target model.') },
        { name: 'store', type: 'boolean', description: t('The gateway is always stateless and returns store=false. Sending store=true does not save Responses objects for later retrieval.') },
      ],
      points: [
        t('Server-state features previous_response_id, conversation, hosted prompt, and background=true are unsupported. Save and resend history on the client; do not call unimplemented Responses retrieval or deletion endpoints.'),
        t('Non-streaming responses contain object=response, status, output, and output_text. output may also contain reasoning or function_call items; do not read only the first item.'),
        t('Streams include response.created, response.output_text.delta, and final response.completed events. Errors may end with response.failed. Check incomplete status and incomplete_details when the output budget is insufficient.'),
      ],
      examples: [example(t('Responses request'), code.responses), example(t('Responses response structure'), code.responsesResponse, 'json')],
    },
    {
      id: 'messages', title: 'Anthropic Messages', path: 'POST /v1/messages · POST /v1/messages/count_tokens',
      description: t('Compatible Anthropic content blocks and SSE events, with local token budget estimates.'),
      fields: [
        { name: 'model', type: 'string', description: t('Select a chat model from the current gateway catalog; its name need not start with claude. Capabilities depend on the model actually serving the request.') },
        { name: 'messages', type: 'array', description: t('Required. Use user / assistant roles. content may be a string or blocks such as text, image, tool_use, and tool_result.') },
        { name: 'system', type: 'string | array', description: t('System instructions belong in this top-level field, not a system role in messages. Accepts text or an array of text blocks.') },
        { name: 'max_tokens', type: 'integer', description: t('The Anthropic standard requires an explicit positive integer. The gateway substitutes 8192 for missing or invalid values, but the upstream model may have a lower budget.') },
        { name: 'stream / stop_sequences', type: 'boolean / string[]', description: t('stream defaults to false; true returns Anthropic events. stop_sequences maps to stop sequences, with support determined by the upstream.') },
        { name: 'tools / tool_choice', type: 'array / object', description: t('Function tools declare name, description, and input_schema. Return results in tool_result blocks of a subsequent user message with the matching tool_use_id.') },
        { name: 'thinking / output_config', type: 'object', description: t('Request reasoning configuration or effort, mapped to the target model. Not all native Anthropic features are guaranteed equivalent across protocol conversion.') },
      ],
      points: [
        t('Non-streaming responses contain type=message, content blocks, stop_reason, and usage. Iterate by type because content may combine text, reasoning, and tool calls.'),
        t('Streams begin with message_start, include content_block_start / content_block_delta / content_block_stop, and finish with message_delta and message_stop. Do not parse them using Chat\'s [DONE] convention.'),
        t('count_tokens estimates locally without calling an upstream tokenizer. Returns input_tokens and estimated=true. Use it for context budgeting, not exact billing, especially not the actual token cost of images.'),
      ],
      examples: [example(t('Messages request'), code.messages), example(t('Messages response structure'), code.messagesResponse, 'json'),
        example(t('Estimate input tokens'), code.countTokens), example(t('Token estimate response'), code.countResponse, 'json')],
    },
    ...imageApiSections(code),
    ...toolApiSections(origin),
    {
      id: 'errors', title: t('Errors, limits, and compatibility boundaries'),
      description: t('Use HTTP status, error details, and request logs to diagnose issues; do not blindly retry billable requests.'),
      paragraphs: [
        t('Chat, Responses, and image endpoints normally return an error object: message describes the error, type classifies it, and code and upstream_code are optional. Anthropic business errors use a type=error envelope; authentication middleware, invalid JSON, and gateway-level errors may also use the generic error shape.'),
        t('This page documents implemented endpoints, not a complete OpenAI / Anthropic service replica. Do not assume unlisted Files, Batch, image variations, or server-side conversation storage endpoints exist.'),
      ],
      fields: [
        { name: '400', type: 'HTTP status', description: t('Unsupported parameters, model capabilities, or protocol formats. Image codes may include invalid_image_input, image_generation_not_supported, or image_editing_not_supported. Correct the request before resubmitting.') },
        { name: '401', type: 'HTTP status', description: t('Missing, invalid, or disabled Key, or upstream authentication failure. The gateway returns model_api_key_required for a missing Key and invalid_api_key for an invalid one. Check request headers and the Gateway Key page.') },
        { name: '404', type: 'HTTP status', description: t('Model missing, disabled, or outside the Key\'s scope, commonly with model_not_found. It may also indicate a wrong path or an expired temporary image URL.') },
        { name: '408 / 413', type: 'HTTP status', description: t('May indicate interruption and oversized requests respectively. Reduce reference-image size or count; resending unchanged data will not help. Interrupted image jobs may already have consumed quota.') },
        { name: '429', type: 'HTTP status', description: t('Rate limit, insufficient quota, or no currently available account window. When recovery is known, Retry-After (seconds), reset_at (Unix milliseconds), and reset_at_text may be returned; they are not always present.') },
        { name: '500 / 502 / 503', type: 'HTTP status', description: t('Possible internal gateway errors, upstream failures, missing plugin capabilities, unavailable accounts, or image download and encoding failures.') },
      ],
      warning: t('If the error warns that an image request may have consumed quota and was not retried, or that the upstream already generated it, check request logs and upstream results first. Client timeout, disconnection, or cancellation does not imply upstream cancellation; resubmission may charge again.'),
      points: [
        t('For all three chat protocols, check stream events and normal termination markers. HTTP 200 alone does not prove success. Image endpoints return non-streaming JSON only.'),
        t('Protocol compatibility does not guarantee every provider supports tools, reasoning, structured output, transparent backgrounds, or every size. Check model capabilities and plugin versions, then inspect the serving provider and error summary in request logs.'),
        t('Gateway safe retries and client SDK retries are independent. Disable automatic retries for image clients; the SDK may resubmit the entire request even after the gateway stopped retrying.'),
      ],
      examples: [example(t('Generic error structure example'), code.error, 'json'), example(t('Anthropic business error structure example'), code.anthropicError, 'json')],
    },
  ]
}
