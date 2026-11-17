import { t } from '../i18n'

export { modelOptionsFrom, selectedModel, type ModelOption } from '../../../ui-shared/assistant-models'

export function unavailableModelMessage(name: string): string {
  return t('The model "{model}" is not in the available list. Pick another one; you can also check whether it is enabled under Model management or on the Plugins page.', { model: name })
}
