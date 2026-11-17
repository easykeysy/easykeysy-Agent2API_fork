import { t } from '../ui-kit/src/i18n'
import { apiExampleCode, type ApiExample } from './docs-api-examples'
import type { ApiSection } from './docs-api-content'

export function imageApiSections(code: ReturnType<typeof apiExampleCode>): ApiSection[] {
  const example = (label: string, value: string, language: ApiExample['language'] = 'bash'): ApiExample => ({ label, code: value, language })
  return [
    {
      id: 'image-generation', title: t('Image generation and shared parameters'), path: 'POST /v1/images/generations',
      description: t('Generate images from a text prompt. Requests use JSON; both generation and editing support non-streaming responses only.'),
      paragraphs: [
        t('Raccoon, Loomy, Antigravity, ModelScope and WorkBuddy currently support image generation; Loomy, Antigravity, ModelScope and WorkBuddy also support reference-image editing, while Raccoon does not. The provider plugin must include the capability, and the account, model and gateway key authorization must be available.'),
        t('Raccoon uses raccoon-image; Loomy uses the kind=image models in the catalog; Antigravity uses the gemini-*-image models in the catalog; ModelScope uses the confirmed Qwen-Image, Z-Image or Qwen-Image-Edit models; WorkBuddy uses the models in the catalog tagged text-to-image or image-to-image, such as hunyuan-image-alpha and hunyuan-image-alpha-edit. Always take the actual model value from the current /v1/models; do not copy model names from a vendor\'s own API defaults.'),
        t('This table shows parameters accepted by the gateway, not support shared by every provider. Generation and editing share these fields, while upstreams may impose stricter count, quality, style, background, or size limits.'),
      ],
      fields: [
        { name: 'prompt', type: 'string', description: t('Required and nonempty, up to 32000 characters. Describe the subject, composition, style, and desired changes. Masks are unsupported, so describe editing regions in the prompt as well.') },
        { name: 'model', type: 'string', description: t('Specify an image model id explicitly. If omitted, the gateway selects a model supporting this operation from the Key\'s available catalog, not the default chat model. Returns 503 if no capable model is available.') },
        { name: 'n', type: 'integer', description: t('The gateway accepts 1–10, default 1; Antigravity and ModelScope accept only n=1. If the upstream returns too few images, the gateway reports an error without generating again to fill the count.') },
        { name: 'size', type: 'string', description: t('Optional; use auto or WIDTHxHEIGHT, such as 1024x1024. The gateway validates edges from 1–8192; providers may further restrict aspect ratios and resolution. Exact output pixels are not guaranteed.') },
        { name: 'response_format', type: 'b64_json | url', description: t('Defaults to b64_json for decodable image base64. url returns an upstream signed URL or temporary gateway download URL. This is not the image file encoding.') },
        { name: 'output_format', type: 'png | jpeg | webp', description: t('The gateway can convert the actual image to the selected encoding. If omitted, it preserves the upstream format. Specify it for a fixed file extension; JPEG cannot preserve transparency.') },
        { name: 'output_compression', type: 'integer · 0–100', description: t('Supported only with explicit output_format=jpeg; the gateway encodes using a JPEG quality setting. PNG and WebP do not currently support this parameter.') },
        { name: 'quality', type: 'string', description: t('Accepted values: auto, standard, hd, low, medium, high, xhigh, max. Raccoon and Antigravity allow only omission, auto, or standard. Other values depend on the particular Loomy model.') },
        { name: 'style', type: 'natural | vivid', description: t('Raccoon and Antigravity do not support this field; describe style in prompt. Loomy forwards it upstream, where support depends on the model.') },
        { name: 'background', type: 'auto | opaque | transparent', description: t('Raccoon and Antigravity allow only omission, auto, or opaque. Transparent output is not supported by every model; file conversion does not remove backgrounds.') },
        { name: 'moderation', type: 'auto | low', description: t('Raccoon and Antigravity allow only omission or auto. This does not guarantee reduced upstream moderation, and content-policy rejections are not retried automatically.') },
        { name: 'user', type: 'string', description: t('Optional caller user identifier, not a gateway Key. It cannot select an account or bypass Key restrictions. Forwarding and handling depend on the provider.') },
        { name: 'stream / partial_images', type: 'false / 0', description: t('Omit or set stream=false and partial_images=0. Image event streams, progressive images, and partial streaming results are unsupported. Unknown parameters are rejected, not silently ignored.') },
      ],
      points: [
        t('ModelScope uses the official asynchronous task API. It polls the same task without resubmitting. Each request uses n=1; size may be omitted, auto, or WIDTHxHEIGHT with edges from 64–2048. Individual models may impose stricter limits.'),
        t('ModelScope does not support style, mask or input_fidelity. quality accepts omission, auto or standard; background accepts omission, auto or opaque; moderation accepts omission or auto. Describe style and editing goals in prompt.'),
        t('Antigravity supports aspect ratios 1:1, 2:3, 3:2, 3:4, 4:3, 4:5, 5:4, 9:16, 16:9, and 21:9, mapping sizes to native 1K / 2K / 4K tiers. Requested longest edges ≤1792 map to 1K, 1793–3072 to 2K, and 3073–4096 to 4K; this provider rejects values above 4096.'),
        t('For Antigravity, size=auto generates at 1:1 and 1K; reference editing uses 1K without forcing a square. Model versions may have stricter output limits. Aspect ratios must be close to supported values; arbitrary pixel sizes are not supported.'),
        t('WorkBuddy uses its official image API: an omitted size, or size=auto, is requested as 1024x1024; this channel does not accept quality, style, background or moderation, so describe style and quality in the prompt. Reference images are sent inline as data URLs inside the JSON request (not multipart), and model ids come from the upstream catalog — check /v1/models when the version changes.'),
      ],
      warning: t('Image generation may consume account quota. Start with n=1 and a smaller size. Do not automatically resubmit after content-policy rejection or an uncertain result.'),
      examples: [example(t('JSON image generation request'), code.generation)],
    },
    {
      id: 'image-editing', title: t('Reference-image editing'), path: 'POST /v1/images/edits',
      description: t('Upload one or more reference images and describe what to preserve and change in the prompt.'),
      paragraphs: [
        t('Use an image model with supports_image_editing=true. This endpoint edits, redraws, or combines references; it is not precise mask-based inpainting. mask, input_fidelity, and file_id are currently unsupported.'),
        t('Use multipart/form-data for local files, or application/json for existing image URLs or base64 data. Other than reference fields, parameters are shared with generation.'),
      ],
      fields: [
        { name: 'image / image[]', type: 'multipart file', description: t('Use image=@reference.png for one image, or repeat image[] for multiple images. images, images[], and numeric fields such as image[0] are also accepted. Regular parameters cannot be repeated.') },
        { name: 'images[].image_url', type: 'JSON string', description: t('JSON images is an array of objects containing only image_url. Accepts absolute HTTP(S) URLs, data:image/...;base64,..., or full temporary gateway image URLs returned earlier. Raw base64, relative paths, file_id, and local file paths are not accepted.') },
        { name: 'PNG / JPEG / WebP', type: 'image', description: t('Only these three decodable formats are supported. Actual file contents are validated, not just extensions. Each image is limited to 10 MiB and edges of 8192 pixels.') },
        { name: 'images', type: 'array · 1–16', description: t('The gateway accepts up to 16 references. Antigravity gemini-2.5-* models accept up to 3; other integrated Antigravity image models accept up to 14. ModelScope accepts 1–3; Loomy may impose additional model limits.') },
        { name: 'multipart / JSON', type: '16 MiB / 14 MiB', description: t('Editing uploads are limited to 16 MiB total. After conversion to data URLs, inline images and the final JSON request are also limited to 14 MiB. Base64 adds about one third to size, so several files below 10 MiB each can still exceed the total.') },
      ],
      points: [
        t('The gateway server downloads HTTP(S) references without gateway Keys or provider credentials. External URLs must be accessible without extra headers. Local, private-network, credential-bearing, or unsafe redirect addresses are rejected. Upload protected images directly instead.'),
        t('A full /v1/images/files/{id} URL previously returned by this gateway can serve as a JSON reference while valid. Do not use the assistant\'s internal assistant-image:ID references as public API image URLs.'),
        t('Image order matters. Explain relationships in prompt, for example, \'image 1 is the subject; image 2 is the style reference\'. Editing does not guarantee pixel-perfect preservation; explicitly identify subjects, text, or layouts to retain.'),
        t('curl -F generates multipart boundaries. Do not add Content-Type: application/json or manually set multipart/form-data without a boundary. Use --form-string for prompt to avoid interpreting special characters as file uploads.'),
      ],
      examples: [example(t('Upload one local reference image'), code.editMultipart), example(t('Upload multiple reference images'), code.editMultiple), example(t('JSON image-reference editing request'), code.editJson)],
    },
    {
      id: 'image-results', title: t('Image responses and downloads'), path: 'GET /v1/images/files/{id}',
      description: t('Generation and editing return the same structure. Base64 suits direct saving; URLs suit short-term retrieval.'),
      fields: [
        { name: 'created', type: 'integer', description: t('Response creation time in Unix seconds. Do not confuse it with error recovery field reset_at, which uses milliseconds.') },
        { name: 'data[].b64_json', type: 'string', description: t('With response_format=b64_json, returns standard base64 without a data URL prefix. Decode to binary and save. For web display, add the appropriate MIME prefix yourself.') },
        { name: 'data[].url', type: 'string', description: t('Returned with response_format=url. May be a provider signed URL or temporary gateway download URL. Use the returned value directly; do not construct download ids yourself.') },
        { name: 'data[].revised_prompt', type: 'string', description: t('Preserved only when the upstream returns a revised prompt; not every model provides one.') },
        { name: 'output_format / usage / points_consumed', type: 'optional', description: t('May include actual encoding, usage, or points. usage and points_consumed depend on the upstream; their absence does not imply free service. Without a requested encoding, do not assume all images are PNG.') },
      ],
      points: [
        t('Temporary gateway images are kept in memory for 1 hour and disappear immediately on gateway restart. Cache capacity is 128 MiB; a full cache returns an error suggesting b64_json. Upstream signed URL lifetimes are determined by the provider.'),
        t('Temporary gateway downloads do not require Authorization. The random URL token is the access credential; anyone with the link can download. Treat it as sensitive, save important results promptly, and do not use it as long-term storage.'),
        t('Downloads return raw image bytes with image/png, image/jpeg, or image/webp Content-Type, not JSON. Missing or expired links return 404; responses use private, no-store and nosniff.'),
        t('When the gateway downloads or converts output, each image is limited to 32 MiB and edges of 8192 pixels with bounded decoding. output_format converts the actual image, not merely its extension.'),
      ],
      warning: t('response_format=url does not guarantee permanent URLs. Temporary gateway links and assistant-session image storage are separate mechanisms; their endpoints and lifetimes are not interchangeable.'),
      examples: [example(t('Base64 image response structure'), code.imageBase64, 'json'), example(t('Temporary gateway URL response structure'), code.imageUrl, 'json'), example(t('Download and save a temporary gateway image'), code.download)],
    },
    {
      id: 'image-sdk', title: t('OpenAI Python SDK image example'),
      description: t('Generate and edit with one client, then save returned base64 as image files.'),
      paragraphs: [
        t('Install an OpenAI Python SDK version supporting Images parameters. Set GATEWAY_KEY and IMAGE_MODEL_ID as described in the connection section. Check editing support and prepare a local reference.png before editing.'),
        t('The example sets max_retries=0 to prevent automatic SDK resubmission of billable requests on timeouts or 5xx. timeout=300.0 is only an example client wait; it does not change gateway or upstream timeouts.'),
      ],
      warning: t('Do not put image base64 into chat text or logs, and do not interpret a request timeout as \'not charged\'. For successive edits, save the result and upload it as the next image.'),
      examples: [example(t('Python: generate, edit, and save images'), code.imageSdk, 'python')],
    },
  ]
}
