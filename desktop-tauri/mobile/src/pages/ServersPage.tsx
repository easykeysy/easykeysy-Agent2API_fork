import { useState } from 'react'
import { Icon } from '../components/Icon'
import { PageHead, PageIntro, SectionHead, ErrorNote } from '../components/ui'
import { store, useStore } from '../state/store'
import { probe } from '../api/client'
import { baseLabel, normalizeBase } from '../lib/servers'
import { t } from '../i18n'
import { Button, Card, Label, Input } from '@ui'
import { Badge } from '@ui'

/**
 * config（only APK）。
 *
 * ── Why 1 not is set ────────────────────────────
 * when no has ，all not to —— this when set
 * is empty 。must is ** before ** 1 ： no config not 。
 *
 * ── Why ────────────────────────────────────────
 * （、 `/m`、 no ） is failure。
 * before 1 `/health`， 「 not up 」， after 
 * 1 need 。success， use can is for that 。
 */
export function ServersPage() {
  const { servers, server, stack } = useStore()
  /** （ 「 1 」） not is when */
  const stacked = stack.length > 0
  const [editing, setEditing] = useState<string | null>(null)
  const [draftName, setDraftName] = useState('')
  const [draftBase, setDraftBase] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [probed, setProbed] = useState<{ version?: string } | null>(null)

  const startAdd = () => {
    setEditing('new')
    setDraftName('')
    setDraftBase('')
    setError('')
    setProbed(null)
  }

  const startEdit = (id: string) => {
    const entry = servers.find(item => item.id === id)
    if (!entry) return
    setEditing(id)
    setDraftName(entry.name)
    setDraftBase(baseLabel(entry.base))
    setError('')
    setProbed(null)
  }

  const cancel = () => {
    setEditing(null)
    setError('')
    setProbed(null)
  }

  const save = async () => {
    const base = normalizeBase(draftBase)
    if (!base) {
      setError(t('地址格式不对。填「IP 或域名:端口」，可带 http:// 或 https://，例如 192.168.1.23:3065'))
      return
    }
    setBusy(true)
    setError('')
    const result = await probe(base)
    setBusy(false)
    if (!result.ok) {
      setError(t('连不上这台网关。确认地址与端口正确、网关已启动，且已开启「局域网访问」；用 https 时还要确认证书有效'))
      return
    }
    setProbed({ version: result.version })
    if (editing === 'new') {
      store.addServer({ name: draftName, base })
    } else if (editing) {
      store.updateServer(editing, { name: draftName, base })
    }
    setEditing(null)
    setProbed(null)
    store.toast(t('已保存'))
    // to new after 1 will ； no will 
    void store.refresh(true).then(() => {
      // this 1 is from 「 1 」 ：，
      // （ need need ） will new up ， use not will in 
      if (store.get().stack.length > 0) store.back()
    })
  }

  return (
    <div className='m-page'>
      {/* as when already has return， not 1 */}
      {stacked ? (
        <PageIntro
          subtitle={servers.length ? t('已添加 {n} 台', { n: servers.length }) : t('先添加一台网关')}
          actions={
            <Button size='touch-sm' variant='default' type='button' className='m-btn m-sm' onClick={startAdd} disabled={editing === 'new'}>
              <Icon name='plus' />
              {t('添加')}
            </Button>
          }
        />
      ) : (
        <PageHead
          title={t('服务器')}
          subtitle={servers.length ? t('已添加 {n} 台', { n: servers.length }) : t('先添加一台网关')}
          actions={
            <Button size='touch-sm' variant='default' type='button' className='m-btn m-sm' onClick={startAdd} disabled={editing === 'new'}>
              <Icon name='plus' />
              {t('添加')}
            </Button>
          }
        />
      )}

      {error ? <ErrorNote text={error} /> : null}

      {/* ：add and use */}
      {editing ? (
        <div className='m-section' style={{ marginTop: 0 }}>
          <Card className='m-card m-card-pad'>
            <div style={{ fontSize: 13, fontWeight: 600 }}>
              {editing === 'new' ? t('添加服务器') : t('编辑服务器')}
            </div>
            <Label style={{ display: 'block', marginTop: 12 }}>
              <span style={{ fontSize: 12, color: 'var(--m-text-3)' }}>{t('地址')}</span>
              <Input
                value={draftBase}
                onChange={event => setDraftBase(event.target.value)}
                placeholder='192.168.1.23:3065 或 https://gw.example.com'
                inputMode='url'
                autoCapitalize='off'
                autoCorrect='off'
                spellCheck={false}
                style={{
                  width: '100%',
                  boxSizing: 'border-box',
                  marginTop: 5,
                  height: 44,
                  padding: '0 12px',
                  borderRadius: 11,
                  border: '1px solid var(--m-line-strong)',
                  background: 'var(--m-surface-2)',
                  color: 'var(--m-text)',
                  fontSize: 16,
                  fontFamily: 'var(--m-mono)',
                }}
              />
            </Label>
            <div style={{ fontSize: 11.5, color: 'var(--m-text-3)', marginTop: 6, lineHeight: 1.6 }}>
              {t('填「IP 或域名:端口」。协议可写 http:// 或 https://，不写默认 http；http 不写端口默认 3065，https 默认 443。地址里的 /m、/v1 会自动去掉。')}
            </div>
            <Label style={{ display: 'block', marginTop: 12 }}>
              <span style={{ fontSize: 12, color: 'var(--m-text-3)' }}>{t('备注名（可选）')}</span>
              <Input
                value={draftName}
                onChange={event => setDraftName(event.target.value)}
                placeholder={t('例如：家里、公司')}
                style={{
                  width: '100%',
                  boxSizing: 'border-box',
                  marginTop: 5,
                  height: 44,
                  padding: '0 12px',
                  borderRadius: 11,
                  border: '1px solid var(--m-line-strong)',
                  background: 'var(--m-surface-2)',
                  color: 'var(--m-text)',
                  fontSize: 16,
                }}
              />
            </Label>
            <div className='m-btn-row' style={{ marginTop: 14 }}>
              <Button size='touch' variant='outline' type='button' className='m-btn m-ghost w-full' onClick={cancel} disabled={busy}>
                {t('取消')}
              </Button>
              <Button size='touch' variant='default' type='button' className='m-btn w-full' onClick={() => void save()} disabled={busy || !draftBase.trim()}>
                {busy ? t('正在连接…') : t('连接并保存')}
              </Button>
            </div>
          </Card>
        </div>
      ) : null}

      <div className='m-section'>
        <SectionHead title={t('已添加的服务器')} />
        {servers.length === 0 ? (
          <Card className='m-card m-card-pad' style={{ fontSize: 12.5, color: 'var(--m-text-2)', lineHeight: 1.7 }}>
            {t('还没有添加服务器。点右上角「添加」，填入网关所在机器的 IP 或域名与端口。')}
          </Card>
        ) : (
          <Card className='m-card'>
            {servers.map(entry => {
              const current = server?.id === entry.id
              return (
                <div key={entry.id} className='m-row'>
                  <span
                    className='m-logo'
                    style={
                      current
                        ? { background: 'var(--m-primary-soft)', borderColor: 'transparent', color: 'var(--m-primary)' }
                        : undefined
                    }
                  >
                    <Icon name='server' />
                  </span>
                  <Button size='default' variant='ghost'
                    type='button'
                    className='m-grow'
                    onClick={() => {
                      store.switchServer(entry.id)
                      void store.refresh(true).then(() => {
                        // from ： this ， by new new 
                        if (store.get().stack.length > 0) store.back()
                      })
                    }}
                    style={{ border: 0, background: 'none', textAlign: 'left', padding: 0 }}
                  >
                    <div className='m-t1' style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      {entry.name || baseLabel(entry.base)}
                      {current ? (
                        <Badge variant='brand' shape='tag' className='m-badge m-badge-brand' style={{ height: 18, fontSize: 10.5 }}>
                          {t('当前')}
                        </Badge>
                      ) : null}
                    </div>
                    <div className='m-t2 m-mono' style={{ fontSize: 11.5 }}>
                      {baseLabel(entry.base)}
                    </div>
                  </Button>
                  <Button size='icon-touch' variant='outline'
                    type='button'
                    className='m-icon-btn'
                    aria-label={t('编辑')}
                    onClick={() => startEdit(entry.id)}
                    style={{ width: 30, height: 30, boxShadow: 'none' }}
                  >
                    <Icon name='edit' />
                  </Button>
                  <Button size='icon-touch' variant='outline'
                    type='button'
                    className='m-icon-btn'
                    aria-label={t('删除')}
                    onClick={() => {
                      store.deleteServer(entry.id)
                      store.toast(t('已删除'))
                      void store.refresh(true)
                    }}
                    style={{ width: 30, height: 30, boxShadow: 'none', color: 'var(--m-danger)' }}
                  >
                    <Icon name='trash' />
                  </Button>
                </div>
              )
            })}
          </Card>
        )}
      </div>

      <div className='m-section' style={{ paddingBottom: 8 }}>
        <Card className='m-card m-card-pad' style={{ fontSize: 11.5, color: 'var(--m-text-3)', lineHeight: 1.7 }}>
          {t('每台服务器的登录状态各自独立保存，切换后不用重新登录。地址改掉后需要重新登录。')}
        </Card>
      </div>
    </div>
  )
}
