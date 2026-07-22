import { describe, expect, it } from 'vitest'

import {
  syncSettingsFormDefaults,
  syncSettingsFormSchema,
  syncSettingsFormToUpdate,
  type SyncSettingsFormValues,
} from './syncSettingsForm'
import {
  DEFAULT_MAIL_BATCH_SIZE,
  DEFAULT_MAIL_SYNC_WINDOWS,
  MAX_MAIL_SYNC_WINDOWS,
  DEFAULT_MAIL_POLL_INTERVAL_SECONDS,
} from './types'

describe('mail sync settings form contract', () => {
  it('applies null-safe defaults for sync settings', () => {
    const values = syncSettingsFormDefaults(null)

    expect(values).toEqual({
      sync_enabled: true,
      batch_size: DEFAULT_MAIL_BATCH_SIZE,
      windows: DEFAULT_MAIL_SYNC_WINDOWS,
      poll_interval_seconds: DEFAULT_MAIL_POLL_INTERVAL_SECONDS,
      failure_threshold: 3,
    })
  })

  it('keeps provided settings and allows windows at max boundary', () => {
    const values = syncSettingsFormDefaults({
      account_id: 'account-id',
      sync_enabled: false,
      batch_size: 100_000,
      windows: MAX_MAIL_SYNC_WINDOWS,
      poll_interval_seconds: 300,
      failure_threshold: 4,
      updated_at: '2026-07-21T00:00:00Z',
    })

    const parsed = syncSettingsFormSchema.parse(values as SyncSettingsFormValues)
    const updatePayload = syncSettingsFormToUpdate(parsed)

    expect(parsed.batch_size).toBe(100_000)
    expect(parsed.windows).toBe(MAX_MAIL_SYNC_WINDOWS)
    expect(updatePayload).toEqual(
      expect.objectContaining({
        batch_size: 100_000,
        windows: MAX_MAIL_SYNC_WINDOWS,
      })
    )
  })
})
