// Historical pre-clean-room navbar health aggregation. Not part of the active client graph.
import type { MailSyncStatus } from '../../domains/communications/types/communications'
import type { ProviderAccount } from '../../domains/settings/types/settings'
import type { TelegramAccount } from '../../integrations/telegram/types/telegram'
import type { WhatsappAccountSummary } from '../../integrations/whatsapp/types/whatsapp'
import type { RealtimeStatusSnapshot } from '../../shared/stores/realtimeStatus'

export type AppLayoutNavbarHealthStatus = 'healthy' | 'degraded' | 'unhealthy'

export type AppLayoutNavbarHealthCheck = {
  id: string
  label: string
  status: AppLayoutNavbarHealthStatus
  detail: string
}

export type AppLayoutNavbarAccountNavigationHealth = {
  mail: readonly ProviderAccount[]
  telegram: readonly TelegramAccount[]
  whatsapp: readonly WhatsappAccountSummary[]
}

type BackendReadinessCheck = {
  status?: string
  message?: string
}

export type BackendReadinessResponse = {
  status?: string
  service?: string
  checks?: Record<string, BackendReadinessCheck | undefined>
}

const STALE_SYNC_AFTER_MS = 2 * 60 * 1000
export const DEFAULT_CONSECUTIVE_PROVIDER_FAILURES_BEFORE_DEGRADED = 3

export function frontendRuntimeHealthChecks(
  realtime: RealtimeStatusSnapshot | null
): readonly AppLayoutNavbarHealthCheck[] {
  return [
    {
      id: 'frontend-runtime',
      label: 'Frontend runtime',
      status: 'healthy',
      detail: 'UI health polling and reconnect loop active',
    },
    realtimeHealthCheck(realtime),
  ]
}

export function backendReadinessHealthChecks(
  readiness: BackendReadinessResponse
): readonly AppLayoutNavbarHealthCheck[] {
  const checks: AppLayoutNavbarHealthCheck[] = [
    {
      id: 'backend-api',
      label: readiness.service
        ? `Backend API: ${readiness.service}`
        : 'Backend API',
      status: readinessHealthStatus(readiness.status),
      detail: readiness.status ?? 'Backend readiness status is missing',
    },
  ]

  for (const [id, check] of Object.entries(readiness.checks ?? {})) {
    if (!check) continue

    checks.push({
      id: `backend-${id}`,
      label: healthCheckLabel(id),
      status: readinessHealthStatus(check.status),
      detail:
        check.message ??
        check.status ??
        'Backend health check returned no detail',
    })
  }

  return checks
}

export function backendErrorHealthCheck(error: unknown): AppLayoutNavbarHealthCheck {
  return {
    id: 'backend-api',
    label: 'Backend API',
    status: 'unhealthy',
    detail: errorMessage(error, 'Backend readiness request failed'),
  }
}

export function mailSyncStatusHealthChecks(
  statuses: readonly MailSyncStatus[],
  consecutiveFailuresBeforeDegraded = DEFAULT_CONSECUTIVE_PROVIDER_FAILURES_BEFORE_DEGRADED
): readonly AppLayoutNavbarHealthCheck[] {
  if (statuses.length === 0) {
    return [
      {
        id: 'mail-sync-none',
        label: 'Mail sync',
        status: 'degraded',
        detail: 'No mail account sync status returned',
      },
    ]
  }

  return statuses.map((status) => {
    const stale = mailSyncStatusIsStale(status)
    return {
      id: `mail-sync-${status.account_id}`,
      label: `Mail sync: ${status.account_id}`,
      status: mailSyncHealthStatus(status, stale, consecutiveFailuresBeforeDegraded),
      detail: mailSyncDetail(status, stale, consecutiveFailuresBeforeDegraded),
    }
  })
}

export function mailSyncErrorHealthCheck(error: unknown): AppLayoutNavbarHealthCheck {
  return {
    id: 'mail-sync-api',
    label: 'Mail sync API',
    status: 'unhealthy',
    detail: errorMessage(error, 'Mail sync status request failed'),
  }
}

export function integrationAccountHealthChecks(
  accounts: AppLayoutNavbarAccountNavigationHealth,
  accountNavigationError: string
): readonly AppLayoutNavbarHealthCheck[] {
  if (accountNavigationError.trim()) {
    return [
      {
        id: 'provider-accounts',
        label: 'Provider accounts',
        status: 'unhealthy',
        detail: accountNavigationError,
      },
    ]
  }

  return [
    mailAccountsHealthCheck(accounts.mail),
    telegramAccountsHealthCheck(accounts.telegram),
    whatsappAccountsHealthCheck(accounts.whatsapp),
  ]
}

export function healthChecksNeedRecovery(
  checks: readonly AppLayoutNavbarHealthCheck[]
): boolean {
  return checks.some((check) => check.status !== 'healthy')
}

function realtimeHealthCheck(
  realtime: RealtimeStatusSnapshot | null
): AppLayoutNavbarHealthCheck {
  if (!realtime) {
    return {
      id: 'frontend-realtime',
      label: 'Frontend realtime',
      status: 'degraded',
      detail: 'Realtime status store is not available yet',
    }
  }

  if (realtime.state === 'connected') {
    return {
      id: 'frontend-realtime',
      label: 'Frontend realtime',
      status: 'healthy',
      detail: `${realtime.transport} live${lastEventDetail(realtime)}`,
    }
  }

  if (
    realtime.state === 'connecting' ||
    realtime.state === 'reconnecting'
  ) {
    return {
      id: 'frontend-realtime',
      label: 'Frontend realtime',
      status: 'degraded',
      detail: realtime.error
        ? `${realtime.state}: ${realtime.error}`
        : `${realtime.transport} ${realtime.state}`,
    }
  }

  return {
    id: 'frontend-realtime',
    label: 'Frontend realtime',
    status: realtime.state === 'idle' ? 'degraded' : 'unhealthy',
    detail: realtime.error ?? `${realtime.transport} ${realtime.state}`,
  }
}

function mailAccountsHealthCheck(
  accounts: readonly ProviderAccount[]
): AppLayoutNavbarHealthCheck {
  const unauthenticated = accounts.filter(
    (account) => account.is_authenticated === false
  )
  return {
    id: 'mail-accounts',
    label: 'Mail accounts',
    status: unauthenticated.length > 0 ? 'unhealthy' : 'healthy',
    detail:
      unauthenticated.length > 0
        ? `${unauthenticated.length}/${accounts.length} account(s) need authentication`
        : `${accounts.length} active account(s) configured`,
  }
}

function telegramAccountsHealthCheck(
  accounts: readonly TelegramAccount[]
): AppLayoutNavbarHealthCheck {
  const loggedOut = accounts.filter(
    (account) => account.lifecycle_state === 'logged_out'
  )
  return {
    id: 'telegram-accounts',
    label: 'Telegram accounts',
    status: loggedOut.length > 0 ? 'unhealthy' : 'healthy',
    detail:
      loggedOut.length > 0
        ? `${loggedOut.length}/${accounts.length} account(s) logged out`
        : `${accounts.length} active account(s) configured`,
  }
}

function whatsappAccountsHealthCheck(
  accounts: readonly WhatsappAccountSummary[]
): AppLayoutNavbarHealthCheck {
  const blocked = accounts.filter((account) =>
    lifecycleLooksBlocked(account.lifecycle_state)
  )
  return {
    id: 'whatsapp-accounts',
    label: 'WhatsApp accounts',
    status: blocked.length > 0 ? 'unhealthy' : 'healthy',
    detail:
      blocked.length > 0
        ? `${blocked.length}/${accounts.length} account(s) need attention`
        : `${accounts.length} active account(s) configured`,
  }
}

function mailSyncHealthStatus(
  status: MailSyncStatus,
  stale: boolean,
  consecutiveFailuresBeforeDegraded: number
): AppLayoutNavbarHealthStatus {
  if (stale) return 'unhealthy'
  if (
    status.status === 'failed' &&
    status.consecutive_failures >= consecutiveFailuresBeforeDegraded
  ) {
    return 'degraded'
  }
  if (status.status === 'recoverable_full_resync_needed') return 'degraded'
  return 'healthy'
}

function mailSyncStatusIsStale(status: MailSyncStatus): boolean {
  if (
    status.status !== 'running' &&
    status.status !== 'queued' &&
    status.status !== 'recoverable_full_resync_needed'
  ) {
    return false
  }

  const lastMovement = Date.parse(
    status.last_updated_at ?? status.last_started_at ?? ''
  )
  if (!Number.isFinite(lastMovement)) return true
  return Date.now() - lastMovement > STALE_SYNC_AFTER_MS
}

function mailSyncDetail(
  status: MailSyncStatus,
  stale: boolean,
  consecutiveFailuresBeforeDegraded: number
): string {
  const fragments = [
    `${status.status}/${status.phase}`,
    `processed ${status.processed_messages}`,
  ]
  if (status.current_batch_size > 0) {
    fragments.push(`batch ${status.current_batch_size}`)
  }
  if (status.status === 'failed') {
    fragments.push(
      `failure ${status.consecutive_failures}/${consecutiveFailuresBeforeDegraded}`
    )
  }
  if (stale) fragments.push('no recent progress')
  if (status.last_error_message) fragments.push(status.last_error_message)
  return fragments.join(' · ')
}

function readinessHealthStatus(
  status: string | undefined
): AppLayoutNavbarHealthStatus {
  if (status === 'ok' || status === 'healthy' || status === 'ready')
    return 'healthy'
  if (status === 'degraded') return 'degraded'
  return 'unhealthy'
}

function healthCheckLabel(id: string): string {
  return id
    .split(/[_-]+/)
    .filter(Boolean)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(' ')
}

function lifecycleLooksBlocked(value: string | null): boolean {
  if (!value) return false
  const normalized = value.toLowerCase()
  return (
    normalized.includes('logged_out') ||
    normalized.includes('removed') ||
    normalized.includes('revoked') ||
    normalized.includes('blocked') ||
    normalized.includes('error')
  )
}

function lastEventDetail(realtime: RealtimeStatusSnapshot): string {
  if (!realtime.lastEventAt) return ''
  return ` · last event ${realtime.lastEventAt}`
}

function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) return error.message
  if (
    error &&
    typeof error === 'object' &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message
  }
  return fallback
}
