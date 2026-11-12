/**
 * Agent2API · 「 etc 」（R7）。
 *
 * from ui/models-reasoning.js ** has ** that 1 （ / /
 * etc ）： that file 1 is 「 select option、/
 * 」， in by React （ models-page.tsx MappingDialog），
 * not need need fillSelect / syncCustom / valueOf DOM 。
 *
 * not ：
 * · etc = ** **（`reasoningLevels`）→ after down use 
 * （= `model_rules::REASONING_LEVELS`）→ ， `levels()`；
 * · etc = 1 response `mappings` that `reasoning` ， by 
 * (alias, target, provider) —— chip up already this ， not 1 
 * （ 1 not for not up ， up will 「chip in 、 etc 」
 * this none from empty）。
 */

import { modelReasoning, type ReasoningModel } from '../../../ui-shared/models-reasoning'

export { bindingReasoningLevel, modelReasoning, POSITIVE_REASONING_LEVELS } from '../../../ui-shared/models-reasoning'

/**
 * 「 etc 」 in down ： not is 1 etc ， in is 。
 * use `__custom__` not is empty —— empty is 「 not 」 that 1 ，must。
 */
export const CUSTOM_LEVEL = '__custom__'

/**
 * not to 「 」 and after when （ and after that ****）。
 * path down use before ， after when before ； this in 
 * 「 + / failure」 when 。
 */
const FALLBACK_LEVELS: readonly string[] = ['off', 'none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max']

/**
 * 1 （ after / directory to is JSON）
 */
export type ReasoningMapping = {
  alias?: unknown
  target?: unknown
  provider?: unknown
  reasoning?: unknown
}

/**
 * can ** etc **：****， no has use use 。
 *
 * ── ────────────────────────────────────────────────
 * ① `model.reasoningLevels` —— this supports （ by after 1 ：
 * this > models.dev effort ， `core::model_meta`）。
 * e.g. GLM-5.3 has low / high / max ， to 1 not `minimal`
 * is unknownparameter to up ；
 * ② no to （ / this no ）→ after down use 
 * （`model_rules::REASONING_LEVELS`， OmniProxy use ）；
 * ③ after no to （ old / failure）→ （ and after ）。
 *
 * already as empty not ； 1 。
 */
export function levels(
  data: { reasoningLevels?: unknown } | null | undefined,
  model?: ReasoningModel | null,
): string[] {
  const info = modelReasoning(model)
  if (info.bindingSupported === false) return []
  if (info.declared) return info.bindingSupported === true
    ? info.levels.filter(level => !['off', 'none'].includes(level.toLowerCase())) : info.levels
  const list = data?.reasoningLevels
  if (Array.isArray(list) && list.length) {
    return list.filter((level): level is string => typeof level === 'string')
  }
  return [...FALLBACK_LEVELS]
}

/** use ； as empty when not to use 。 */
export function commonLevels(
  data: { reasoningLevels?: unknown } | null | undefined,
  models: ReadonlyArray<ReasoningModel | null | undefined>,
): string[] {
  if (!models.length) return levels(data)
  const [first, ...rest] = models.map(model => levels(data, model))
  return first.filter(candidate => rest.every(list => list.some(level => level.toLowerCase() === candidate.toLowerCase())))
}

/** → （； use not can can in \u0001 ） */
function keyOf(alias: unknown, target: unknown, provider: unknown): string {
  return `${String(alias ?? '').trim().toLowerCase()}\u0001${String(target ?? '').trim().toLowerCase()}`
    + `\u0001${String(provider ?? '').trim().toLowerCase()}`
}

/**
 * 「 → etc 」，return 1 「 1 」 （ not Map ： use not 
 * ， not will has in this Map ）。
 *
 * Why is not is find： in ** chip** up （ 1 can can up ），
 * up —— is O(chips × mappings) for ， 1 need 
 * 。 after is 1 。
 *
 * use must** before 1 **（，otherwise use etc 、，
 * chip up is old that ）。
 */
export function buildIndex(
  mappings: readonly ReasoningMapping[] | null | undefined,
): (alias: unknown, target: unknown, provider: unknown) => string {
  const index = new Map<string, string>()
  for (const mapping of Array.isArray(mappings) ? mappings : []) {
    const level = typeof mapping?.reasoning === 'string' ? mapping.reasoning.trim() : ''
    if (!level) continue
    index.set(keyOf(mapping.alias, mapping.target, mapping.provider), level)
  }
  return (alias, target, provider) => index.get(keyOf(alias, target, provider)) || ''
}
