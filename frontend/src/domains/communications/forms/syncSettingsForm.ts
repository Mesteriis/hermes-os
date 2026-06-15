import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { MailSyncSettings, MailSyncSettingsUpdate } from '../types/communications'

export const syncSettingsFormSchema = z.object({
  sync_enabled: z.boolean(),
  batch_size: z.coerce
    .number()
    .int('Batch size must be a whole number')
    .min(1, 'Batch size must be at least 1')
    .max(500, 'Batch size must be 500 or less'),
  poll_interval_seconds: z.coerce
    .number()
    .int('Poll interval must be a whole number')
    .min(60, 'Poll interval must be at least 60 seconds')
    .max(86400, 'Poll interval must be 86400 seconds or less')
})

export type SyncSettingsFormValues = z.infer<typeof syncSettingsFormSchema>

export const syncSettingsVeeValidationSchema = toTypedSchema(syncSettingsFormSchema)

export function syncSettingsFormDefaults(settings: MailSyncSettings | null): SyncSettingsFormValues {
  return {
    sync_enabled: settings?.sync_enabled ?? true,
    batch_size: settings?.batch_size ?? 100,
    poll_interval_seconds: settings?.poll_interval_seconds ?? 300
  }
}

export function syncSettingsFormToUpdate(values: SyncSettingsFormValues): MailSyncSettingsUpdate {
  return {
    sync_enabled: values.sync_enabled,
    batch_size: values.batch_size,
    poll_interval_seconds: values.poll_interval_seconds
  }
}
