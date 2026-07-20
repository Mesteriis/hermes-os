// Historical pre-clean-room navbar health test. Not part of the active test suite.
import { describe, expect, it } from 'vitest'
import { mailSyncStatusHealthChecks } from './appLayoutHealthChecks'
import type { MailSyncStatus } from '../../domains/communications/types/communications'

function failedSyncStatus(consecutiveFailures: number): MailSyncStatus {
  return {
    account_id: 'mail-account',
    status: 'failed',
    phase: 'failed',
    progress_mode: 'none',
    progress_percent: null,
    processed_messages: 0,
    estimated_total_messages: null,
    current_batch_size: 100,
    last_started_at: '2026-07-10T20:00:00Z',
    last_updated_at: '2026-07-10T20:00:01Z',
    last_completed_at: '2026-07-10T20:00:01Z',
    next_run_at: '2026-07-10T20:00:31Z',
    last_error_code: 'provider_unavailable',
    last_error_message: 'Provider unavailable',
    last_fetched_messages: 0,
    last_projected_messages: 0,
    last_upserted_personas: 0,
    last_upserted_organizations: 0,
    consecutive_failures: consecutiveFailures,
  }
}

describe('mailSyncStatusHealthChecks', () => {
  it('degrades system health only after three consecutive provider failures', () => {
    expect(mailSyncStatusHealthChecks([failedSyncStatus(1)])[0]?.status).toBe('healthy')
    expect(mailSyncStatusHealthChecks([failedSyncStatus(2)])[0]?.status).toBe('healthy')
    expect(mailSyncStatusHealthChecks([failedSyncStatus(3)])[0]?.status).toBe('degraded')
  })

  it('uses the configured degradation threshold', () => {
    expect(mailSyncStatusHealthChecks([failedSyncStatus(2)], 2)[0]?.status).toBe('degraded')
    expect(mailSyncStatusHealthChecks([failedSyncStatus(2)], 3)[0]?.status).toBe('healthy')
  })
})
