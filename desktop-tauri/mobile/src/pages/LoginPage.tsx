import { Progress as UiProgress, Verification } from '@ui'
import { useEffect, useRef, useState } from 'react'
import { Icon } from '../components/Icon'
import { store, useStore } from '../state/store'
import { authenticate, fetchPanelStatus, solveCaptcha } from '../api/auth'
import { storeApiKey } from '../api/client'
import { baseLabel, serverIdentity } from '../lib/servers'
import { t } from '../i18n'
import { Alert, AppFrame, Button, Field as UiField, FieldLabel, Input } from '@ui'

/**
 * / （APK ）。
 *
 * ── Why is not is in up ────────────────────
 * is 1 「 no not 」 ， will after ， use 
 * as can 、 not in 1 。 to ：、
 * 、 。 when ， not will by to not 。
 *
 * ── in after ， not in this 1 ────────────────────────────────
 * when ，`POST /api/panel/login` and `/api/panel/setup` ** in 
 * server** 1 ALTCHA （ `server::api::panel::check_captcha`），
 * no or 1 400。client —— 、 APK、curl —— not 。
 * this 1 is ** use **：、 after 、 when up 。
 *
 * must。 1 ， use to 「」
 * by not in etc ； this 1 ，andfailure when to 。
 *
 * ── ──────────────────────────────────────────────
 * request `x-panel-auth-mode: body`，server response
 * （ `server::api::panel::tokens_in_body`）。 to after by id 
 * localStorage， after request `x-panel-token` —— request no has 
 * up （`Allow-Origin: *` down not cookie）。
 */
export function LoginPage({ mode, onDone }: { mode: 'login' | 'key'; onDone?: () => void }) {
  const { server, packaged } = useStore()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  /** this has no has —— no has is */
  const [setup, setSetup] = useState(false)
  const [statusKnown, setStatusKnown] = useState(false)
  /**
   * 。
   * idle no （）
   * solving in / 
   * ready already to can 
   * off no （ 400）
   * failed failure（ / ）， can 
   */
  const [captchaState, setCaptchaState] = useState<'idle' | 'solving' | 'ready' | 'off' | 'failed'>('idle')
  const [captcha, setCaptcha] = useState<string | null>(null)
  const [progress, setProgress] = useState(0)
  /** ： when up 1 ， to new */
  const generation = useRef(0)

  const runCaptcha = async () => {
    const mine = ++generation.current
    setCaptchaState('solving')
    setCaptcha(null)
    setProgress(0)
    try {
      const payload = await solveCaptcha(ratio => {
        if (generation.current === mine) setProgress(ratio)
      })
      if (generation.current !== mine) return
      if (payload) {
        setCaptcha(payload)
        setCaptchaState('ready')
      } else {
        // 400 = no ；failure（） by 「 not need need 」 ——
        // need need when server will 1 Reason
        setCaptchaState('off')
      }
    } catch {
      if (generation.current === mine) setCaptchaState('failed')
    }
  }

  // ： has no has 、 need not need 
  useEffect(() => {
    let alive = true
    void (async () => {
      try {
        const status = await fetchPanelStatus()
        if (!alive) return
        setSetup(!status.registered)
        setStatusKnown(true)
      } catch {
        if (!alive) return
        setStatusKnown(true)
      }
      if (!alive) return
      await runCaptcha()
    })()
    return () => {
      alive = false
      generation.current += 1
    }
    // in when 1 
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const submit = async () => {
    const identity = serverIdentity()
    if (mode === 'key') {
      const value = apiKey.trim()
      if (!value) return
      storeApiKey(value)
      store.setAuthLost(null)
      onDone?.()
      await store.refresh(true)
      return
    }

    if (!username.trim() || !password) {
      setError(t('请填写账号与密码'))
      return
    }
    // no ，server 400 —— in this 1 ， 1 
    if (captchaState === 'solving') {
      setError(t('人机验证还在计算，请稍候'))
      return
    }
    setBusy(true)
    setError('')
    const result = await authenticate({
      mode: setup ? 'setup' : 'login',
      username: username.trim(),
      password,
      captcha,
    })
    if (serverIdentity() !== identity) return
    setBusy(false)
    if ('error' in result) {
      setError(result.error)
      // 1 ：failure after new 1 （ 1 ， down need use new ）
      void runCaptcha()
      return
    }
    store.setAuthLost(null)
    onDone?.()
    await store.refresh(true)
  }

  const submitDisabled =
    busy ||
    (mode === 'key'
      ? !apiKey.trim()
      : !statusKnown || !username.trim() || !password || captchaState === 'solving')

  return (
    <AppFrame
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 80,
        display: 'flex',
        flexDirection: 'column',
        background: 'var(--m-bg)',
        overflowY: 'auto',
      }}
    >
      {/* ： need has ， not will in empty */}
      <div
        style={{
          flex: 'none',
          padding: 'calc(var(--m-safe-t) + 40px) 26px 22px',
        }}
      >
        <div
          style={{
            width: 48,
            height: 48,
            borderRadius: 14,
            display: 'grid',
            placeItems: 'center',
            background: 'var(--m-primary)',
            color: '#fff',
            boxShadow: '0 10px 24px -12px var(--m-primary)',
          }}
        >
          <Icon name='shield' style={{ width: 24, height: 24 }} />
        </div>
        <h1 style={{ margin: '16px 0 0', fontSize: 23, letterSpacing: '-0.02em' }}>
          {mode === 'key'
            ? t('需要网关 API Key')
            : setup
              ? t('创建管理员账号')
              : t('登录 Agent2API')}
        </h1>
        <p style={{ margin: '6px 0 0', fontSize: 12.5, lineHeight: 1.6, color: 'var(--m-text-3)' }}>
          {mode === 'key'
            ? t('这台网关没有管理员，用一把已启用的 API Key 访问。')
            : setup
              ? t('这台网关还没有管理员。设置一个账号密码，之后用它登录。')
              : t('输入这台网关的管理员账号与密码。')}
        </p>
      </div>

      <div style={{ flex: 1, padding: '0 20px', display: 'flex', flexDirection: 'column', gap: 10 }}>
        {/* current： before this 1 ， to 1 can */}
        <Button size='default' variant='ghost'
          type='button'
          className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left'
          disabled={busy}
          onClick={() => packaged && store.push('servers')}
          style={{
            borderRadius: 'var(--m-r-md)',
            border: '1px solid var(--m-line)',
            background: 'var(--m-surface)',
            minHeight: 56,
          }}
        >
          <span
            className='m-logo'
            style={{ background: 'var(--m-primary-soft)', borderColor: 'transparent', color: 'var(--m-primary)' }}
          >
            <Icon name='server' />
          </span>
          <div className='m-grow' style={{ minWidth: 0 }}>
            <div className='m-t1' style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {server?.name || baseLabel(server?.base || '') || t('未选择服务器')}
            </div>
            <div className='m-t2 m-mono' style={{ fontSize: 11.5 }}>
              {server?.base ? baseLabel(server.base) : t('点此添加服务器')}
            </div>
          </div>
          {packaged ? (
            <span className='m-chev'>
              <Icon name='chev' />
            </span>
          ) : null}
        </Button>

        {mode === 'key' ? (
          <Field
            label={t('API Key')}
            value={apiKey}
            onChange={setApiKey}
            placeholder='sk-…'
            type='password'
            autoComplete='off'
          />
        ) : (
          <>
            <Field
              label={t('账号')}
              value={username}
              onChange={setUsername}
              placeholder={t('管理员账号')}
              autoComplete='username'
              autoCapitalize='off'
              autoCorrect='off'
              spellCheck={false}
            />
            <Field
              label={t('密码')}
              value={password}
              onChange={setPassword}
              placeholder={setup ? t('设置密码（至少 8 位）') : t('密码')}
              type='password'
              autoComplete={setup ? 'new-password' : 'current-password'}
            />

            <div className='flex flex-col gap-1.5'>
              <Verification className='w-full justify-start whitespace-normal text-left'
                state={captchaState === 'ready' ? 'verified' : captchaState === 'solving' ? 'verifying' : captchaState === 'failed' ? 'error' : 'idle'}
                disabled={captchaState !== 'failed'} onClick={() => void runCaptcha()}>
                {captchaState === 'ready' ? t('人机验证已完成')
                  : captchaState === 'solving' ? t('正在完成人机验证…')
                    : captchaState === 'failed' ? t('人机验证失败')
                      : captchaState === 'off' ? t('这台网关未启用机器人校验') : t('人机验证')}
              </Verification>
              {captchaState === 'solving' && progress > 0 ? (
                <UiProgress value={Math.round(progress * 100)} aria-label={t('人机验证')} />
              ) : null}
              {captchaState === 'failed' ? <Button size='touch-sm' variant='outline' onClick={() => void runCaptcha()}>{t('重试')}</Button> : null}
            </div>
          </>
        )}

        {error ? <Alert variant='destructive'>{error}</Alert> : null}

        <Button size='touch' variant='default'
          type='button'
          className='m-btn w-full'
          style={{ marginTop: 4, height: 46 }}
          disabled={submitDisabled}
          onClick={() => void submit()}
        >
          {busy ? t('验证中…') : mode === 'key' ? t('保存并继续') : setup ? t('创建并进入') : t('登录')}
        </Button>

        {/*
           is ， use after not to 「set → 」——
           this to 1 ：，/ 1 after will 。
        */}
        {packaged ? (
          <Button size='default' variant='ghost'
            type='button'
            disabled={busy}
            onClick={() => store.push('servers')}
            style={{
              display: 'block',
              width: '100%',
              marginTop: 2,
              padding: '10px 0',
              border: 0,
              background: 'none',
              color: 'var(--m-primary)',
              fontSize: 12.5,
            }}
          >
            {t('换一台服务器')}
          </Button>
        ) : null}
      </div>

      <div
        style={{
          flex: 'none',
          padding: '16px 24px calc(var(--m-safe-b) + 20px)',
          textAlign: 'center',
          fontSize: 11,
          color: 'var(--m-text-3)',
        }}
      >
        {t('凭证只保存在这台手机上，按服务器分别存放')}
      </div>
    </AppFrame>
  )
}

/** （ placeholder ： after ） */
function Field({
  label,
  value,
  onChange,
  placeholder,
  type,
  autoComplete,
  autoCapitalize,
  autoCorrect,
  spellCheck,
}: {
  label: string
  value: string
  onChange: (next: string) => void
  placeholder?: string
  type?: string
  autoComplete?: string
  autoCapitalize?: string
  autoCorrect?: string
  spellCheck?: boolean
}) {
  return (
    <UiField className='gap-1.5'>
      <FieldLabel className='block text-[12px] text-muted-foreground'>{label}</FieldLabel>
      <Input
        value={value}
        onChange={event => onChange(event.target.value)}
        placeholder={placeholder}
        type={type}
        autoComplete={autoComplete}
        autoCapitalize={autoCapitalize}
        autoCorrect={autoCorrect}
        spellCheck={spellCheck}
        className={`h-[46px] text-base${type === 'password' ? ' font-mono' : ''}`}
        aria-label={label}
      />
    </UiField>
  )
}
