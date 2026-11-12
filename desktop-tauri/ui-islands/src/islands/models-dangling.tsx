import * as React from 'react'
import { Badge, Button } from '@ui'
import { t } from '../i18n'
import { ALL_CUSTOM, type ManageMapping } from './models-custom-source'
import {
  bindingKeyOf, currentProvider, esc, getSnapshot, mappings, providerLabelOf, runRowAction, same,
  shared, subscribe, writeRemoveMapping,
} from './models-panel-state'

/**
 * 「 」。
 *
 * ── ────────────────────────────────────────────────
 * in 1 ， ** up ** in not in ： by 
 * `( × up id)` ， chip in that 1 up （
 * `models-panel-state::bindingsOf`）。 is up 1 down （ or id） after ，
 * to ** no has ** —— after `catalog::manage_view` 
 * return（`dangling: true`）， before path by ， this 
 * not not ， use can config.json。
 *
 * this is that ： `dangling` that （
 * and ），、 to delete。 after this for can new use （ 
 * new not will 「 already has enabled」）。
 *
 * ── Why `mappings()` not is ────────────────────────
 * `viewData().mappings` already ****（enabled / / empty in ），
 * `dangling` is after and 「 no has 」 —— before not 
 * 「 in not in 」（ that need need 1 ， this this after to ）。
 *
 * ── and 「 / 」 （ not and ）─────────────────────
 * is **** not is ：current and 「 has no has 」 is 
 * 。 not / ， need current down has empty。
 */
export function DanglingMappings() {
  const state = React.useSyncExternalStore(subscribe, getSnapshot)
  const provider = currentProvider()
  /** （「」/ 「」） empty */
  const aggregate = provider === 'all' || provider === ALL_CUSTOM
  const items = mappings()
    .filter(mapping => mapping.dangling === true)
    // 「 this 1 」 and 「 old （provider as empty， for all）」
    // —— and chip 1 （ models-panel-state::bindingsOf）
    .filter(mapping => aggregate || !mapping.provider || same(mapping.provider, provider))
    .sort((left, right) => (left.provider || '').localeCompare(right.provider || '')
      || left.alias.localeCompare(right.alias))
  if (!items.length) return null

  /**
   * delete 1 ：（ can has ）， has and toast。
   * delete after after will new manage （`writeRemoveMapping` → `accept`），
   * this —— to 1 not when 。
   */
  async function remove(item: ManageMapping): Promise<void> {
    const alias = String(item.alias)
    const target = String(item.target)
    const ok = await shared().wbConfirm?.ask?.({
      title: t('删除未挂载的映射'),
      html: t('确定删除映射「<strong>{alias} → {target}</strong>」？它指向的上游模型已不在清单里。', {
        alias: esc(alias), target: esc(target),
      }),
      okText: t('删除'),
      okClass: 'danger',
    })
    if (!ok) return
    void runRowAction(
      bindingKeyOf(alias, target, item.provider || ''),
      () => writeRemoveMapping(item.provider || '', alias, target),
      t('映射已删除'),
    )
  }

  return (
    <section className='models-dangling' aria-label={t('未挂载的模型映射')}>
      <div className='dg-head'>
        <span className='dg-title'>{t('未挂载的模型映射')}</span>
        <Badge variant='warning' shape='tag'>{items.length}</Badge>
      </div>
      <p className='dg-hint'>
        {t('这些映射指向的上游模型已不在当前清单里，挂不到任何模型行上：它们不会出现在 /v1/models，用对应别名发请求也会失败。删除后该对外名即可重新使用。')}
      </p>
      <ul className='dg-list'>
        {items.map(item => {
          const providerId = item.provider || ''
          const alias = String(item.alias)
          const target = String(item.target)
          // （alias == target）= default。 use for this ，config
          // will down this 1 ； after down ， not to 。 `x → x` will ，
          // 1 and 1 「default」。
          const isDefault = item.isDefault === true || same(alias, target)
          const busy = state.pending.has(bindingKeyOf(alias, target, providerId))
          return (
            <li className='dg-item' key={`${providerId}\u0001${alias}\u0001${target}`}>
              <span className='dg-prov'>
                {providerId ? providerLabelOf(providerId) : t('全部提供商')}
              </span>
              <span className='dg-map'>
                <code data-copy={alias} title={t('点击复制：{name}', { name: alias })}>{alias}</code>
                {isDefault ? null : (
                  <>
                    <span className='dg-arrow' aria-hidden='true'> → </span>
                    <code data-copy={target} title={t('点击复制：{name}', { name: target })}>{target}</code>
                  </>
                )}
              </span>
              {isDefault ? <span className='dg-tag'>{t('默认绑定')}</span> : null}
              {/* is 「 up down 」 is 「 current not in 」， to not ，
                   this in title （ after manage_view carried） */}
              {item.carried === false ? (
                <span className='dg-state' title={t('上游已经不再提供这个模型（可能已下线或改了 ID）')}>{t('目标已下线')}</span>
              ) : (
                <span className='dg-state' title={t('这个模型仍在承载它的提供商清单里，但那一家的模型行当前没有出现（例如该家没有可用账号）')}>{t('未接入')}</span>
              )}
              <Button variant='ghost' size='sm' className='text-destructive' disabled={busy}
                title={t('删除这条映射')}
                onClick={() => void remove(item)}>{t('删除')}</Button>
            </li>
          )
        })}
      </ul>
    </section>
  )
}
