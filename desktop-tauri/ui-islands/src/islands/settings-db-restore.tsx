import * as React from 'react'
import { Button, Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, Progress } from '@ui'
import { dismissDbRestore, useSettings, type DbRestoreProgress } from './settings-state'
import { t } from '../i18n'
import { Alert } from '@ui'

/**
 * 整库恢复的进度弹窗 —— **不可关闭**，直到恢复收口。
 *
 * ── 为什么不能关 ────────────────────────────────────────────
 * 恢复是替换语义：主库被整体覆盖。中途关窗（或让用户以为「关掉就停了」）会
 * 留下一个既不是旧库也不是新库的现场 —— 暂存库执行到一半、主库覆盖到一半。
 * 所以进行中不给关闭按钮，Esc / 点遮罩 / 点空白全部忽略；收口（成功或失败）
 * 之后才给一个「知道了」。
 *
 * ── 为什么要进度条 ──────────────────────────────────────────
 * 恢复要读几百 MB 的 SQL、在暂存库上执行、校验、拍安全备份、覆盖主库 —— 从
 * 几十秒到几分钟。只给一个「恢复中…」的灰按钮，用户既不知道还要等多久，
 * 也不知道程序是不是卡死了。
 *
 * 进度来自后端（`GET /api/backup/progress`，见 `db::backup` 的 `RestoreProgress`），
 * 前端每 500ms 拉一次。**不在前端自己编百分比**：各阶段耗时没有可靠计量
 * （`execute_batch` 不回调），后端给的是「阶段起点」—— 一个只增不减的粗进度
 * 比一个乱跳的假进度诚实。
 *
 * ── 结果也在这里显示 ────────────────────────────────────────
 * 成功给「恢复了 N 张表」与安全备份文件名；失败给原因。密钥指纹警告同时写进
 * 面板里的常驻警告框（用户关掉弹窗后仍要看得到）。
 */
export function DbRestoreDialog(): React.ReactElement | null {
  const snap = useSettings()
  const progress: DbRestoreProgress | null = snap.dbRestore
  if (!progress) return null
  const { active, done, failed } = progress

  return (
    <Dialog
      open
      // 进行中忽略一切关闭请求（Esc / 点遮罩 / 点空白）。
      // 收口后也**不**自动关：结果要让人看完再走，只由「知道了」关。
      onOpenChange={() => { /* 一律忽略，见文件头 */ }}
    >
      <DialogContent showCloseButton={false} className='db-restore-dialog'>
        <DialogHeader>
          <DialogTitle>
            {done ? t('The full-database restore is complete') : failed ? t('The full-database restore failed') : t('Restoring the full database')}
          </DialogTitle>
        </DialogHeader>

        <div className='db-restore-body'>
          {/* 阶段文案 + 已用时：文案由后端给（它是「这一步在做什么」的唯一权威） */}
          <div className='db-restore-phase'>
            <span>{progress.phase || t('Restoring…')}</span>
            <Elapsed startedAt={progress.startedAt} finishedAt={progress.finishedAt} active={active} />
          </div>

          <Progress
            value={progress.percent}
            aria-label={t('Restore progress')}
            className='db-restore-progress'
          />

          {active ? (
            <p className='db-restore-note'>
              {t('Please do not close the app or log out during the restore—— An interruption midway leaves the data in an incomplete state. This page will refresh automatically when done.')}
            </p>
          ) : null}

          {done ? (
            <div className='db-restore-result'>
              <p>
                <strong>{t('✅ Restored {n} tables', { n: progress.tables })}</strong>
              </p>
              {progress.safetyBackup ? (
                <p className='hint'>
                  {t('The pre-replacement data was automatically backed up as {name} (in the data directory\'s backups/; you can copy the whole file back)",', {
                    name: progress.safetyBackup,
                  })}
                </p>
              ) : null}
              {progress.localKeysPreserved ? (
                <p className='hint'>
                  {t('Re-encrypted {n} credentials with local keys. Existing keys were not overwritten.', { n: progress.reencryptedSecrets })}
                </p>
              ) : null}
              {progress.warning ? (
                <Alert variant='destructive' className='danger-zone'>
                  <strong>{t('⚠️ Key warning')}</strong>
                  <br />
                  {progress.warning}
                </Alert>
              ) : null}
            </div>
          ) : null}

          {failed ? (
            <div className='db-restore-result'>
              <Alert variant='destructive' className='danger-zone'>
                <strong>{t('The restore did not complete')}</strong>
                <br />
                {progress.error || t('原因未知')}
              </Alert>
              <p className='hint'>
                {t('Password, file, or key validation failures leave the main database unchanged. If replacement fails, restoration is attempted automatically; the error above reports the actual outcome.')}
              </p>
            </div>
          ) : null}
        </div>

        <DialogFooter>
          {/* 进行中：只有一个禁用的说明按钮（让人看到「现在不能关」是设计，不是卡住） */}
          {active ? (
            <Button variant='outline' disabled>
              {t('Restoring; please wait…')}
            </Button>
          ) : (
            <Button variant='default' onClick={() => dismissDbRestore()}>
              {t('Got it')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

/**
 * 「已用时 / 总耗时」。
 *
 * 进行中每秒走一次（只有这一小块在跳，不带着整页重渲）；收口后显示总耗时并停表。
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
    ? t('{n} s', { n: seconds })
    : t('{m} {s} seconds', { m: Math.floor(seconds / 60), s: seconds % 60 })
  return <span className='db-restore-elapsed'>{active ? t('Elapsed {time}', { time: text }) : t('Total duration {time}', { time: text })}</span>
}
