import * as React from 'react'
import {
  Badge,
  Dialog,
  DialogBody,
  DialogContent,
  DialogHeader,
  DialogTitle,
  Item,
  Button } from '@ui'
import { COMMUNITY_LINKS, QQ_GROUPS, handleExternalClick, shared } from './update-shared'
import { t } from '../i18n'

/**
 * 「」（update that by ）。
 *
 * ── ：， not is ──────────────────────────
 * 「 | | 」 1 ， not this 。
 * ： 1 （QQ / Telegram ，
 * icons.js that ）、 + ， can 。QQ is 「」
 * （ is need ）， 1 is 「」（ to ）。
 *
 * 1、 2 「 will 」 in up not is ： is this ****，
 * in up will ， when not this is 「」 。
 *
 * ── Why not ────────────────────────────────────
 * is ， / must 1 （ update-shared.ts
 * up QQ_GROUPS ）。 in will （ 1 、 after can can ），
 * + no has this ， 1 can 。
 */
export function QqGroupDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  /** 1 SVG（icons.js by index.html in before ，total is ） */
  function iconHtml(name: string, size: number): string {
    return shared().wbIcons?.icon?.(name, size) || ''
  }

  return (
    // open（ by ）： 1 by onClose ，
    // Esc / / up ✕ by Base UI Dialog （ and 「updateset」 1 ）
    <Dialog open={open} onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent className='w-[min(440px,calc(100vw-48px))]'>
        <DialogHeader>
          <DialogTitle>{t('加入交流群')}</DialogTitle>
        </DialogHeader>
        {/* in DialogBody up ： is ， onClick */}
        <DialogBody className='gap-2.5' onClick={handleExternalClick}>
          <p className='qq-intro'>{t('遇到问题或有使用心得，欢迎到群里交流。')}</p>

          <div className='qq-cards'>
            {/* 1、 2 3， in 。 */}
            {QQ_GROUPS.map((group, index) => (
              <Item render={<Button variant='outline' />} variant='outline'
                key={group.no}
                className='qq-card h-auto whitespace-normal text-left'
                data-copy={group.no}
                title={t('复制群号 {group}', { group: group.no })}
              >
                <span
                  className='qq-card-logo'
                  data-brand='qq'
                  dangerouslySetInnerHTML={{ __html: iconHtml('qq', 24) }}
                />
                <span className='qq-card-body'>
                  <span className='qq-card-title'>
                    {index === 0 ? t('QQ 群 1') : index === 1 ? t('QQ 群 2') : t('QQ 群 3')}
                    <code>{group.no}</code>
                  </span>
                  <span className='qq-card-sub'>
                    {group.nearlyFull ? t('即将满员，请优先加群 3') : t('点击复制群号')}
                  </span>
                </span>
                {group.nearlyFull ? (
                  <Badge shape='tag' variant='warning' className='qq-card-flag'>{t('即将满员')}</Badge>
                ) : null}
              </Item>
            ))}

            {/* Telegram： is 。（ not link up 
                 1 ）—— ， not ， will 1 */}
            {COMMUNITY_LINKS.map(link => (
              <Item render={<a href={link.url} target='_blank' rel='noopener' />} variant='outline'
                key={link.id}
                className='qq-card'
                data-external={link.url}
                title={t('在浏览器中打开')}
              >
                <span
                  className='qq-card-logo'
                  data-brand={link.id}
                  dangerouslySetInnerHTML={{ __html: iconHtml(link.id, 24) }}
                />
                <span className='qq-card-body'>
                  <span className='qq-card-title'>
                    {t('Telegram 群')}
                  </span>
                  <span className='qq-card-sub'>{t('点击在浏览器中打开')}</span>
                </span>
              </Item>
            ))}
          </div>

          <p className='qq-foot'>{t('在 QQ 里搜索群号即可加入。')}</p>
        </DialogBody>
      </DialogContent>
    </Dialog>
  )
}
