import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from '../../../shared/mailSync/types'
import {
  MAX_MAIL_BATCH_SIZE,
  MAX_MAIL_SYNC_WINDOWS,
  MAX_MAIL_POLL_INTERVAL_SECONDS,
  MIN_MAIL_POLL_INTERVAL_SECONDS,
} from '../../../shared/mailSync/types'
import { normalizeMailSyncSettingsValues } from './mailSyncNormalization'

type MailSyncTranslator = (key: string) => string

interface CommunicationsMailSyncDependencies {
  t: MailSyncTranslator
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  updateSyncSettings: (request: {
    accountId: string
    settings: MailSyncSettingsUpdate
  }) => Promise<unknown>
}

export async function saveMailSyncSettings(
  accountId: string | null,
  settings: MailSyncSettings | null,
  batchSizeDraft: string,
  pollIntervalDraft: string,
  windowsDraft: string,
  dependencies: CommunicationsMailSyncDependencies
): Promise<void> {
  if (!accountId || !settings) return

  const batchSize = Number.parseInt(batchSizeDraft, 10)
  const pollIntervalSeconds = Number.parseInt(pollIntervalDraft, 10)
  const syncWindows = Number.parseInt(windowsDraft, 10)
  if (
    !Number.isInteger(batchSize) ||
    batchSize < 1 ||
    batchSize > MAX_MAIL_BATCH_SIZE ||
    !Number.isInteger(pollIntervalSeconds) ||
    pollIntervalSeconds < MIN_MAIL_POLL_INTERVAL_SECONDS ||
    pollIntervalSeconds > MAX_MAIL_POLL_INTERVAL_SECONDS ||
    !Number.isInteger(syncWindows) ||
    syncWindows < 1 ||
    syncWindows > MAX_MAIL_SYNC_WINDOWS
  ) {
    dependencies.setError(dependencies.t('Mail sync settings must be within allowed ranges.'))
    return
  }

  await executeMailSyncUpdate(
    accountId,
    {
      sync_enabled: settings.sync_enabled,
      batch_size: batchSize,
      windows: syncWindows,
      poll_interval_seconds: pollIntervalSeconds,
      failure_threshold: settings.failure_threshold ?? 3,
    },
    dependencies,
    'Mail sync settings saved',
    'Mail sync settings update failed'
  )
}

export async function toggleMailSync(
  accountId: string | null,
  settings: MailSyncSettings | null,
  enabled: boolean,
  dependencies: CommunicationsMailSyncDependencies
): Promise<void> {
  if (!accountId || !settings) return

  const normalized = normalizeMailSyncSettingsValues(settings)
  await executeMailSyncUpdate(
    accountId,
    {
      ...normalized,
      sync_enabled: enabled,
    },
    dependencies,
    enabled ? 'Mail sync enabled' : 'Mail sync paused',
    'Mail sync settings update failed'
  )
}

async function executeMailSyncUpdate(
  accountId: string,
  update: MailSyncSettingsUpdate,
  dependencies: CommunicationsMailSyncDependencies,
  successMessage: string,
  fallbackError: string
): Promise<void> {
  dependencies.clearMessages()
  try {
    await dependencies.updateSyncSettings({ accountId, settings: update })
    dependencies.setActionMessage(dependencies.t(successMessage))
  } catch (error) {
    dependencies.setError(error instanceof Error ? error.message : dependencies.t(fallbackError))
  }
}
