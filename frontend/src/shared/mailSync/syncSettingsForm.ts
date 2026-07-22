import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from './types'
import {
  DEFAULT_MAIL_BATCH_SIZE,
  DEFAULT_MAIL_SYNC_WINDOWS,
  DEFAULT_MAIL_POLL_INTERVAL_SECONDS,
  MAX_MAIL_POLL_INTERVAL_SECONDS,
  MIN_MAIL_POLL_INTERVAL_SECONDS,
  MAX_MAIL_BATCH_SIZE,
  MAX_MAIL_SYNC_WINDOWS,
} from './types'

export const syncSettingsFormSchema = z.object({
  sync_enabled: z.boolean(),
  batch_size: z.coerce
    .number()
    .int('Batch size must be a whole number')
    .min(1, 'Batch size must be at least 1')
    .max(MAX_MAIL_BATCH_SIZE, `Batch size must be ${MAX_MAIL_BATCH_SIZE} or less`),
  windows: z.coerce
    .number()
    .int('Sync windows must be a whole number')
    .min(1, 'Sync windows must be at least 1')
    .max(MAX_MAIL_SYNC_WINDOWS, `Sync windows must be ${MAX_MAIL_SYNC_WINDOWS} or less`),
  poll_interval_seconds: z.coerce
    .number()
    .int('Poll interval must be a whole number')
    .min(MIN_MAIL_POLL_INTERVAL_SECONDS, `Poll interval must be at least ${MIN_MAIL_POLL_INTERVAL_SECONDS} seconds`)
    .max(
      MAX_MAIL_POLL_INTERVAL_SECONDS,
      `Poll interval must be ${MAX_MAIL_POLL_INTERVAL_SECONDS} seconds or less`
    ),
  failure_threshold: z.coerce
    .number()
    .int('Failure threshold must be a whole number')
    .min(1, 'Failure threshold must be at least 1')
    .max(10, 'Failure threshold must be 10 or less')
})

export type SyncSettingsFormValues = z.infer<typeof syncSettingsFormSchema>

export const syncSettingsVeeValidationSchema = toTypedSchema(syncSettingsFormSchema)

export function syncSettingsFormDefaults(settings: MailSyncSettings | null): SyncSettingsFormValues {
  return {
    sync_enabled: settings?.sync_enabled ?? true,
    batch_size: settings?.batch_size ?? DEFAULT_MAIL_BATCH_SIZE,
    windows: settings?.windows ?? DEFAULT_MAIL_SYNC_WINDOWS,
    poll_interval_seconds: settings?.poll_interval_seconds ?? DEFAULT_MAIL_POLL_INTERVAL_SECONDS,
    failure_threshold: settings?.failure_threshold ?? 3
  }
}

export function syncSettingsFormToUpdate(values: SyncSettingsFormValues): MailSyncSettingsUpdate {
  return {
    sync_enabled: values.sync_enabled,
    batch_size: values.batch_size,
    windows: values.windows,
    poll_interval_seconds: values.poll_interval_seconds,
    failure_threshold: values.failure_threshold
  }
}
