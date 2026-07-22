import type { MailSyncStatus } from '../../types/communications'

type Translate = (key: string) => string

export const MAIL_SYNC_STALE_AFTER_MS = 2 * 60 * 1000

export function mailSyncIsRunning(status: string | undefined): boolean {
  return status === 'queued' || status === 'running' || status === 'recoverable_full_resync_needed'
}

export function mailSyncFailureKey(status: MailSyncStatus | null | undefined): string | null {
  if (!status || status.status !== 'failed') return null
  return [
    status.account_id,
    status.last_started_at ?? status.last_updated_at ?? 'unknown-start',
    status.last_error_code ?? 'unknown-code',
    status.last_error_message ?? 'unknown-error',
  ].join(':')
}

export function mailSyncTimestampMs(value: string | null | undefined): number | null {
  if (!value) return null
  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? parsed : null
}

export function mailSyncProgressPercent(status: MailSyncStatus | null | undefined): number | null {
  if (!status || status.progress_mode !== 'determinate') return null
  if (typeof status.progress_percent !== 'number') return null
  return Math.min(100, Math.max(0, status.progress_percent))
}

export function mailSyncProgressClass(options: {
  failed: boolean
  failureKey: string | null
  exitingFailureKey: string | null
  stale: boolean
  running: boolean
  indeterminate: boolean
}): string[] {
  return [
    'mail-sync-progress',
    options.failed && 'mail-sync-progress--failed',
    options.failureKey !== null && options.exitingFailureKey === options.failureKey && 'mail-sync-progress--exiting',
    options.stale && 'mail-sync-progress--warning',
    options.running && !options.stale && 'mail-sync-progress--active',
    options.indeterminate && 'mail-sync-progress--indeterminate',
  ].filter((value): value is string => Boolean(value))
}

export function mailSyncIcon(failed: boolean, stale: boolean): string {
  if (failed) return 'tabler:alert-circle'
  if (stale) return 'tabler:alert-triangle'
  return 'tabler:loader-2'
}

export function mailSyncTitle(failed: boolean, stale: boolean, t: Translate): string {
  if (failed) return t('Mail sync failed')
  if (stale) return t('Mail sync needs attention')
  return t('Loading mail')
}

export function mailSyncBadgeLabel(
  failed: boolean,
  stale: boolean,
  progressPercent: number | null,
  t: Translate
): string {
  if (failed) return t('failed')
  if (stale) return t('stalled')
  if (progressPercent !== null) return `${progressPercent}%`
  return t('running')
}

export function mailSyncPhaseLabel(phase: string, t: Translate): string {
  switch (phase) {
    case 'listing': return t('listing mailboxes')
    case 'fetching':
    case 'fetch': return t('fetching messages')
    case 'projecting':
    case 'project': return t('projecting messages')
    case 'personas_graph': return t('updating graph')
    case 'completed': return t('completed')
    case 'failed': return t('failed')
    default: return phase || t('idle')
  }
}

export function mailSyncIsStale(
  status: MailSyncStatus | null | undefined,
  nowMs: number,
  staleAfterMs = MAIL_SYNC_STALE_AFTER_MS
): boolean {
  if (!mailSyncIsRunning(status?.status)) return false
  const lastMovementMs = mailSyncTimestampMs(status?.last_updated_at ?? status?.last_started_at)
  return lastMovementMs !== null && nowMs - lastMovementMs > staleAfterMs
}

export function mailSyncDetail(
  status: MailSyncStatus | null | undefined,
  failed: boolean,
  t: Translate
): string {
  if (!status) return ''
  const parts = [`${t('processed')} ${status.processed_messages}`]
  if (typeof status.estimated_total_messages === 'number') parts.push(`${t('of')} ${status.estimated_total_messages}`)
  if (status.current_batch_size > 0) parts.push(`${t('batch')} ${status.current_batch_size}`)
  if (failed && status.last_error_message) parts.push(status.last_error_message)
  if (status.last_fetched_messages > 0 || status.last_projected_messages > 0) {
    parts.push(`${t('fetched')} ${status.last_fetched_messages}`)
    parts.push(`${t('projected')} ${status.last_projected_messages}`)
  }
  return parts.join(' · ')
}

export function mailSyncFailureNotificationBody(status: MailSyncStatus, t: Translate): string {
  const parts = [mailSyncPhaseLabel(status.phase, t), `${t('processed')} ${status.processed_messages}`]
  if (status.last_error_message) parts.push(status.last_error_message)
  return parts.join(' · ')
}

export function formatMailSyncAge(ageMs: number, t: Translate): string {
  const seconds = Math.floor(ageMs / 1000)
  if (seconds < 60) return t('just now')
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes} ${t('min ago')}`
  return `${Math.floor(minutes / 60)} ${t('h ago')}`
}
