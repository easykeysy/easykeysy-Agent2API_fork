import { t } from '../i18n'
import { errorMessage, shared, toast, type AccountExportSelection } from './accounts-shared'

let exporting = false

export async function exportAccountFiles(selection: AccountExportSelection): Promise<void> {
  if (exporting) {
    toast(t('Accounts are being exported. Please wait.'))
    return
  }
  const bridge = shared().workbuddyDesktop
  if (!bridge?.exportAccountFiles) {
    toast(t('This client does not support individual account exports. Please update it.'), 'err')
    return
  }
  exporting = true
  try {
    const confirm = shared().wbConfirm?.ask
    const html = t('Exported files contain credentials that can be used to sign in. Keep them safe; do not share them or upload them to a public location.')
      + (selection.provider ? `<br><br>${t('Each account in this group will be saved as a separate JSON file, including accounts hidden by filters.')}` : '')
      + (selection.provider && bridge.platform === 'web'
        ? `<br><br>${t('The web panel downloads files individually. If prompted, allow this site to download multiple files.')}` : '')
    const accepted = confirm
      ? await confirm({
          title: selection.provider ? t('Export accounts in this group') : t('Export Accounts'),
          html,
          okText: t('Export'),
        })
      : window.confirm(t('Exported files contain credentials that can be used to sign in. Keep them safe; do not share them or upload them to a public location.'))
    if (!accepted) return
    const result = await bridge.exportAccountFiles(selection)
    if (result?.canceled) return
    if (!result) throw new Error(t('The export returned no result.'))
    const count = result.count || 0
    const failed = result.failed || []
    if (failed.length) {
      const first = failed[0]
      const detail = [first.name || first.id, first.error].filter(Boolean).join(': ')
      toast(t('Exported {count} account files; {failed} could not be exported: {reason}', {
        count, failed: failed.length, reason: detail,
      }), 'err')
    } else if (count) {
      toast(result.downloadStarted
        ? t('Started downloads for {count} account files. Check your browser\'s downloads to confirm.', { count })
        : t('✅ Exported {count} account files', { count }), 'ok')
    } else {
      toast(t('No accounts to export'))
    }
  } catch (error) {
    toast(t('Account export failed: {reason}', { reason: errorMessage(error) }), 'err')
  } finally {
    exporting = false
  }
}
