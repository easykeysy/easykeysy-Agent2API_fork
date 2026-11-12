/**
 * ：file → data URL， and by need 。
 *
 * ── Why is data URL not is blob URL ───────────────────────
 * will need （`/api/assistant/sessions` is JSON），blob URL in 
 * valid，；base64 this JSON ， is 1 
 * 「 in 」 ， up `image_url` data URI（ 
 * by this ）。
 *
 * ── Why need ──────────────────────────────────────────
 * 2560 ， base64 will will JSON to MB（ after will 
 * limit 8 MB）， as 「 for in up ， no 」。 up need need not to 
 * that ， by to [`MAX_IMAGE_EDGE`]、 by need WebP。
 */

import { t } from '../i18n'

/** limit（）。 etc */
export const MAX_IMAGE_EDGE = 1600
/** data URL limit（≈ 0.9 MB ）： 1 */
export const MAX_IMAGE_CHARS = 1_200_000

export function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result || ''))
    reader.onerror = () => reject(reader.error || new Error(t('读取图片失败')))
    reader.readAsDataURL(file)
  })
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image()
    image.onload = () => resolve(image)
    image.onerror = () => reject(new Error(t('读取图片失败')))
    image.src = src
  })
}

/**
 * and ** by need **。
 *
 * GIF not （canvas will down 1 ，）； its 1 by 
 * to [`MAX_IMAGE_EDGE`]， [`MAX_IMAGE_CHARS`] when WebP ——
 * that 1 ， not use 。
 */
export async function prepareImage(file: File): Promise<string> {
  const raw = await readAsDataUrl(file)
  if (file.type === 'image/gif') return raw
  const image = await loadImage(raw)
  const longest = Math.max(image.naturalWidth, image.naturalHeight)
  const scale = longest > 0 ? Math.min(1, MAX_IMAGE_EDGE / longest) : 1
  if (scale === 1 && raw.length <= MAX_IMAGE_CHARS) return raw
  const canvas = document.createElement('canvas')
  canvas.width = Math.max(1, Math.round(image.naturalWidth * scale))
  canvas.height = Math.max(1, Math.round(image.naturalHeight * scale))
  const context = canvas.getContext('2d')
  if (!context) return raw
  context.drawImage(image, 0, 0, canvas.width, canvas.height)
  const encoded = canvas.toDataURL('image/webp', 0.86)
  return encoded.length < raw.length ? encoded : raw
}
