import { PageHead, SectionHead, IconButton } from '../components/ui'
import { Icon, type IconName } from '../components/Icon'
import { store, useStore } from '../state/store'
import { apiBase, baseLabel } from '../lib/servers'
import { isRateLimited } from '../lib/format'
import { usePullToRefresh, PullIndicator } from '../components/PullToRefresh'
import { t } from '../i18n'
import { Card, Button, SegmentedControl } from '@ui'

/**
 * 更多。
 *
 * 桌面端 13 个一级菜单在这里收成三组宫格。**宫格不是垃圾桶**：按
 * 「转发链路 / 诊断 / 系统」分组，每组内部按使用频率排；每个入口带实时
 * 角标（待签数、限流数），不用点进去才知道有事。
 */

type Entry = { id: string; label: string; icon: IconName; badge?: string }

export function MorePage() {
  const { session } = useStore()
  const accounts = session?.accounts?.accounts || []
  const limited = accounts.filter(item => item.enabled && isRateLimited(item)).length

  // 本页没有自己的列表请求：下拉只需重拉会话（角标与网关状态都来自它）
  const pull = usePullToRefresh({ onRefresh: () => store.refresh(true) })

  const groups: Array<{ title: string; entries: Entry[] }> = [
    {
      title: t('Forwarding'),
      entries: [
        { id: 'checkin', label: t('Check-in center'), icon: 'checkin' },
        { id: 'models', label: t('Model management'), icon: 'model' },
        { id: 'agent', label: t('Agent Configuration'), icon: 'agent' },
        { id: 'tools', label: t('MCP & Skills'), icon: 'tools' },
        { id: 'proxies', label: t('Network proxy'), icon: 'proxy' },
        { id: 'keys', label: t('Gateway key'), icon: 'key' },
        { id: 'docs', label: t('The API documentation'), icon: 'doc' },
      ],
    },
    {
      title: t('Diagnostics'),
      entries: [
        { id: 'requests', label: t('Request logs'), icon: 'request', badge: limited ? String(limited) : undefined },
        { id: 'logs', label: t('The run log'), icon: 'log' },
        { id: 'tasks', label: t('Automatic maintenance'), icon: 'task' },
      ],
    },
    {
      title: t('System'),
      entries: [
        { id: 'plugins', label: t('Gateway plugins'), icon: 'plugin', badge: session?.accounts?.hiddenAccounts?.total ? String(session.accounts.hiddenAccounts.total) : undefined },
        { id: 'settings', label: t('Settings'), icon: 'settings' },
        { id: 'report', label: t('Reports'), icon: 'chart' },
      ],
    },
  ]

  return (
    <div className='m-page' ref={pull.bindRef}>
      <PullIndicator pull={pull.pull} refreshing={pull.refreshing} />
      <PageHead
        title={t('More')}
        subtitle={t('Forwarding · Diagnostics · System')}
        actions={<IconButton name='settings' label={t('Settings')} onClick={() => store.push('settings')} />}
      />

      <div className='m-section' style={{ marginTop: 0 }}>
        <Card className='m-card' style={{ padding: '10px 0' }}>
          {groups.map((group, index) => (
            <div key={group.title} style={{ padding: index ? '8px 0 4px' : '0 0 4px' }}>
              <div style={{ padding: '0 20px 4px', fontSize: 11.5, fontWeight: 600, color: 'var(--m-text-3)', letterSpacing: '.02em' }}>
                {group.title}
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, minmax(0, 1fr))', gap: 4, padding: '4px 10px 10px' }}>
                {group.entries.map(entry => (
                  <Button size='default' variant='ghost' className='h-auto min-w-0 whitespace-normal'
                    key={entry.id}
                    type='button'
                    onClick={() => store.push(entry.id)}
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: 7,
                      padding: '12px 2px',
                      borderRadius: 13,
                      border: 0,
                      background: 'none',
                      fontSize: 11.5,
                      color: 'var(--m-text-2)',
                      position: 'relative',
                    }}
                  >
                    <span
                      className='shrink-0'
                      style={{
                        width: 44,
                        height: 44,
                        borderRadius: 14,
                        display: 'grid',
                        placeItems: 'center',
                        background: 'var(--m-surface)',
                        border: '1px solid var(--m-line)',
                        color: 'var(--m-primary)',
                        boxShadow: 'var(--m-shadow-1)',
                      }}
                    >
                      <Icon name={entry.icon} style={{ width: 22, height: 22 }} />
                    </span>
                    {entry.label}
                    {entry.badge ? (
                      <span
                        style={{
                          position: 'absolute',
                          top: 8,
                          right: '50%',
                          transform: 'translateX(26px)',
                          minWidth: 16,
                          height: 16,
                          padding: '0 4px',
                          borderRadius: 8,
                          background: 'var(--m-danger)',
                          color: '#fff',
                          fontSize: 10,
                          fontWeight: 600,
                          display: 'grid',
                          placeItems: 'center',
                        }}
                      >
                        {entry.badge}
                      </span>
                    ) : null}
                  </Button>
                ))}
              </div>
            </div>
          ))}
        </Card>
      </div>

      <div className='m-section'>
        <SectionHead title={t('Appearance')} />
        <Card className='m-card'>
          <div className='m-row' style={{ flexWrap: 'wrap', gap: 8 }}>
            <span className='m-logo'>
              <Icon name='moon' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Theme')}</div>
              <div className='m-t2'>{t('System · Light · Dark')}</div>
            </div>
            <ThemeSwitch />
          </div>
        </Card>
      </div>

      <div className='m-section'>
        <Card className='m-card'>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => store.push('settings')}>
            <span className='m-logo'>
              <Icon name='server' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('Gateway status')}</div>
              <div className='m-t2 m-mono' style={{ fontSize: 11.5 }}>
                {baseLabel(apiBase())}
              </div>
            </div>
            <span className='m-chev'>
              <Icon name='chev' />
            </span>
          </Button>
          <Button size='default' variant='ghost' type='button' className='m-row h-auto min-h-[52px] w-full justify-start rounded-none border-0 px-3.5 py-3 text-left' onClick={() => store.push('docs')}>
            <span className='m-logo'>
              <Icon name='doc' />
            </span>
            <div className='m-grow'>
              <div className='m-t1'>{t('API address')}</div>
              <div className='m-t2'>{t('The Base URL to copy for your client')}</div>
            </div>
            <span className='m-chev'>
              <Icon name='chev' />
            </span>
          </Button>
        </Card>
      </div>

      <div className='m-section' style={{ paddingBottom: 8 }}>
        <Card className='m-card m-card-pad' style={{ fontSize: 11.5, color: 'var(--m-text-3)', lineHeight: 1.7 }}>
          {t('The mobile app manages accounts, check-in and plugins on the connected gateway and supports sign-in through providers’ declared dynamic methods. Other existing add-account entry points, credential editing and client configuration changes still require the desktop panel.')}
        </Card>
      </div>
    </div>
  )
}

/** 主题三档：跟随系统 / 浅色 / 深色 */
export function ThemeSwitch() {
  const { theme } = useStore()
  const options: Array<{ value: 'system' | 'light' | 'dark'; label: string }> = [
    { value: 'system', label: t('System') },
    { value: 'light', label: t('Light') },
    { value: 'dark', label: t('Dark') },
  ]
  return <SegmentedControl options={options} value={theme} aria-label={t('Theme')}
    onValueChange={value => store.setTheme(value as 'system' | 'light' | 'dark')} />
}
