import { describe, expect, it } from 'vitest'

import { normalizeMailSyncSettingsValues } from './mailSyncNormalization'
import {
  DEFAULT_MAIL_SYNC_WINDOWS,
  MAX_MAIL_SYNC_WINDOWS,
} from '../../../shared/mailSync/types'

describe('mail sync settings normalization', () => {
  it('applies window max and min boundaries', () => {
    const normalized = normalizeMailSyncSettingsValues({
      account_id: 'account-id',
      sync_enabled: true,
      batch_size: 1,
      windows: 0,
      poll_interval_seconds: 60,
      failure_threshold: 3,
      updated_at: '2026-07-21T00:00:00Z',
    })

    expect(normalized.windows).toBe(DEFAULT_MAIL_SYNC_WINDOWS)
  })

  it('uses max window bound when value is above limit', () => {
    const normalized = normalizeMailSyncSettingsValues({
      account_id: 'account-id',
      sync_enabled: true,
      batch_size: 1,
      windows: MAX_MAIL_SYNC_WINDOWS + 1,
      poll_interval_seconds: 60,
      failure_threshold: 3,
      updated_at: '2026-07-21T00:00:00Z',
    })

    expect(normalized.windows).toBe(MAX_MAIL_SYNC_WINDOWS)
  })

  it('uses default windows when value is not parseable', () => {
    const raw: Record<string, unknown> = {
      account_id: 'account-id',
      sync_enabled: true,
      batch_size: 1,
      windows: 'not-a-number' as unknown as number,
      poll_interval_seconds: 60,
      failure_threshold: 3,
      updated_at: '2026-07-21T00:00:00Z',
    }

    const normalized = normalizeMailSyncSettingsValues(raw as typeof raw)

    expect(normalized.windows).toBe(DEFAULT_MAIL_SYNC_WINDOWS)
  })
})
