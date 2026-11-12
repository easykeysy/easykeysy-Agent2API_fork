import * as React from 'react'
import { Button, Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, Progress } from '@ui'
import { dismissDbRestore, useSettings, type DbRestoreProgress } from './settings-state'
import { t } from '../i18n'
import { Alert } from '@ui'

/**
 * —— ** not can **， to 。
 *
 * ── Why not can ────────────────────────────────────────────
 * is ： 。 in （ or use as 「」） will 
 * down 1 not is old not is new —— to 1 、 to 1 。
 * in not to by ，Esc / / emptyignore；（success or failure）
 * after to 1 「」。
 *
 * ── Why need ──────────────────────────────────────────
 * need MB SQL、 in up 、、、 —— from 
 * seconds to minutes。 to 1 「 in …」 by ， use not need etc ，
 * not is not is 。
 *
 * after （`GET /api/backup/progress`， `db::backup` `RestoreProgress`），
 * before 500ms 1 。** not in before **： when no has can 
 * （`execute_batch` not ）， after to is 「」—— 1 not 
 * 1 。
 *
 * ── in this ────────────────────────────────────────
 * success to 「 N 」 and file；failure to Reason。 when 
 * （ use after need to ）。
 */
export function DbRestoreDialog(): React.ReactElement | null {
  const snap = useSettings()
  const progress: DbRestoreProgress | null = snap.dbRestore
  if (!progress) return null
  const { active, done, failed } = progress

  return (
    <Dialog
      open
      // in ignore 1 request（Esc / / empty）。
      // after ** not **： need ， by 「」。
      onOpenChange={() => { /* 1 ignore，file */ }}
    >
      <DialogContent showCloseButton={false} className='db-restore-dialog'>
        <DialogHeader>
          <DialogTitle>
            {done ? t('整库恢复完成') : failed ? t('整库恢复失败') : t('正在恢复整库')}
          </DialogTitle>
        </DialogHeader>

        <div className='db-restore-body'>
          {/* + already use when ： by after to （ is 「 this 1 in 」 1 ） */}
          <div className='db-restore-phase'>
            <span>{progress.phase || t('正在恢复…')}</span>
            <Elapsed startedAt={progress.startedAt} finishedAt={progress.finishedAt} active={active} />
          </div>

          <Progress
            value={progress.percent}
            aria-label={t('恢复进度')}
            className='db-restore-progress'
          />

          {active ? (
            <p className='db-restore-note'>
              {t('恢复期间请不要关闭应用或退出登录 —— 中途中断会让数据停在一个不完整的状态。完成后本页会自动刷新。')}
            </p>
          ) : null}

          {done ? (
            <div className='db-restore-result'>
              <p>
                <strong>{t('✅ 已恢复 {n} 张表', { n: progress.tables })}</strong>
              </p>
              {progress.safetyBackup ? (
                <p className='hint'>
                  {t('替换前的数据已自动备份为 {name}（在数据目录的 backups/ 下，可整文件拷回）', {
                    name: progress.safetyBackup,
                  })}
                </p>
              ) : null}
              {progress.localKeysPreserved ? (
                <p className='hint'>
                  {t('已使用本机密钥重新加密 {n} 项凭证，原有密钥未被覆盖。', { n: progress.reencryptedSecrets })}
                </p>
              ) : null}
              {progress.warning ? (
                <Alert variant='destructive' className='danger-zone'>
                  <strong>{t('⚠️ 密钥警告')}</strong>
                  <br />
                  {progress.warning}
                </Alert>
              ) : null}
            </div>
          ) : null}

          {failed ? (
            <div className='db-restore-result'>
              <Alert variant='destructive' className='danger-zone'>
                <strong>{t('恢复没有完成')}</strong>
                <br />
                {progress.error || t('原因未知')}
              </Alert>
              <p className='hint'>
                {t('密码、文件或密钥校验失败时不会替换主库；替换阶段失败会自动尝试还原，实际结果以上方错误说明为准。')}
              </p>
            </div>
          ) : null}
        </div>

        <DialogFooter>
          {/* in ： has 1 use by （ to 「 in not can 」 is ， not is ） */}
          {active ? (
            <Button variant='outline' disabled>
              {t('恢复中，请稍候…')}
            </Button>
          ) : (
            <Button variant='default' onClick={() => dismissDbRestore()}>
              {t('知道了')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/**
 * 「 already use when / total when 」。
 *
 * in seconds 1 （ has this 1 in ， not ）； after total when and 。
 */
function Elapsed({ startedAt, finishedAt, active }: {
  startedAt: number
  finishedAt: number
  active: boolean
}): React.ReactElement | null {
  const [now, setNow] = React.useState(() => Date.now())
  React.useEffect(() => {
    if (!active) return
    const timer = window.setInterval(() => setNow(Date.now()), 1000)
    return () => window.clearInterval(timer)
  }, [active])
  if (!startedAt) return null
  const end = active ? now : (finishedAt || now)
  const seconds = Math.max(0, Math.round((end - startedAt) / 1000))
  const text = seconds < 60
    ? t('{n} 秒', { n: seconds })
    : t('{m} 分 {s} 秒', { m: Math.floor(seconds / 60), s: seconds % 60 })
  return <span className='db-restore-elapsed'>{active ? t('已用时 {time}', { time: text }) : t('总耗时 {time}', { time: text })}</span>
}
