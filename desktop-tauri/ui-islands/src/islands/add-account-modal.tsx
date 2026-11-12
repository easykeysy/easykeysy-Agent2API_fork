/**
 * Agent2API · 「 / 」（React ，）。
 *
 * old ：ui/add-account.js（ + WorkBuddy that ）、
 * ui/add-provider-forms.js（ + + ）、
 * ui/add-provider-import.js（）、ui/add-custom-provider.js（）
 * and config（add-qoder / add-cline / add-accio / add-zcode）。
 *
 * ── ： and not ──────────────────────────────────
 * this not is ， is （ by need create / ）：open() div 
 * React root，close() when unmount + （ and confirm-dialog.tsx / conc-dialog.tsx
 * 1 ）。index.html that 47 （#add-modal）delete。
 *
 * ── （ and old for ）──────────────────────────
 * 1 #add-modal → this Dialog（）， 1100px；
 * 2 #add-form-modal → this Dialog（ + ）， 620px。
 * use Dialog：Esc / / / after by Base UI
 * ， Esc up that （ old document up keydown +
 * closeFormStep ）。** 1 as **： when old is DOM
 * （ and in ）， this will —— by 
 * （add-account-bridge draftProps）， that 1 
 * deviceId will （ use to 1 can ， up server as not ）。
 *
 * ── for （ use grep ，）────────────
 * window.wbAddAccountModal = { open, close } —— this 1 
 *   window.wbAccountAddForms  = { syncAddProvider, openNewCustomForm }
 * · syncAddProvider （account-panel.js） in 「」 by up use 
 * · openNewCustomForm 「＋ new 」 new 
 * old its （closeFormStep / registerAddForm / bindSeg / segValueOf /
 * setSegValue / SEG_EVENT） use ， old file 1 。
 */

import * as React from 'react'
import { createRoot } from 'react-dom/client'
import { Dialog, DialogBody, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@ui'

import { shared, type CustomProviderRecord, type SharedWindow } from './add-account-bridge'
import {
  BUILTIN_CONFIGS,
  CUSTOM_PROVIDER,
  WORKBUDDY_ENTRY_LABEL,
  WORKBUDDY_PROVIDER,
  configOf,
} from './add-account-configs'
import { CustomFootActions, CustomProviderBlock, type CustomMode } from './add-custom-provider'
import { ImportFootActions, ImportPanel } from './add-provider-import'
import {
  IMPORT_SEGMENT_ENABLED,
  NEW_PROVIDER_CARD_ID,
  PRESET_CARD_PREFIX,
  PROVIDER_ICONS,
  PickStep,
  TYPE_ALL,
  TYPE_IMPORT,
  type AccountType,
} from './add-provider-pick'
import { ProviderBlock, WorkBuddyBlock } from './add-provider-blocks'
import { DynamicProviderBlock } from './add-dynamic-provider'
import { DeepseekAccountBlock } from './add-provider-deepseek-block'
import '../styles/add-account-dialog.css'
import { t } from '../i18n'

/**
 * that ： ， its use 
 * （ and 1 1 ， not need ）。
 */
function ProviderMark({ provider, label }: { provider: string; label: string }): React.ReactElement {
  const icon = PROVIDER_ICONS[provider]
  if (icon) return <span className='acct-add-head-mark'><img src={icon} alt='' /></span>
  const initial = String(label || '?').trim().slice(0, 1).toUpperCase() || '?'
  return <span className='acct-add-head-mark' aria-hidden='true'>{initial}</span>
}

type Step = 'pick' | 'form'

type IslandState = {
  step: Step
  accountType: AccountType
  search: string
  /** current in 1 （provider id / 'workbuddy' / 'custom'） */
  provider: string
  /** from 1 already has when id（empty = no has ） */
  providerHint: string
  /** from when key（empty = not is from ） */
  presetKey: string
  /** 1 1 +1： / directory this 「 this 1 need 1 」 use */
  showToken: number
}

/**
 * （ and 「」） when ： 1 + **「」** + WorkBuddy。
 * 「」 not is Agent：current，default in Agent when ， 1 in 
 * （Cline Pass、 in ） will to 「 no has 」—— this 。
 * in ， 1 when that is 。
 */
const INITIAL: IslandState = {
  step: 'pick',
  accountType: TYPE_ALL,
  search: '',
  provider: WORKBUDDY_PROVIDER,
  providerHint: '',
  presetKey: '',
  showToken: 0,
}

/** to that （ after ， when ） */
export type IslandHandle = {
  /** and to 1 （ old resetAddStep， for syncAddProvider） */
  reset: () => void
  /** 「 new 」（ old openNewCustomForm） */
  openNewCustomForm: () => void
}

const customList = (): CustomProviderRecord[] => shared().wbProviders?.customList?.() || []

function AddAccountModal({
  onClose,
  onReady,
}: {
  onClose: () => void
  onReady: (handle: IslandHandle | null) => void
}): React.ReactElement {
  const [state, setState] = React.useState<IslandState>(INITIAL)
  /** need / directory to after （ not and ） */
  const [version, setVersion] = React.useState(0)
  /**
   * ：** in this createElement **， not is ref + useState。
   * after to is null， will by in 、 down 1 portal ——
   * 1 ， in `create()` when up click （sms-login.js /
   * autoclaw-oauth.js in create() when by id by ）。 use this ，
   * by already in ， after not 。
   */
  const [formFootHost] = React.useState(() => {
    const node = document.createElement('div')
    node.className = 'acct-add-foot-host'
    return node
  })
  const bump = React.useCallback(() => setVersion(value => value + 1), [])

  const reset = React.useCallback(() => {
    setState(INITIAL)
    const providers = shared().wbProviders
    // need no to （ that refresh） when 1 ：
    // otherwise up will 1 「 no has 」， its has 
    if (!(providers?.all?.() || []).length) void providers?.load?.().then(bump)
    // directory： to 「」 when already need 
    void providers?.refreshCustom?.().then(bump)
  }, [bump])

  const openNewCustomForm = React.useCallback(() => {
    reset()
    setState({ ...INITIAL, step: 'form', provider: CUSTOM_PROVIDER })
  }, [reset])

  React.useEffect(() => {
    onReady({ reset, openNewCustomForm })
    return () => onReady(null)
  }, [onReady, reset, openNewCustomForm])

  // when by （ old openModal that 1 refresh）：
  // up if in etc in ， this will new available
  React.useEffect(() => {
    void shared().wbWebLogin?.refresh?.()
  }, [])

  // directory before 1 ： 1 this 1 when down has （ old ）
  React.useEffect(() => {
    void shared().wbProviders?.refreshCustom?.().then(bump)
  }, [bump])

  // directory（ old onShow that refreshCustom）
  React.useEffect(() => {
    if (state.step !== 'form' || state.provider !== CUSTOM_PROVIDER) return
    void shared().wbProviders?.refreshCustom?.().then(bump)
  }, [state.step, state.provider, state.showToken, bump])

  const config = configOf(state.provider)
  const isCustom = state.provider === CUSTOM_PROVIDER
  const isDeepseek = state.provider === 'deepseek-web'
  const hasBlock = state.provider === WORKBUDDY_PROVIDER || isCustom || isDeepseek || Boolean(config)
  const customMode: CustomMode = state.providerHint ? 'existing' : 'create'
  const customLabel = state.providerHint
    ? customList().find(item => item.id === state.providerHint)?.name || ''
    : state.presetKey
      ? shared().wbPresetProviders?.presetOf?.(state.presetKey)?.name || ''
      : ''
  // WorkBuddy use in （ not ）： this has 「」，
  // 「」 use to （ and 1 that 1 ，
  // add-account-configs WORKBUDDY_ENTRY_LABEL）。 use ，
  // need 1 （「AutoClaw 」 this in in ）。
  const providerLabel = state.provider === WORKBUDDY_PROVIDER
    ? WORKBUDDY_ENTRY_LABEL
    : t(shared().wbProviders?.labelOf?.(state.provider) || state.provider)
  // in ： is 「」， not 
  const heading = isCustom
    ? (state.providerHint
      ? t('登录 / 添加 {name} 账号', { name: customLabel || t('自定义提供商') })
      : state.presetKey
        ? t('使用 {name} 模板新建提供商', { name: customLabel })
        : t('新建自定义提供商'))
    : isDeepseek ? t('添加 DeepSeek 账号') : t('登录 / 添加 {name} 账号', { name: providerLabel })

  /** 1 1 ： to （ pickProvider old ） */
  function pick(id: string): void {
    if (!id) return
    const entry = { step: 'form' as Step, showToken: state.showToken + 1 }
    if (id === NEW_PROVIDER_CARD_ID) {
      setState(prev => ({ ...prev, ...entry, provider: CUSTOM_PROVIDER, providerHint: '', presetKey: '' }))
    } else if (id.startsWith(PRESET_CARD_PREFIX)) {
      const presetKey = id.slice(PRESET_CARD_PREFIX.length)
      setState(prev => ({
        ...prev,
        ...entry,
        provider: CUSTOM_PROVIDER,
        providerHint: '',
        presetKey,
      }))
    } else if (customList().some(item => item.id === id)) {
      setState(prev => ({ ...prev, ...entry, provider: CUSTOM_PROVIDER, providerHint: id, presetKey: '' }))
    } else {
      setState(prev => ({ ...prev, ...entry, provider: id, providerHint: '', presetKey: '' }))
    }
  }

  const backToPick = (): void => setState(prev => ({ ...prev, step: 'pick' }))
  const importing = state.accountType === TYPE_IMPORT

  return (
    // 1 ：。 open（ as true）： is 「 when 、」，
    // 1 by onClose （Esc / / ✕ by Base UI to onOpenChange）。
    //
    // 2 （） in **DialogContent children **（ down that Dialog）。
    <Dialog open onOpenChange={next => { if (!next) onClose() }}>
      <DialogContent className='acct-add-dialog w-[min(1100px,calc(100vw-48px))]'>
        <DialogHeader>
          <DialogTitle>{t('添加账号')}</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <PickStep
            accountType={state.accountType}
            onAccountTypeChange={value => setState(prev => ({ ...prev, accountType: value }))}
            search={state.search}
            onSearchChange={value => setState(prev => ({ ...prev, search: value }))}
            onPick={pick}
            version={version}
            importing={importing}
            importPanel={
              IMPORT_SEGMENT_ENABLED ? <ImportPanel segmentOn={importing} /> : undefined
            }
          />
        </DialogBody>
        {/* this 「 in 1 」 （ 「」）；
             2 by down that ， */}
        {IMPORT_SEGMENT_ENABLED && importing ? (
          <DialogFooter>
            <ImportFootActions />
          </DialogFooter>
        ) : null}

        {/* 2 ： + 。 in up 1 ， 1 
            （ 800px is ； 1 ）。

             is this 1 can not can 「 in up 」 ， need ：
              · React —— Base UI 「」（ useDialogRoot 
                parentStore）， when Esc and by not ；
              · in ** DialogContent children** —— Dialog in 
                Popup in ， Portal will Portal 、 in Popup
                 after （Base UI is this after ）。 and DialogContent
                 when Portal will to body up ， as z-30， in DOM after in 
                 up —— is in down 。

            keepMounted： this 1 when ** not ** —— in 
            （ that 1 deviceId、）， 1 etc 「 in 
             deviceId up 」， use 「 → → 」 will to id。
             when Base UI will as inert， and not will 。

            overlayForceRender： this 1 need has （ down + empty
            ， and old #add-form-modal that ）。Base UI for default
             not ， when is 「empty」 （useDialogRoot outsidePress
            ）， this 1 not can 。 */}
        <Dialog open={state.step === 'form'} onOpenChange={next => { if (!next) backToPick() }}>
          <DialogContent keepMounted overlayForceRender className={isDeepseek
            ? 'compact-login-dialog acct-add-form-dialog w-[min(540px,calc(100vw-48px))]'
            : 'acct-add-form-dialog w-[min(620px,calc(100vw-48px))]'}>
            <DialogHeader>
              <ProviderMark provider={state.provider} label={providerLabel} />
              <DialogTitle>{heading}</DialogTitle>
            </DialogHeader>
            <DialogBody>
              {/* 、： in ，
                   etc 「 in deviceId up 」 */}
              <WorkBuddyBlock active={state.provider === WORKBUDDY_PROVIDER} footerHost={formFootHost} />
              {BUILTIN_CONFIGS.filter(item => item.provider !== 'deepseek-web').map(item => (
                <ProviderBlock
                  key={item.provider}
                  config={item}
                  active={state.provider === item.provider}
                  footerHost={formFootHost}
                />
              ))}
              <div
                className='add-provider-block'
                style={isCustom ? undefined : { display: 'none' }}
              >
                <CustomProviderBlock
                  mode={customMode}
                  providerHint={state.providerHint}
                  presetKey={state.presetKey}
                  showToken={state.showToken}
                  version={version}
                />
              </div>
              <DeepseekAccountBlock active={isDeepseek && state.step === 'form'}
                footerHost={formFootHost} onClose={backToPick} />
              {hasBlock ? null : (
                <DynamicProviderBlock
                  key={state.provider}
                  provider={state.provider}
                  label={providerLabel}
                  active={state.step === 'form'}
                  version={version}
                  footerHost={formFootHost}
                />
              )}
            </DialogBody>
            {/* this 1 1 1 ： by and portal （ add-account-kit
                 AddFoot）， by in this 。 as empty when ——
                 is `:has()`， not need need 。 */}
            <DialogFooter className='acct-add-foot'>
              {isCustom ? (
                <div className='acct-add-foot-host'>
                  <CustomFootActions mode={customMode} presetKey={state.presetKey} providerId={state.providerHint} />
                </div>
              ) : (
                <div
                  className='acct-add-foot-mount'
                  ref={node => {
                    if (node && formFootHost.parentElement !== node) node.append(formFootHost)
                  }}
                />
              )}
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </DialogContent>
    </Dialog>
  )
}

/* ─── ： and old window 1 ─── */

let root: ReturnType<typeof createRoot> | null = null
let host: HTMLElement | null = null
let handle: IslandHandle | null = null
/** after that 1 （open / openNewCustomForm in 1 tick to when use ） */
let pendingAction: (() => void) | null = null

function unmountModal(): void {
  if (root) {
    root.unmount()
    root = null
  }
  if (host) {
    host.remove()
    host = null
  }
  handle = null
}

function mountModal(): void {
  host = document.createElement('div')
  document.body.append(host)
  root = createRoot(host)
  root.render(
    <AddAccountModal
      onClose={closeModal}
      onReady={next => {
        handle = next
        if (!next) return
        const action = pendingAction
        pendingAction = null
        action?.()
      }}
    />,
  )
}

/**
 * ： 1 （ ）， and etc in 。
 *
 * etc ： in after ，otherwise by will 1 in use 。
 * not provider = 「 in etc 」： use 1 ， this none need 。
 * AutoClaw OAuth has 1 etc （、 etc ）， and up that 
 * not （ need that etc ）， 1 ——
 * no has when not 。
 */
function closeModal(): void {
  void shared().wbWebLogin?.cancelIfActive?.('')
  shared().wbAutoclawOauth?.cancel?.()
  unmountModal()
}

/** （ already when etc 「 and to 1 」） */
function openModal(): void {
  if (root) {
    handle?.reset()
    return
  }
  pendingAction = () => handle?.reset()
  mountModal()
}

/** 「 new 」：、，「 new 」 that 1 path */
function openNewCustomForm(): void {
  if (root) {
    handle?.openNewCustomForm()
    return
  }
  pendingAction = () => handle?.openNewCustomForm()
  mountModal()
}

// 「」 by （ that already will delete，id 1 ）。
// in index.html islands/ui.js ， in that by after ，DOM already 。
for (const id of ['btn-add-account', 'btn-add-account-2']) {
  document.getElementById(id)?.addEventListener('click', openModal)
}

// for use 「 + 」（SharedWindow already this ），
// not use declare global Window up —— and when not 1 will TS2717。
const bridgeWindow = window as unknown as SharedWindow
bridgeWindow.wbAddAccountModal = { open: openModal, close: closeModal }
bridgeWindow.wbAccountAddForms = {
  /** 「」 by ： by providers need and to WorkBuddy */
  syncAddProvider: () => {
    if (root) handle?.reset()
    else openModal()
  },
  openNewCustomForm,
}