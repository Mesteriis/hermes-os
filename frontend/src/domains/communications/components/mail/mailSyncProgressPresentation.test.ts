import { describe, expect, it } from 'vitest'
import type { MailSyncStatus } from '../../types/communications'
import {
  formatMailSyncAge,
  mailSyncBadgeLabel,
  mailSyncDetail,
  mailSyncFailureKey,
  mailSyncIcon,
  mailSyncIsRunning,
  mailSyncIsStale,
  mailSyncPhaseLabel,
  mailSyncProgressClass,
  mailSyncProgressPercent,
  mailSyncTitle,
} from './mailSyncProgressPresentation'

describe('mail sync progress presentation', () => {
  it('classifies running, stale and bounded progress states', () => {
    const status = syncStatus({
      status: 'running',
      progress_mode: 'determinate',
      progress_percent: 140,
      last_updated_at: '2026-07-21T10:00:00.000Z',
    })

    expect(mailSyncIsRunning(status.status)).toBe(true)
    expect(mailSyncIsStale(status, Date.parse('2026-07-21T10:03:00.000Z'))).toBe(true)
    expect(mailSyncProgressPercent(status)).toBe(100)
    expect(mailSyncIcon(false, true)).toBe('tabler:alert-triangle')
    expect(mailSyncTitle(false, true, (key) => key)).toBe('Mail sync needs attention')
    expect(mailSyncBadgeLabel(false, false, 100, (key) => key)).toBe('100%')
    expect(mailSyncProgressClass({
      failed: false, failureKey: null, exitingFailureKey: null,
      stale: true, running: true, indeterminate: false,
    })).toContain('mail-sync-progress--warning')
  })

  it('builds translated labels, detail and stable failure identity', () => {
    const status = syncStatus({
      status: 'failed',
      phase: 'fetching',
      processed_messages: 4,
      estimated_total_messages: 8,
      current_batch_size: 2,
      last_error_code: 'timeout',
      last_error_message: 'provider timeout',
    })
    const t = (key: string) => key

    expect(mailSyncPhaseLabel('fetch', t)).toBe('fetching messages')
    expect(mailSyncDetail(status, true, t)).toContain('processed 4')
    expect(mailSyncFailureKey(status)).toContain('timeout')
    expect(formatMailSyncAge(61_000, t)).toBe('1 min ago')
  })
})

function syncStatus(overrides: Partial<MailSyncStatus> = {}): MailSyncStatus {
  return {
    account_id: 'account-1',
    status: 'queued',
    phase: 'listing',
    progress_mode: 'none',
    progress_percent: null,
    processed_messages: 0,
    estimated_total_messages: null,
    current_batch_size: 0,
    last_started_at: '2026-07-21T09:59:00.000Z',
    last_updated_at: '2026-07-21T09:59:00.000Z',
    last_completed_at: null,
    next_run_at: null,
    last_error_code: null,
    last_error_message: null,
    consecutive_failures: 0,
    last_fetched_messages: 0,
    last_projected_messages: 0,
    last_upserted_personas: 0,
    last_upserted_organizations: 0,
    ...overrides,
  }
}
