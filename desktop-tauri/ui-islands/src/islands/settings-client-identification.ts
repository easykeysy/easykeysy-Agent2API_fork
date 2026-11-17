import { t } from '../i18n'
import type { ClientIdentificationApi, ClientIdentificationView, IdentificationPreview } from '../../../ui-shared/client-identification'

type Bridge = {
  getClientIdentification(): Promise<ClientIdentificationView>
  saveClientIdentification: ClientIdentificationApi['save']
  previewClientIdentification: ClientIdentificationApi['preview']
}
function bridge(): Bridge {
  const value = (window as unknown as { workbuddyDesktop?: Bridge }).workbuddyDesktop
  if (!value?.getClientIdentification || !value.saveClientIdentification || !value.previewClientIdentification) {
    throw new Error(t('Client identification is not ready. Refresh the page and try again.'))
  }
  return value
}
export const clientIdentificationApi: ClientIdentificationApi = {
  load: () => bridge().getClientIdentification(),
  save: payload => bridge().saveClientIdentification(payload),
  preview: (payload): Promise<IdentificationPreview> => bridge().previewClientIdentification(payload),
}
