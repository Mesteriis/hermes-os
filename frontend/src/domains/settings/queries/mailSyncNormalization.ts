import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from '../../../shared/mailSync/types'
import {
  DEFAULT_MAIL_BATCH_SIZE,
  DEFAULT_MAIL_SYNC_WINDOWS,
  DEFAULT_MAIL_POLL_INTERVAL_SECONDS,
  MAX_MAIL_POLL_INTERVAL_SECONDS,
  MAX_MAIL_BATCH_SIZE,
  MAX_MAIL_SYNC_WINDOWS,
  MIN_MAIL_POLL_INTERVAL_SECONDS,
} from '../../../shared/mailSync/types'

function clampWindows(value: number): number {
  if (value < 1) {
    return DEFAULT_MAIL_SYNC_WINDOWS
  }
  if (value > MAX_MAIL_SYNC_WINDOWS) {
    return MAX_MAIL_SYNC_WINDOWS
  }
  return value
}

function clampBatchSize(value: number): number {
  return Math.min(Math.max(value, 1), MAX_MAIL_BATCH_SIZE)
}

function clampPollIntervalSeconds(value: number): number {
  return Math.min(Math.max(value, MIN_MAIL_POLL_INTERVAL_SECONDS), MAX_MAIL_POLL_INTERVAL_SECONDS)
}

export function normalizeMailSyncSettingsValues(
  settings: MailSyncSettings
): MailSyncSettingsUpdate {
  const batchSize = Number.parseInt(String(settings.batch_size), 10)
  const syncWindows = Number.parseInt(String(settings.windows), 10)
  const pollIntervalSeconds = Number.parseInt(String(settings.poll_interval_seconds), 10)

  return {
    sync_enabled: settings.sync_enabled,
    batch_size: clampBatchSize(Number.isInteger(batchSize) ? batchSize : DEFAULT_MAIL_BATCH_SIZE),
    windows: clampWindows(Number.isInteger(syncWindows) ? syncWindows : DEFAULT_MAIL_SYNC_WINDOWS),
    poll_interval_seconds: clampPollIntervalSeconds(
      Number.isInteger(pollIntervalSeconds)
        ? pollIntervalSeconds
        : DEFAULT_MAIL_POLL_INTERVAL_SECONDS
    ),
    failure_threshold: settings.failure_threshold ?? 3,
  }
}
