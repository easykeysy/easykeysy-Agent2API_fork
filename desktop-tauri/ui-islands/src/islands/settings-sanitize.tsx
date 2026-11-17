import * as React from 'react'
import {
  Alert, Badge, Button, Card, CardContent, CardHeader, Dialog, DialogBody, DialogClose,
  DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogTrigger, Label,
  MultiSelect, SegmentedControl, Switch,
} from '@ui'
import { t } from '../i18n'
import { NOTES, STATES, TIPS } from './settings-model'
import { refreshSanitize, saveSanitizeConfig, type SettingsSnapshot } from './settings-state'
import { sameRules, type SanitizePatch, type SanitizeRule } from './settings-sanitize-model'
import { SanitizeRuleEditor, SanitizeRulesTable } from './settings-sanitize-rules'

export function SanitizePanel({ snap }: { snap: SettingsSnapshot }) {
  const state = snap.sanitize
  const [on, setOn] = React.useState(state.on)
  const [rules, setRules] = React.useState(state.rules)
  const [providers, setProviders] = React.useState(state.providers)
  const [scope, setScope] = React.useState<'all' | 'selected'>(state.providers.length ? 'selected' : 'all')
  const [restore, setRestore] = React.useState(false)
  const [rulesOpen, setRulesOpen] = React.useState(false)
  const [editor, setEditor] = React.useState<{ rule: SanitizeRule; isNew: boolean } | null>(null)
  const busy = snap.busy === 'sanitize'
  const ready = state.status === 'ready'
  const editable = ready && state.editable
  const disabled = !editable || busy

  React.useEffect(() => {
    setOn(state.on)
    setRules(state.rules)
    setProviders(state.providers)
    setScope(state.providers.length ? 'selected' : 'all')
    setRestore(false)
    setEditor(null)
  }, [state])

  const selected = scope === 'all' ? [] : providers
  const sameProviders = selected.length === state.providers.length
    && selected.every(id => state.providers.includes(id))
  const dirty = ready && (on !== state.on || !sameRules(rules, state.rules)
    || !sameProviders || (restore && state.customRules))
  const defaultLabel = restore || (!state.customRules && sameRules(rules, state.defaults))
  const saveStatus = <span className='hint mr-auto'>
    {dirty ? t('There are unsaved changes') : t('Current settings are saved')}{!state.on ? t('; rules do not run while masking is disabled') : ''}
  </span>

  function changeRules(next: SanitizeRule[]): void {
    setRules(next)
    setRestore(false)
  }

  function addRule(): void {
    setEditor({ isNew: true, rule: {
      id: `rule-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`,
      name: '', enabled: true, matchType: 'literal', pattern: '', replacement: '', repeat: false, roles: [],
    } })
  }

  async function save(): Promise<void> {
    const patch: SanitizePatch = {}
    if (on !== state.on) patch.sanitizeBlacklistFingerprints = on
    if (state.editable) {
      if (restore) patch.sanitizeFingerprintRules = null
      else if (!sameRules(rules, state.rules)) {
        patch.sanitizeFingerprintRules = rules.map(({ roles, ...rule }) =>
          roles.length ? { ...rule, roles } : rule)
      }
      if (!sameProviders) patch.sanitizeFingerprintProviders = selected
    }
    await saveSanitizeConfig(patch)
  }

  return (
    <Card render={<section />} className='panel'>
      <CardHeader className='panel-head'>
        <h2>{t('Fingerprint sanitizing')}</h2>
        <span className='tip-q' data-tip={TIPS.sanitize}></span>
        <Badge variant={!ready ? (state.status === 'loading' ? 'outline' : 'destructive') : state.on ? 'success' : 'outline'}>
          {!ready ? (state.status === 'loading' ? t('Checking…') : t('Unavailable')) : state.on ? t('Enabled') : t('Disabled')}
        </Badge>
        <div className='head-actions'>
          <Button id='btn-sanitize-refresh' variant='outline' size='sm' disabled={busy}
            onClick={() => void refreshSanitize()}>{t('Refresh')}</Button>
        </div>
      </CardHeader>
      <CardContent className='panel-body flex flex-col gap-3'>
        <div className='settings-switches'>
          <Label htmlFor='settings-sanitize' className='flex items-center gap-2'>
            <Switch id='settings-sanitize' checked={on} disabled={!ready || busy} onCheckedChange={setOn} />
            {t('Strip upstream review blacklist fingerprints (rewrite template sentences and headers in request bodies)')}
          </Label>
        </div>
        <div className='settings-state'>
          {!ready ? (state.status === 'loading' ? STATES.appLoading : STATES.sanitizeUnavailable)
            : !state.on ? STATES.sanitizeOff
              : t('{count} rules enabled, applied to {scope}.', {
                  count: state.rules.filter(rule => rule.enabled).length,
                  scope: state.providers.length
                    ? state.providers.map(id => state.options.find(option => option.id === id)?.label ?? id).join(t(', '))
                    : t('All providers'),
                })}
        </div>
        {!state.saved ? <Alert variant='destructive' role='alert' className='text-xs text-destructive'>
          {t('Fingerprint masking settings apply only to this run. Database writing failed; check the configuration directory.')}
        </Alert> : null}
        {editable ? <>
          <div className='retention-list'>
            <div className='retention-row'>
              <Label>{t('Apply to providers')}</Label>
              <SegmentedControl aria-label={t('Apply to providers')} value={scope}
                options={[{ value: 'all', label: t('All providers') },
                  { value: 'selected', label: t('Selected providers'), disabled: !state.options.length }]}
                disabled={disabled} onValueChange={value => {
                  if (value === 'all') { setScope('all'); return }
                  if (!providers.length) setProviders(state.options.map(option => option.id))
                  setScope('selected')
                }} />
            </div>
            {scope === 'selected' ? <div className='flex flex-col gap-2'>
              <Label htmlFor='sanitize-provider-select'>{t('Select providers (multiple allowed)')}</Label>
              <MultiSelect id='sanitize-provider-select' value={providers}
                options={state.options.map(option => ({ value: option.id, label: option.label }))}
                disabled={disabled} onValueChange={value => {
                  setProviders(value)
                  if (!value.length) setScope('all')
                }} className='w-full'
                placeholder={t('Select at least one provider')} />
            </div> : null}
            <div className='hint'>{t('Applies to all providers by default, including plugins and custom providers added later. With a selected scope, other providers are not masked. Clearing the selection returns to all providers.')}</div>
          </div>
          <Dialog open={rulesOpen} onOpenChange={open => {
            setRulesOpen(open)
            if (!open) setEditor(null)
          }}>
            <div className='flex flex-wrap items-center gap-2'>
              <h3 className='text-[12.5px] font-semibold'>{t('Masking rules')}</h3>
              <Badge variant='outline'>{defaultLabel ? t('Built-in defaults') : t('Custom')}</Badge>
              <span className='hint'>{t('{count} rules, executed in list order', { count: rules.length })}</span>
              <DialogTrigger id='btn-sanitize-rules-open' disabled={disabled}
                render={<Button variant='outline' size='sm' className='ml-auto' />}>
                {t('Edit')}
              </DialogTrigger>
            </div>
            <DialogContent className='w-[min(1120px,calc(100vw-48px))]'>
              <DialogHeader>
                <DialogTitle>{t('Masking rules')}</DialogTitle>
                <Badge variant='outline'>{defaultLabel ? t('Built-in defaults') : t('Custom')}</Badge>
              </DialogHeader>
              <DialogBody>
                <div className='flex flex-wrap items-center gap-2'>
                  <span className='hint'>{t('{count} rules, executed in list order', { count: rules.length })}</span>
                  <Button id='btn-sanitize-rule-add' variant='dashed' size='sm' className='ml-auto'
                    disabled={disabled || rules.length >= state.limits.maxRules} onClick={addRule}>{t('Add rule')}</Button>
                  <Button id='btn-sanitize-rules-reset' variant='outline' size='sm' disabled={disabled}
                    onClick={() => { setRules(state.defaults.map(rule => ({ ...rule }))); setRestore(true) }}>
                    {t('Restore default rules')}
                  </Button>
                </div>
                <SanitizeRulesTable rules={rules} defaults={state.defaults} disabled={disabled} onChange={changeRules}
                  onEdit={rule => setEditor({ rule, isNew: false })} />
                <div className='hint'>{t('Add, edit, delete, enable, disable and reorder rules. An empty replacement deletes matched text. Restoring defaults resets only the rules, not the switch or provider scope.')}</div>
                {!state.saved ? <Alert variant='destructive' role='alert' className='text-xs text-destructive'>
                  {t('Fingerprint masking settings apply only to this run. Database writing failed; check the configuration directory.')}
                </Alert> : null}
              </DialogBody>
              <DialogFooter className='flex-wrap justify-end'>
                {saveStatus}
                <DialogClose render={<Button variant='outline' />}>{t('Close')}</DialogClose>
                <Button id='btn-sanitize-rules-save' disabled={disabled || !dirty} onClick={() => void save()}>
                  {busy ? t('Saving…') : t('Save settings')}
                </Button>
              </DialogFooter>
              {editor ? <SanitizeRuleEditor rule={editor.rule} isNew={editor.isNew} limits={state.limits}
                roleScopeEditable={state.roleScopeEditable} onClose={() => setEditor(null)} onApply={rule => {
                  changeRules(editor.isNew ? [...rules, rule] : rules.map(item => item.id === rule.id ? rule : item))
                  setEditor(null)
                }} /> : null}
            </DialogContent>
          </Dialog>
        </> : ready ? <div className='hint'>{t('This backend does not support editable rules or provider scope. Update the backend and try again.')}</div> : null}
        {ready ? <div className='flex flex-wrap items-center justify-end gap-2'>
          {saveStatus}
          <Button id='btn-sanitize-save' disabled={busy || !dirty} onClick={() => void save()}>
            {busy ? t('Saving…') : t('Save settings')}
          </Button>
        </div> : null}
        <div className='hint retention-note'>{NOTES.sanitize}</div>
      </CardContent>
    </Card>
  )
}
