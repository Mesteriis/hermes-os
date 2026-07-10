import { describe, expect, it } from 'vitest'
import {
  buildBackgroundJobRows,
  buildBackgroundJobSummaryTiles,
  buildBackgroundJobTabs,
  buildMailSyncStatusRows
} from './backgroundJobsPresentation'
import type { MailSyncStatus } from '../../../shared/mailSync/types'
import type { RealtimeStatusSnapshot } from '../../../shared/stores/realtimeStatus'
import type { SignalHubRuntimeState } from '../types/signalHub'

describe('backgroundJobsPresentation', () => {
  it('enriches declared backend jobs with mail sync and Signal Hub runtime evidence', () => {
    const rows = buildBackgroundJobRows({
      aiBusy: false,
      aiModelCount: 7,
      aiProviderCount: 2,
      healthItems: [],
      integrationAccountCount: 3,
      mailStatuses: [
        mailStatus({
          account_id: 'mail-1',
          status: 'running',
          phase: 'fetching',
          next_run_at: '2026-07-08T12:00:00Z'
        })
      ],
      mailStatusesError: null,
      mailStatusesLoading: false,
      realtimeStatus: realtimeStatus(),
      realtimeStatusLabel: 'Realtime live',
      realtimeStatusTone: 'success',
      replayPendingCount: 2,
      runtimeStates: [
        runtimeState('mail', 'mail_background_sync', 'running'),
        runtimeState('mail', 'address_book_sync', 'running'),
        runtimeState('system', 'signal_replay_dispatcher', 'running')
      ],
      signalSourceCount: 8
    })

    const mailSync = rows.find((row) => row.id === 'mail-background-sync')
    const addressBookSync = rows.find((row) => row.id === 'address-book-sync')
    const replay = rows.find((row) => row.id === 'signal-replay-dispatcher')
    const ai = rows.find((row) => row.id === 'ai-model-catalog-sync')

    expect(mailSync?.statusLabel).toBe('Running')
    expect(mailSync?.metric).toBe('1 mail accounts')
    expect(mailSync?.nextRunLabel).toBe('2026-07-08T12:00:00Z')
    expect(addressBookSync?.statusLabel).toBe('Running')
    expect(addressBookSync?.controlSection).toBe('accounts')
    expect(replay?.statusLabel).toBe('Replay pending')
    expect(replay?.tone).toBe('warn')
    expect(ai?.metric).toBe('7 models')
  })

  it('builds category tabs and summary from the enriched rows', () => {
    const rows = buildBackgroundJobRows({
      aiBusy: true,
      aiModelCount: 0,
      aiProviderCount: 0,
      healthItems: [],
      integrationAccountCount: 0,
      mailStatuses: [],
      mailStatusesError: 'Mail sync status request failed',
      mailStatusesLoading: false,
      realtimeStatus: realtimeStatus(),
      realtimeStatusLabel: 'Realtime offline',
      realtimeStatusTone: 'danger',
      replayPendingCount: 0,
      runtimeStates: [],
      signalSourceCount: 0
    })

    const tabs = buildBackgroundJobTabs(rows)
    const summary = buildBackgroundJobSummaryTiles(rows)

    expect(tabs[0]).toEqual({ id: 'all', label: 'All', count: rows.length })
    expect(tabs.some((tab) => tab.id === 'mail' && tab.count > 0)).toBe(true)
    expect(summary.find((tile) => tile.id === 'jobs')?.value).toBe(String(rows.length))
    expect(summary.find((tile) => tile.id === 'attention')?.tone).toBe('warn')
  })

  it('formats per-account mail sync status rows without provider content', () => {
    const rows = buildMailSyncStatusRows([
      mailStatus({
        account_id: 'mail-1',
        status: 'failed',
        phase: 'failed',
        last_error_message: 'auth expired',
        last_fetched_messages: 12,
        last_projected_messages: 9
      })
    ])

    expect(rows[0]?.tone).toBe('bad')
    expect(rows[0]?.throughputLabel).toBe('12 fetched / 9 projected')
    expect(rows[0]?.errorLabel).toBe('auth expired')
  })
})

function runtimeState(sourceCode: string, runtimeKind: string, state: string): SignalHubRuntimeState {
  return {
    id: `${sourceCode}-${runtimeKind}`,
    source_code: sourceCode,
    connection_id: null,
    runtime_kind: runtimeKind,
    state,
    last_started_at: '2026-07-08T11:00:00Z',
    last_stopped_at: null,
    last_heartbeat_at: '2026-07-08T11:01:00Z',
    last_error_at: null,
    last_error_code: null,
    last_error_message_redacted: null,
    metadata: {},
    updated_at: '2026-07-08T11:01:00Z'
  }
}

function mailStatus(overrides: Partial<MailSyncStatus>): MailSyncStatus {
  return {
    account_id: 'mail',
    status: 'completed',
    phase: 'completed',
    progress_mode: 'none',
    progress_percent: null,
    processed_messages: 0,
    estimated_total_messages: null,
    current_batch_size: 25,
    last_started_at: '2026-07-08T10:00:00Z',
    last_updated_at: '2026-07-08T10:01:00Z',
    last_completed_at: '2026-07-08T10:01:00Z',
    next_run_at: null,
    last_error_code: null,
    last_error_message: null,
    last_fetched_messages: 0,
    last_projected_messages: 0,
    last_upserted_personas: 0,
    last_upserted_organizations: 0,
    ...overrides
  }
}

function realtimeStatus(): RealtimeStatusSnapshot {
  return {
    transport: 'websocket',
    state: 'connected',
    attempt: null,
    maxAttempts: null,
    error: null,
    lastEventId: '42',
    lastEventAt: '2026-07-08T11:02:00Z',
    lastLaggedSkipped: null,
    lastLaggedAt: null,
    updatedAt: '2026-07-08T11:02:00Z'
  }
}
