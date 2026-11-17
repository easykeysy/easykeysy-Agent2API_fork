import * as React from 'react'
import {
  Badge, Button, Checkbox, Dialog, DialogBody, DialogContent, DialogFooter,
  DialogHeader, DialogTitle, Input, Label, Select, SelectContent, SelectItem,
  SelectTrigger, SelectValue, SegmentedControl, MultiSelect, Switch, Table, TableBody, TableCell, TableHead,
  TableHeader, TableRow, Textarea,
} from '@ui'
import { t } from '../i18n'
import {
  SANITIZE_ROLES, validateSanitizeRule, visibleRuleText, type SanitizeRule, type SanitizeState,
} from './settings-sanitize-model'
import { Alert } from '@ui'

type RuleEditorProps = {
  rule: SanitizeRule
  isNew: boolean
  limits: SanitizeState['limits']
  roleScopeEditable: boolean
  onApply: (rule: SanitizeRule) => void
  onClose: () => void
}

export function SanitizeRuleEditor({ rule, isNew, limits, roleScopeEditable, onApply, onClose }: RuleEditorProps) {
  const [draft, setDraft] = React.useState(rule)
  const [error, setError] = React.useState('')
  const types = [
    { value: 'literal', label: t('Exact text') },
    { value: 'regex', label: t('Regular expression') },
  ]

  function update(patch: Partial<SanitizeRule>): void {
    setDraft(current => ({ ...current, ...patch }))
    setError('')
  }

  function apply(): void {
    const message = validateSanitizeRule(draft, limits)
    if (message) { setError(message); return }
    onApply({ ...draft, name: draft.name.trim() })
  }

  return (
    <Dialog open onOpenChange={open => { if (!open) onClose() }}>
      <DialogContent overlayForceRender className='w-[min(760px,calc(100vw-48px))]'>
        <DialogHeader><DialogTitle>{isNew ? t('Add masking rule') : t('Edit masking rule')}</DialogTitle></DialogHeader>
        <DialogBody>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='sanitize-rule-name'>{t('Rule name')}</Label>
            <Input id='sanitize-rule-name' value={draft.name}
              onChange={event => update({ name: event.currentTarget.value })} />
          </div>
          <div className='flex flex-wrap items-center gap-3'>
            <Label htmlFor='sanitize-rule-type'>{t('Match type')}</Label>
            <Select value={draft.matchType} onValueChange={value => {
              if (value === 'literal' || value === 'regex') update({ matchType: value, repeat: false })
            }}>
              <SelectTrigger id='sanitize-rule-type' className='w-[160px]'>
                <SelectValue>{types.find(item => item.value === draft.matchType)?.label}</SelectValue>
              </SelectTrigger>
              <SelectContent>{types.map(item => <SelectItem key={item.value} value={item.value}>{item.label}</SelectItem>)}</SelectContent>
            </Select>
            <Label htmlFor='sanitize-rule-enabled' className='ml-auto'>{t('Enable rule')}</Label>
            <Switch id='sanitize-rule-enabled' checked={draft.enabled} onCheckedChange={enabled => update({ enabled })} />
          </div>
          {roleScopeEditable ? <div className='flex flex-col gap-2'>
            <div className='flex flex-wrap items-center gap-3'>
              <Label htmlFor='sanitize-rule-roles'>{t('Message roles')}</Label>
              <SegmentedControl aria-label={t('Message roles')}
                value={draft.roles.length ? 'selected' : 'all'}
                options={[{ value: 'all', label: t('All roles') }, { value: 'selected', label: t('Selected roles') }]}
                onValueChange={value => update({ roles: value === 'selected'
                  ? ['system', 'developer', 'assistant'] : [] })} />
            </div>
            {draft.roles.length ? <MultiSelect id='sanitize-rule-roles' value={draft.roles}
              options={SANITIZE_ROLES.map(role => ({ value: role, label: role }))}
              onValueChange={roles => update({ roles: SANITIZE_ROLES.filter(role => roles.includes(role)) })}
              placeholder={t('Selected roles')} className='w-full' /> : null}
            <div className='hint'>{t('All roles keeps the existing behavior, including reasoning content and tool arguments. Selected roles rewrites only message content; unselected roles, reasoning content and tool arguments remain unchanged.')}</div>
          </div> : null}
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='sanitize-rule-pattern'>{t('Match pattern')}</Label>
            <Textarea id='sanitize-rule-pattern' rows={4} spellCheck={false} value={draft.pattern}
              onChange={event => update({ pattern: event.currentTarget.value })} />
            <div className='hint'>{draft.matchType === 'literal'
              ? t('Matches the exact text, case-sensitively. Leading and trailing whitespace is preserved.')
              : t('Uses Rust regular expression syntax, validated by the backend when saving. Lookaround and backreferences are not supported.')}</div>
          </div>
          <div className='flex flex-col gap-1.5'>
            <Label htmlFor='sanitize-rule-replacement'>{t('Replacement text')}</Label>
            <Textarea id='sanitize-rule-replacement' rows={3} spellCheck={false} value={draft.replacement}
              onChange={event => update({ replacement: event.currentTarget.value, repeat: event.currentTarget.value ? false : draft.repeat })} />
            <div className='hint'>{t('Leave empty to delete matched text. Replacement text is inserted literally; $ is not a capture-group reference.')}</div>
          </div>
          {draft.matchType === 'regex' && !draft.replacement ? (
            <div className='flex items-center gap-2'>
              <Checkbox id='sanitize-rule-repeat' checked={draft.repeat} onCheckedChange={repeat => update({ repeat })} />
              <Label htmlFor='sanitize-rule-repeat'>{t('Repeat until unchanged (regular-expression deletion only)')}</Label>
            </div>
          ) : null}
          {/[\u200b]/.test(draft.pattern) ? <div className='hint'>{t('The matched original text contains zero-width spaces; the vocabulary preview uses\\u{200b} mark it; keep that character when editing.')}</div> : null}
          {error ? <Alert variant='destructive' role='alert' className='text-xs text-destructive'>{error}</Alert> : null}
          <div className='hint'>{t('Confirm adds the rule to the draft. Click Save settings to apply it.')}</div>
        </DialogBody>
        <DialogFooter>
          <Button variant='outline' onClick={onClose}>{t('Cancel')}</Button>
          <Button onClick={apply}>{t('Confirm')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export function SanitizeRulesTable({ rules, defaults, disabled, onChange, onEdit }: {
  rules: SanitizeRule[]
  defaults: SanitizeRule[]
  disabled: boolean
  onChange: (rules: SanitizeRule[]) => void
  onEdit: (rule: SanitizeRule) => void
}) {
  function move(index: number, offset: number): void {
    const next = [...rules]
    const moved = next[index]
    next[index] = next[index + offset]
    next[index + offset] = moved
    onChange(next)
  }

  return (
    <Table className='min-w-[820px]' containerClassName='max-h-[440px] rounded-md border border-border'>
      <TableHeader><TableRow>
        <TableHead>{t('enabled')}</TableHead>
        <TableHead>{t('Rule / match type')}</TableHead>
        <TableHead>{t('Message roles')}</TableHead>
        <TableHead>{t('Match pattern')}</TableHead>
        <TableHead>{t('Replacement text')}</TableHead>
        <TableHead className='text-right'>{t('Actions')}</TableHead>
      </TableRow></TableHeader>
      <TableBody>{rules.length ? rules.map((rule, index) => (
        <TableRow key={rule.id}>
          <TableCell>
            <Switch checked={rule.enabled} disabled={disabled} aria-label={t('Enable rule: {name}', { name: rule.name })}
              onCheckedChange={enabled => onChange(rules.map(item => item.id === rule.id ? { ...item, enabled } : item))} />
          </TableCell>
          <TableCell>
            <div className='max-w-[180px] break-words'>{rule.name}</div>
            <div className='mt-1 flex flex-wrap gap-1'>
              <Badge variant='outline'>{rule.matchType === 'regex' ? t('Regular expression') : t('Exact text')}</Badge>
              {defaults.some(item => item.id === rule.id) ? <Badge variant='outline'>{t('Built-in')}</Badge> : null}
              {rule.repeat ? <Badge variant='outline'>{t('Repeat cleanup')}</Badge> : null}
            </div>
          </TableCell>
          <TableCell><div className='flex max-w-[180px] flex-wrap gap-1'>
            {rule.roles.length ? rule.roles.map(role => <Badge key={role} variant='outline'>{role}</Badge>)
              : <Badge variant='outline'>{t('All roles')}</Badge>}
          </div></TableCell>
          <TableCell><span className='line-clamp-3 max-w-[260px] whitespace-pre-wrap break-all font-mono text-xs'
            title={visibleRuleText(rule.pattern)}>{visibleRuleText(rule.pattern)}</span></TableCell>
          <TableCell>{rule.replacement
            ? <span className='line-clamp-3 max-w-[260px] whitespace-pre-wrap break-all font-mono text-xs'
                title={visibleRuleText(rule.replacement)}>{visibleRuleText(rule.replacement)}</span>
            : <Badge variant='outline'>{t('Delete matched text')}</Badge>}</TableCell>
          <TableCell>
            <div className='flex justify-end gap-1'>
              <Button variant='ghost' size='icon-xs' disabled={disabled || index === 0}
                title={t('Move rule up')} aria-label={t('Move rule up')} onClick={() => move(index, -1)}>↑</Button>
              <Button variant='ghost' size='icon-xs' disabled={disabled || index === rules.length - 1}
                title={t('Move rule down')} aria-label={t('Move rule down')} onClick={() => move(index, 1)}>↓</Button>
              <Button variant='outline' size='xs' disabled={disabled} onClick={() => onEdit(rule)}>{t('Edit')}</Button>
              <Button variant='destructive' size='xs' disabled={disabled}
                onClick={() => onChange(rules.filter(item => item.id !== rule.id))}>{t('Delete')}</Button>
            </div>
          </TableCell>
        </TableRow>
      )) : <TableRow><TableCell colSpan={6} className='py-5 text-center text-muted-foreground'>
        {t('The rule list is empty; no fingerprints will be rewritten. Add a rule or restore the defaults.')}
      </TableCell></TableRow>}</TableBody>
    </Table>
  )
}
