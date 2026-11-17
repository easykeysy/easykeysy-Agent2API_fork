import { t } from '../i18n'

export type SyncStatus = {
  lastSyncAt?: number
  lastAction?: string
  lastSnapshotId?: string
  lastLocalHash?: string
  lastKeyFingerprint?: string
  lastError?: string
  lastErrorSource?: string
  retryAt?: number
  lastUploadAt?: number
}

export type CloudSyncView = {
  provider: string
  webdav: { baseUrl: string; username: string; hasPassword: boolean; userAgent: string }
  s3: {
    region: string
    bucket: string
    accessKeyId: string
    endpoint: string
    hasSecretKey: boolean
    userAgent: string
  }
  hasSyncPassword: boolean
  enabled: boolean
  autoSync: boolean
  autoSyncIntervalMinutes: number
  remoteRoot: string
  profile: string
  status: SyncStatus
  remotePath: string
  keyFingerprint: string | null
}

export type RemoteInfo = {
  empty?: boolean
  remotePath?: string
  format?: string
  version?: number
  appVersion?: string
  schemaVersion?: number
  deviceName?: string
  createdAt?: string
  snapshotId?: string
  artifacts?: string[]
  compatible?: boolean
  incompatibleReason?: string
  keyFingerprint?: string | null
  localKeyFingerprint?: string | null
  keyRequired?: boolean
  hasKeyArtifact?: boolean
}

export type Form = {
  provider: string
  webdavBaseUrl: string
  webdavUsername: string
  webdavPassword: string
  webdavUserAgent: string
  s3Region: string
  s3Bucket: string
  s3AccessKeyId: string
  s3SecretAccessKey: string
  s3Endpoint: string
  s3UserAgent: string
  syncPassword: string
  enabled: boolean
  autoSync: boolean
  autoSyncIntervalMinutes: number
  remoteRoot: string
  profile: string
}

export const PROVIDER_WEBDAV = 'webdav'
export const PROVIDER_S3 = 's3'

export const WEBDAV_PRESETS = [
  {
    id: 'jianguoyun',
    label: t('Jianguoyun (Nutstore)'),
    baseUrl: 'https://dav.jianguoyun.com/dav/',
    hint: t('Jianguoyun requires an app-specific password (generate it under Account info → Security), and the URL must point to a directory under /dav/.'),
  },
  {
    id: 'nextcloud',
    label: t('Nextcloud'),
    baseUrl: 'https://your-server/remote.php/dav/files/USERNAME/',
    hint: t('Replace your-server and USERNAME in the URL with your own; the directory is created automatically if missing.'),
  },
  {
    id: 'synology',
    label: t('Synology NAS'),
    baseUrl: 'http://your-nas-ip:5005/',
    hint: t('On Synology, enable the service under Control Panel → Application Portal → WebDAV; the default ports are 5005 (HTTP) / 5006 (HTTPS).'),
  },
  { id: 'custom', label: t('Custom'), baseUrl: '', hint: '' },
] as const

export const S3_PRESETS = [
  {
    id: 'aws',
    label: 'Amazon S3',
    regionPlaceholder: 'us-east-1',
    hint: t('Leave Endpoint empty to use the official AWS address; Region must match the bucket\'s region.'),
  },
  {
    id: 'r2',
    label: 'Cloudflare R2',
    regionPlaceholder: 'auto',
    hint: t('Set Endpoint to https://<account ID>.r2.cloudflarestorage.com and Region to auto.'),
  },
  {
    id: 'minio',
    label: 'MinIO',
    regionPlaceholder: 'us-east-1',
    hint: t('Endpoint looks like http://192.168.1.10:9000 (self-hosted usually uses http); Region is typically us-east-1.'),
  },
  {
    id: 'oss',
    label: t('Alibaba Cloud OSS'),
    regionPlaceholder: 'cn-hangzhou',
    hint: t('Set Endpoint to https://oss-cn-hangzhou.aliyuncs.com and Region to cn-hangzhou (change to your actual region).'),
  },
  {
    id: 'cos',
    label: t('Tencent Cloud COS'),
    regionPlaceholder: 'ap-guangzhou',
    hint: t('Set Endpoint to https://cos.ap-guangzhou.myqcloud.com and Region to ap-guangzhou (change to your actual region).'),
  },
  {
    id: 'obs',
    label: t('Huawei Cloud OBS'),
    regionPlaceholder: 'cn-north-4',
    hint: t('Set Endpoint to https://obs.cn-north-4.myhuaweicloud.com and Region to cn-north-4 (change to your actual region).'),
  },
  { id: 'custom', label: t('Custom'), regionPlaceholder: 'us-east-1', hint: '' },
] as const

export function formatTime(ms?: number): string {
  if (!ms) return ''
  try { return new Date(ms).toLocaleString() }
  catch { return String(ms) }
}

export function formatDateText(raw?: string): string {
  if (!raw) return '—'
  const parsed = new Date(raw)
  return Number.isNaN(parsed.getTime()) ? raw : parsed.toLocaleString()
}

export function emptyForm(): Form {
  return {
    provider: PROVIDER_WEBDAV,
    webdavBaseUrl: '', webdavUsername: '', webdavPassword: '', webdavUserAgent: '',
    s3Region: '', s3Bucket: '', s3AccessKeyId: '', s3SecretAccessKey: '', s3Endpoint: '', s3UserAgent: '',
    syncPassword: '', enabled: false, autoSync: false, autoSyncIntervalMinutes: 5,
    remoteRoot: 'agent2api-sync', profile: 'default',
  }
}

export function formFromView(view: CloudSyncView): Form {
  return {
    provider: view.provider === PROVIDER_S3 ? PROVIDER_S3 : PROVIDER_WEBDAV,
    webdavBaseUrl: view.webdav.baseUrl,
    webdavUsername: view.webdav.username,
    // 密码不回显：留空 = 沿用后端已保存的那份。
    webdavPassword: '',
    webdavUserAgent: view.webdav.userAgent,
    s3Region: view.s3.region,
    s3Bucket: view.s3.bucket,
    s3AccessKeyId: view.s3.accessKeyId,
    s3Endpoint: view.s3.endpoint,
    s3SecretAccessKey: '',
    s3UserAgent: view.s3.userAgent,
    syncPassword: '',
    enabled: view.enabled,
    autoSync: view.autoSync,
    autoSyncIntervalMinutes: view.autoSyncIntervalMinutes || 5,
    remoteRoot: view.remoteRoot,
    profile: view.profile,
  }
}

export function payloadOf(form: Form): Record<string, unknown> {
  return {
    provider: form.provider,
    webdavBaseUrl: form.webdavBaseUrl,
    webdavUsername: form.webdavUsername,
    webdavPassword: form.webdavPassword,
    webdavUserAgent: form.webdavUserAgent,
    s3Region: form.s3Region,
    s3Bucket: form.s3Bucket,
    s3AccessKeyId: form.s3AccessKeyId,
    s3SecretAccessKey: form.s3SecretAccessKey,
    s3Endpoint: form.s3Endpoint,
    s3UserAgent: form.s3UserAgent,
    syncPassword: form.syncPassword,
    enabled: form.enabled,
    autoSync: form.autoSync,
    autoSyncIntervalMinutes: form.autoSyncIntervalMinutes,
    remoteRoot: form.remoteRoot,
    profile: form.profile,
  }
}
