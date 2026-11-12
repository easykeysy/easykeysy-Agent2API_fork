/**
 * Agent2API · 「」 1 ：（ + + ）。
 *
 * old add-provider-forms.js providerCards / cardHtml / logoHtml / cardMeta /
 * renderProviderCards / newCardHtml and mountAddProviderUi that 。
 *
 * and （.add-provider-grid / .add-provider-card / .add-provider-logo…）
 * ： that is （ + + ），
 * no has for ， Tailwind will 1 already 1 。
 * （、）。
 *
 * ── 「」 ──────────────────────────────────────────────
 * before 1 「」（ is default）： by 
 * in 1 ， 1 。 —— 
 * current， use in 「API 」 Cline、 Qoder will to 「 no has 」，
 * need that in that 。 old ： 1 when is 
 * path，「」 「 this 1 」 is use that path 。
 */

import * as React from 'react'
import { Alert, InputGroup, InputGroupAddon, InputGroupInput, SegmentedControl, Button, Item } from '@ui'

import { isAddAccountProviderHidden } from '../../../ui-shared/provider-availability'
import { accountCountOf, shared } from './add-account-bridge'
import { presetOfProvider, providerPresetIcon, providerSiteLabel } from './add-provider-metadata'
import { WORKBUDDY_ENTRY_LABEL, WORKBUDDY_PROVIDER } from './add-account-configs'
import { DIALOG_SEG_CLASS } from './dialog-segmented'
import { AddGroup } from './add-account-kit'
import { t } from '../i18n'
import { Empty } from '@ui'

/** 1 ： / Agent / Web / API / already API / */
export type AccountType = 'all' | 'proxy' | 'web' | 'preset' | 'custom' | 'import'

export const TYPE_ALL: AccountType = 'all'
export const TYPE_PROXY: AccountType = 'proxy'
export const TYPE_WEB: AccountType = 'web'
export const TYPE_PRESET: AccountType = 'preset'
export const TYPE_CUSTOM: AccountType = 'custom'
export const TYPE_IMPORT: AccountType = 'import'

/**
 * 「」 by ： and 1 1 for （Agent / Web / API /
 * already API）， —— not 1 ， use to 「」
 * for not up ，。
 */
const GROUPED_TYPES: AccountType[] = [TYPE_PROXY, TYPE_WEB, TYPE_PRESET, TYPE_CUSTOM]

/**
 * 「」 is 。current **false**： this 1 no ， from up 。
 *
 * is 、 not is —— 、 after `/api/import/cc-switch` and CSS
 * （ add-provider-import.tsx）， etc this 1 this true can 。
 * after ： not → to not TYPE_IMPORT → not 、「」
 * not will ， is 「 use from no this 」 that path。
 */
export const IMPORT_SEGMENT_ENABLED = false

/** before （ not is provider id， is ） */
export const PRESET_CARD_PREFIX = 'preset:'
/** 「 new 」 that （ not is provider id， is ） */
export const NEW_PROVIDER_CARD_ID = '__new__'

/** 1 ：unknown by 「Agent」。 */
export function typeValueOf(value: string): AccountType {
  if (value === TYPE_ALL) return TYPE_ALL
  if (value === TYPE_PRESET || value === TYPE_CUSTOM || value === TYPE_WEB) return value
  if (IMPORT_SEGMENT_ENABLED && value === TYPE_IMPORT) return TYPE_IMPORT
  return TYPE_PROXY
}

/**
 * ： and 「」 ** use this 1 **。
 * must is 1 —— up 「API 」 is ， use 
 * for not up 「 this is that 1 」。Agent / Web is client （）， not 。
 */
function typeLabelOf(type: AccountType): string {
  if (type === TYPE_ALL) return t('全部')
  if (type === TYPE_WEB) return 'Web'
  if (type === TYPE_PRESET) return t('API 模板')
  if (type === TYPE_CUSTOM) return t('已建 API')
  if (type === TYPE_IMPORT) return t('导入')
  return 'Agent'
}

/**
 * ：`id → assets/providers/<file>.png`，clientdirectory
 * （ and as 1 ；AutoClaw / WorkBuddy 、Cline 
 * 、Accio / ZCode use 1 —— is 1 client）。
 * and no to 。
 * use ： （accounts-panels GroupBandRow）、
 * （models-page） and to in （checkin-page） use 1 ， not need 。
 */
export const PROVIDER_ICONS: Record<string, string> = {
  workbuddy: 'assets/providers/workbuddy.png',
  'workbuddy-intl': 'assets/providers/workbuddy.png',
  raccoon: 'assets/providers/raccoon.png',
  catpaw: 'assets/providers/catpaw.png',
  autoclaw: 'assets/providers/autoclaw.png',
  'autoclaw-intl': 'assets/providers/autoclaw.png',
  qoder: 'assets/providers/qoder.png',
  'qoder-intl': 'assets/providers/qoder.png',
  'cline-free': 'assets/providers/cline.png',
  'cline-pass': 'assets/providers/cline.png',
  accio: 'assets/providers/accio.png',
  'accio-cn': 'assets/providers/accio.png',
  zcode: 'assets/providers/zcode.png',
  'zcode-intl': 'assets/providers/zcode.png',
  codearts: 'assets/providers/codearts.png',
  trae: 'assets/providers/trae.png',
  // Loomy： `resources/app.asar` Windows 
  // （`build/icons/favicon-228.png`， and use as 1 ；
  // up ）
  loomy: 'assets/providers/loomy.png',
  // KukuAI：client `GenFlowPro.exe` RT_ICON （256×256 that ，
  // and use as 1 ）
  kuku: 'assets/providers/kuku.png',
  // MonkeyCode：client asar use （`electron/icon.png`， and 
  // use as 1 ； / use this 1 ， AutoClaw ）
  monkeycode: 'assets/providers/monkeycode.png',
  'monkeycode-intl': 'assets/providers/monkeycode.png',
  // Command Code / Antigravity： this no has can ， ——
  // Command Code use `apple-touch-icon.png`（ + use ；
  // brand that 「 + 」 in 13px will ），Antigravity
  // use press 「Icon - Full Color」（antigravity.google/press）
  commandcode: 'assets/providers/commandcode.png',
  antigravity: 'assets/providers/antigravity.png',
  // OfficeAce（ as ）： 
  // （`res-static.hc-cdn.cn/.../OfficeAce/ new .png`，176×176 RGBA，
  // 1 256×256 —— and use as 1 ）
  officeace: 'assets/providers/officeace.png',
  // OrcaTerm（client）： PWA 
  // （`cloudcache.tencent-cloud.com/qcloud/orcaterm/pwa/icon/orcaterm_192x192.png`，
  // and client / use as 1 ）
  orcaterm: 'assets/providers/orcaterm.png',
// ：client `resources/app.asar` use 
  // （`assets/icon.png`，512×512， and use as 1 ；
  // 1 256×256 —— and officeace 1 ）
  chatglm: 'assets/providers/chatglm.png',
  'deepseek-web': 'assets/providers/deepseek.png',
  // Tabbit ： `web.tabbit.com/favicon.ico`（ is 160×160 PNG），
  // use this 1 （ 1 ）。
  tabbit: 'assets/providers/tabbit.png',
  'tabbit-intl': 'assets/providers/tabbit.png',
  // ModelScope（）： favicon（`https://modelscope.cn/favicon.ico`，
  // 128×128 1 ， 1 256×256 —— and up is 1 ；
  // / use this 1 ， AutoClaw / MonkeyCode ）
  modelscope: 'assets/providers/modelscope.png',
  'modelscope-intl': 'assets/providers/modelscope.png',
  // AtomCode（AtomGit / GitCode）： 
  // （`https://cdn-news.gitcode.com/news/atomcode-icon1.png`，524×524 RGBA，
  // 1 256×256 —— and / client use as 1 ）。
  // ⚠️ `<link rel="icon">` to is GitCode **** favicon
  // （`cdn-static.gitcode.host/static/images/logo-favicon.png`）， not is ，
  // no has use 。
  atomcode: 'assets/providers/atomcode.png',
  // LobsterAI（ has 「 has 」）： use 
  // （`build/icons/png/256x256.png`， —— ， and 
  // `lobsterai.youdao.com/icon-192.png`、README `public/logo.png` 1 ，
  // is client）。⚠️ before use is sibling 
  // cpa-jethub-plugins that 24×24 use SVG， and not ， already 。
  lobsterai: 'assets/providers/lobsterai.png',
  doubaowork: 'assets/providers/doubaowork.png',
  cursor: 'assets/providers/cursor.svg',
  kiro: 'assets/providers/kiro.svg',
  // AStudio（ Astron Studio）：client（ N + ，
  // and use 1 ； 32×32 1 256×256 ）
  astudio: 'assets/providers/astudio.png',
  devin: 'assets/providers/devin.svg',
  factory: 'assets/providers/factory.svg',
  zed: 'assets/providers/zed.svg',
  grok: 'assets/providers/grok.svg',
  'minimax-code': 'assets/providers/minimax-code.svg',
  'minimax-code-global': 'assets/providers/minimax-code.svg',
  qwenwork: 'assets/providers/qwenwork.svg',
  'qwenwork-intl': 'assets/providers/qwenwork.svg',
  'qwenwork-enterprise': 'assets/providers/qwenwork.svg',
  'kimi-work': 'assets/providers/kimi-work.svg',
  opendesign: 'assets/providers/opendesign.svg',
  'nous-portal': 'assets/providers/nous-portal.svg',
  'claude-subscription': 'assets/providers/anthropic.png',
  'codex-subscription': 'assets/providers/openai.png',
  'chatgpt-subscription': 'assets/providers/openai.png',
  'copilot-subscription': 'assets/providers/copilot.svg',
  'gemini-subscription': 'assets/providers/gemini-cli.svg',
}

/**
 * Agent ** not ** （ provider id）。 before has 
 * `cline-pass`：
 * ① supports API Key （ and OAuth token 1 Bearer ）， already to 
 * 「 API」 Cline Pass （`preset-providers.js` `clinepass`）；
 * ② after Provider and not —— / new / delete，
 * will cline-pass（ that accounts-domain 
 * providerSummaries providers need ， not file）；
 * ③ this is （ and `workbuddy-intl` that ： 1 ），** not need 
 * bug **。
 * Note by id ：`cline-free`（） not ， in Agent 。
 */
const HIDDEN_AGENT_PROVIDERS = new Set(['cline-pass'])

type CardItem = {
  id: string
  label: string
  count: number
  preset?: boolean
  custom?: boolean
  icon?: string
  site?: string
  template?: string
}

/** new ； already API by ID ， not by or and 。 */
function providerCards(accountType: AccountType): CardItem[] {
  const providers = shared().wbProviders
  if (accountType === TYPE_PRESET) {
    return (shared().wbPresetProviders?.list || []).map(preset => ({
      id: PRESET_CARD_PREFIX + preset.key,
      label: preset.name,
      count: 0,
      preset: true,
    }))
  }
  if (accountType === TYPE_CUSTOM) {
    return (providers?.customList?.() || []).map(provider => ({
      id: provider.id,
      label: provider.name || provider.id,
      count: accountCountOf(provider.id),
      custom: true,
      icon: providerPresetIcon(provider),
      site: providerSiteLabel(provider.baseUrl),
      template: presetOfProvider(provider)?.name,
    }))
  }
  // Cline Pass not in this 1 （ and by HIDDEN_AGENT_PROVIDERS）：
  // in before need that ， after not 。
  // in ****（PickStep ） up ： / 
  // is 1 providers need ， has （accounts-domain 
  // providerSummaries、models-page ）， not this ， cline-pass
  // 。
  const list = (providers?.all?.() || [])
    .filter(item => !isAddAccountProviderHidden(String(item?.id || '')))
    .filter(item => (item.category === 'web') === (accountType === TYPE_WEB))
    .filter(item => accountType === TYPE_WEB || !HIDDEN_AGENT_PROVIDERS.has(String(item?.id || '')))
  // WorkBuddy this and use ** in **（ not ） not is ：
  // this has 「」， from this 1 （
  // `WORKBUDDY_ENTRY_LABEL` ）。 and down that 「 not 」 is 1 
  // ： 1 、 in 。
  // （「AutoClaw 」 this ） 1 ： and by ，
  // not in will in this in 
  const labelOf = (id: string, fallback: string): string => {
    if (id === 'minimax-code') return t('MiniMax Code 国内版')
    if (id === 'minimax-code-global') return t('MiniMax Code 国际版')
    // already by 「」， and meta 1 
    // （` ` / ` `）。 this need 1 ： to 
    // label is in ， has t() will its 。
    if (id === 'qwenwork') return t('千问办公 国内版')
    if (id === 'qwenwork-intl') return t('千问办公 国际版')
    if (id === 'qwenwork-enterprise') return t('QwenWork 千问办公企业版')
    return id === WORKBUDDY_PROVIDER ? WORKBUDDY_ENTRY_LABEL : t(shared().wbProviders?.labelOf?.(id) || fallback)
  }
  // need as empty** not WorkBuddy 1 **（2026-10 ）：
  // before that by is 「 not can as 1 empty」，
  // and in ， no when 503 —— 1 not 
  // empty（ use as can ）。empty to ：
  // 「」， and to 1 。
  //
  // 「 need no to 」 that 1 empty：`add-account-modal` reset already will in 
  // as empty when 1 （ that `providers?.load?.().then(bump)`），
  // empty 1 down 。 this 1 failure 。
  const cards: CardItem[] = list.map(item => {
    const id = String(item.id || '')
    return {
      id,
      label: labelOf(id, String(item.label || item.id || '')),
      count: Number(item.count) || 0,
    }
  })
  // WorkBuddy ** not this **： already by that 「」
  // （ 1 WorkBuddyBlock ）， will use as need 
  // ， to when that not will （`add-account-modal` by 
  // `provider === 'workbuddy'` ， is 1 empty）。
  // after is provider，**** is 1 —— and AutoClaw /
  // Accio / ZCode that （ 1 、 no has ） not ， this is ：
  // WorkBuddy and 1 ， 1 for use 。
  const visible = cards.filter(item => item.id !== 'workbuddy-intl')
  // ：AutoClaw and Qoder need （ 1 ）
  // and ** in before ** —— need to is ，Qoder in Qoder after 
  // （ after none need ）；AutoClaw in Cline after ， need before 。
  const from = visible.findIndex(item => item.id === 'autoclaw-intl')
  const to = visible.findIndex(item => item.id === 'autoclaw')
  if (to >= 0 && from > to) visible.splice(to + 1, 0, visible.splice(from, 1)[0])
  return visible
}

/** ： （directory need ）， its use */
function Logo({ item }: { item: CardItem }): React.ReactElement {
  const icon = item.id.startsWith(PRESET_CARD_PREFIX)
    ? shared().wbPresetProviders?.iconOf?.(item.id.slice(PRESET_CARD_PREFIX.length)) || ''
    : item.icon || PROVIDER_ICONS[item.id]
  if (icon) {
    return (
      <span className='add-provider-logo has-icon'>
        <img src={icon} alt='' loading='lazy' />
      </span>
    )
  }
  const initial = String(item.label || '?').trim().slice(0, 1).toUpperCase() || '?'
  return <span className='add-provider-logo'>{initial}</span>
}

function cardMeta(item: CardItem): string {
  if (item.preset) return t('使用模板新建提供商')
  if (item.custom) return item.count ? t('{n} 个账号，点击添加', { n: item.count }) : t('点击添加账号')
  return item.count ? t('{n} 个账号', { n: item.count }) : t('还没有账号')
}

function ProviderCard({ item, onPick }: { item: CardItem; onPick: (id: string) => void }): React.ReactElement {
  return (
    <Item render={<Button variant='outline' />} variant='outline'
      className='add-provider-card h-auto whitespace-normal text-left'
      data-provider={item.id}
      title={[item.label, item.site, item.template].filter(Boolean).join(' · ')}
      role='option'
      onClick={() => onPick(item.id)}
    >
      <Logo item={item} />
      <span className='add-provider-info'>
        <span className='add-provider-name'>{item.label}</span>
        {item.site ? <span className='add-provider-meta'>{[item.site, item.template].filter(Boolean).join(' · ')}</span> : null}
        <span className='add-provider-meta'>{cardMeta(item)}</span>
      </span>
      <span className='add-provider-go'>›</span>
    </Item>
  )
}

/**
 * 「 new 」： in up ， by that to 「 new 」。
 * to 1 （ and its ）：「OpenAI / Anthropic 」 that to 。
 */
function NewProviderCard({ onPick }: { onPick: (id: string) => void }): React.ReactElement {
  return (
    <Item render={<Button variant='dashed' />} variant='outline'
      className='add-provider-card is-new h-auto whitespace-normal text-left'
      data-provider={NEW_PROVIDER_CARD_ID}
      role='option'
      onClick={() => onPick(NEW_PROVIDER_CARD_ID)}
    >
      <span className='add-provider-logo is-new'>＋</span>
      <span className='add-provider-info'>
        <span className='add-provider-name'>{t('新建自定义提供商')}</span>
        <span className='add-provider-meta'>{t('接入一个兼容上游')}</span>
      </span>
      <span className='add-provider-go'>›</span>
    </Item>
  )
}

export type PickStepProps = {
  accountType: AccountType
  onAccountTypeChange: (value: AccountType) => void
  search: string
  onSearchChange: (value: string) => void
  onPick: (id: string) => void
  /** need / directory to after （ not and ） */
  version: number
  /** ： and （ 1 ） */
  importing: boolean
  /** （ when not in ）： and ， 1 */
  importPanel?: React.ReactNode
}

export function PickStep({
  accountType,
  onAccountTypeChange,
  search,
  onSearchChange,
  onPick,
  version,
  importing,
  importPanel,
}: PickStepProps): React.ReactElement {
  const hasAgentCards = React.useMemo(() => providerCards(TYPE_PROXY).length > 0, [version])
  const hasWebCards = React.useMemo(() => providerCards(TYPE_WEB).length > 0, [version])
  const hiddenType = (accountType === TYPE_PROXY && !hasAgentCards)
    || (accountType === TYPE_WEB && !hasWebCards)
  const effectiveType = hiddenType ? (hasWebCards ? TYPE_WEB : TYPE_PRESET) : accountType
  const keyword = search.trim().toLowerCase()
  const hitOf = (item: CardItem): boolean => !keyword
    || [item.label, item.site, item.template].filter(Boolean).join(' ').toLowerCase().includes(keyword)
  // has when 「 new 」 that —— use in is 1 。
  // 「」 in 「 already API」 1 （ and 1 ）
  const newCard = !keyword && (effectiveType === TYPE_CUSTOM || effectiveType === TYPE_ALL)
  const grouped = effectiveType === TYPE_ALL
  /** this 1 ： has that 1 （ not ），「」 by */
  const groups = (grouped ? GROUPED_TYPES : [effectiveType]).map(type => ({
    type,
    label: typeLabelOf(type),
    items: providerCards(type).filter(hitOf),
  }))
  // 「」：empty not （ 1 ），otherwise 1 empty。
  // 1 is 「 already API」—— 「 new 」 that in ， can 
  const shownGroups = groups.filter(group => group.items.length > 0
    || (group.type === TYPE_CUSTOM && newCard))
  const hit = shownGroups.flatMap(group => group.items)
  // in ：「」 that 1 is （ no when total）。
  // not is —— in 「API 」 Cline when 1， etc 「」 1 
  const allHitCount = GROUPED_TYPES
    .reduce((sum, type) => sum + providerCards(type).filter(hitOf).length, 0)
  // current 1 no has 、 has ： this is 「」 need that ，
  // to 1 ， not is to 「」。
  // down current in as 0， in is 「 its 」 in 
  const otherHitCount = keyword && !grouped && !hit.length ? allHitCount : 0
  // Agent and Web already and 。

  return (
    // and 1 ： is 1 （「 to up 」→
    // 「 to 1 」）， will as is 。
    // AddGroup label（ in ）， 1 is + 。
    <AddGroup label={t('选择提供商')}>
      {/* and 1 ：， is this 1 */}
      <div className='add-pick-row' style={importing ? { display: 'none' } : undefined}>
        <SegmentedControl
          aria-label={t('账号类型')}
          className={DIALOG_SEG_CLASS}
          options={[
            // default in 「」： 1 ， not this 1 
            { value: TYPE_ALL, label: typeLabelOf(TYPE_ALL), count: allHitCount },
            // Agent in in when （ hasAgentCards）
            ...(hasAgentCards ? [{ value: TYPE_PROXY, label: typeLabelOf(TYPE_PROXY) }] : []),
            ...(hasWebCards ? [{ value: TYPE_WEB, label: typeLabelOf(TYPE_WEB) }] : []),
            { value: TYPE_PRESET, label: typeLabelOf(TYPE_PRESET) },
            { value: TYPE_CUSTOM, label: typeLabelOf(TYPE_CUSTOM) },
            // 「」 when （ IMPORT_SEGMENT_ENABLED）： not 
            ...(IMPORT_SEGMENT_ENABLED ? [{ value: TYPE_IMPORT, label: typeLabelOf(TYPE_IMPORT) }] : []),
          ]}
          value={effectiveType}
          onValueChange={value => onAccountTypeChange(typeValueOf(value))}
        />
        <span id='add-search-wrap' className='add-provider-search ml-auto max-w-[340px] min-w-0 flex-[1_1_200px]'>
          <InputGroup>
            <InputGroupAddon>⌕</InputGroupAddon>
            <InputGroupInput
              id='add-provider-search'
              type='search'
              placeholder={t('搜索提供商…')}
              autoComplete='off'
              aria-label={t('搜索提供商')}
              value={search}
              onChange={event => onSearchChange(event.currentTarget.value)}
            />
          </InputGroup>
        </span>
      </div>
      {/* in ： in up （ not ）， 1 down to 「」 */}
      {otherHitCount > 0 ? (
        <Alert className='flex items-center gap-3'>
          <span>
            {t('当前分类里没有「{keyword}」，其它分类有 {n} 个匹配', {
              keyword: search.trim(),
              n: otherHitCount,
            })}
          </span>
          <Button size='sm' variant='outline' type='button' className='add-provider-hint-go'
            onClick={() => onAccountTypeChange(TYPE_ALL)}>
            {t('看全部匹配')}
          </Button>
        </Alert>
      ) : null}
      {/* in up （ and ）， in before ——
           1 ，， not will */}
      {importPanel}
      <div
        className='add-provider-grid'
        id='add-provider-grid'
        role='listbox'
        aria-label={t('选择要添加账号的提供商')}
        style={importing ? { display: 'none' } : undefined}
      >
        {shownGroups.map(group => (
          // is 1 （grid-column: 1/-1）， is ——
          // 1 Fragment not is wrapper， not will 1 
          <React.Fragment key={group.type}>
            {grouped ? (
              <div className='add-provider-band' role='group' aria-label={group.label}>
                <span className='add-provider-band-name'>{group.label}</span>
                {group.items.length ? (
                  // use 「{count} 」（ ）： 1 
                  // in must is 1 ， new 1 will 1 need empty
                  <span className='add-provider-band-stat'>{t('{count} 家', { count: group.items.length })}</span>
                ) : null}
                <span className='add-provider-band-fill' />
                <Button size='xs' variant='ghost' type='button' className='add-provider-band-only'
                  onClick={() => onAccountTypeChange(group.type)}>
                  {t('只看这一类')}
                </Button>
              </div>
            ) : null}
            {newCard && group.type === TYPE_CUSTOM ? <NewProviderCard onPick={onPick} /> : null}
            {group.items.map(item => <ProviderCard key={item.id} item={item} onPick={onPick} />)}
          </React.Fragment>
        ))}
        {!hit.length && keyword ? (
          <Empty className='add-provider-empty'>{t('没有匹配「{keyword}」的提供商', { keyword: search.trim() })}</Empty>
        ) : null}
        {/* 1 no has 、 no in 、 no has 「 new 」= no 。 this is 
            ****（ and in ）， need not is 
            ； to 1 by ， use 。
            「」 this 1 ：empty no when ， that 1 no has 。 */}
        {!hit.length && !newCard && !keyword
          && (effectiveType === TYPE_PROXY || effectiveType === TYPE_ALL) ? (
            <Empty className='add-provider-empty'>
              <p className='mb-2'>{t('请安装并启用对应的提供商插件；列表为空时请刷新后重试。')}</p>
              <Button size='default' variant='ghost'
                type='button'
                className='text-primary underline underline-offset-2'
                onClick={() => shared().wbApp?.showPage?.('plugins')}
              >
                {t('去安装提供商插件包')}
              </Button>
            </Empty>
          ) : null}
      </div>
    </AddGroup>
  )
}
