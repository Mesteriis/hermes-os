import type { MailSyncStatus } from '../../../shared/mailSync/types'
import type { RealtimeStatusSnapshot, RealtimeStatusTone } from '../../../shared/stores/realtimeStatus'
import type { SignalHubHealth, SignalHubRuntimeState } from '../types/signalHub'
import type { SettingsSection } from '../stores/settings'

export type BackgroundJobGroup =
  | 'mail'
  | 'providers'
  | 'meetings'
  | 'signal-hub'
  | 'projections'
  | 'infrastructure'
  | 'ai'

export type BackgroundJobFilter = BackgroundJobGroup | 'all'
export type BackgroundJobTone = 'good' | 'warn' | 'bad' | 'neutral'

export interface BackgroundJobCatalogItem {
  id: string
  group: BackgroundJobGroup
  label: string
  description: string
  icon: string
  sourceCode: string | null
  runtimeKinds: string[]
  cadence: string
  evidence: string
  controlSection: SettingsSection | null
}

export interface BackgroundJobRow extends BackgroundJobCatalogItem {
  groupLabel: string
  statusLabel: string
  statusDetail: string
  tone: BackgroundJobTone
  metric: string
  lastActivityLabel: string
  nextRunLabel: string
  observedRuntimeCount: number
}

export interface BackgroundJobTab {
  id: BackgroundJobFilter
  label: string
  count: number
}

export interface BackgroundJobSummaryTile {
  id: string
  label: string
  value: string
  detail: string
  icon: string
  tone: BackgroundJobTone
}

export interface BackgroundMailSyncStatusRow {
  accountId: string
  status: string
  phase: string
  tone: BackgroundJobTone
  progressLabel: string
  lastActivityLabel: string
  nextRunLabel: string
  throughputLabel: string
  errorLabel: string
}

export interface BackgroundJobBuildInput {
  aiBusy: boolean
  aiModelCount: number
  aiProviderCount: number
  healthItems: SignalHubHealth[]
  integrationAccountCount: number
  mailStatuses: MailSyncStatus[]
  mailStatusesError: string | null
  mailStatusesLoading: boolean
  realtimeStatus: RealtimeStatusSnapshot
  realtimeStatusLabel: string
  realtimeStatusTone: RealtimeStatusTone
  replayPendingCount: number
  runtimeStates: SignalHubRuntimeState[]
  signalSourceCount: number
}

export const BACKGROUND_JOB_GROUP_LABELS: Record<BackgroundJobGroup, string> = {
  mail: 'Mail',
  providers: 'Providers',
  meetings: 'Meetings',
  'signal-hub': 'Signal Hub',
  projections: 'Projections',
  infrastructure: 'Infrastructure',
  ai: 'AI'
}

export const BACKGROUND_JOB_GROUP_ORDER: BackgroundJobGroup[] = [
  'mail',
  'providers',
  'meetings',
  'signal-hub',
  'projections',
  'infrastructure',
  'ai'
]

export const BACKGROUND_JOB_CATALOG: BackgroundJobCatalogItem[] = [
  {
    id: 'mail-background-sync',
    group: 'mail',
    label: 'Mail background sync',
    description: 'Polls due mail accounts and projects fetched messages into Communications evidence.',
    icon: 'tabler:mail-down',
    sourceCode: 'mail',
    runtimeKinds: ['mail_background_sync'],
    cadence: '30s scheduler tick, then per-account poll interval',
    evidence: 'backend/src/application/bootstrap.rs + mail sync status API',
    controlSection: 'accounts'
  },
  {
    id: 'address-book-sync',
    group: 'mail',
    label: 'Address book sync',
    description: 'Pulls provider contacts into Personas and pushes confirmed local contacts when bidirectional sync is enabled.',
    icon: 'tabler:address-book',
    sourceCode: 'mail',
    runtimeKinds: ['address_book_sync'],
    cadence: '300s scheduler tick, then per-account address-book poll interval',
    evidence: 'backend/src/application/bootstrap.rs + /api/v1/integrations/mail/accounts/{account_id}/address-book-sync-now',
    controlSection: 'accounts'
  },
  {
    id: 'mail-outbox-delivery',
    group: 'mail',
    label: 'Mail outbox delivery',
    description: 'Claims due outbound email commands and delivers them through configured mail transports.',
    icon: 'tabler:send',
    sourceCode: 'mail',
    runtimeKinds: ['mail_outbox_delivery'],
    cadence: '10s scheduler tick',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'accounts'
  },
  {
    id: 'provider-command-executors',
    group: 'providers',
    label: 'Provider command executors',
    description: 'Executes queued provider commands for Telegram, WhatsApp and Zulip runtime channels.',
    icon: 'tabler:terminal-2',
    sourceCode: null,
    runtimeKinds: ['telegram_command_executor', 'whatsapp_command_executor', 'zulip_command_executor'],
    cadence: 'Provider runtime ticks',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'zulip-ingest-download',
    group: 'providers',
    label: 'Zulip ingest and attachment download',
    description: 'Ingests Zulip events and downloads attachment payloads through runtime-owned workers.',
    icon: 'tabler:paperclip',
    sourceCode: 'zulip',
    runtimeKinds: ['zulip_event_ingest', 'zulip_attachment_download'],
    cadence: 'Runtime worker ticks',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'provider-observation-reconciliation',
    group: 'providers',
    label: 'Provider observation reconciliation',
    description: 'Reconciles provider observations before they are promoted into communication projections.',
    icon: 'tabler:arrows-join',
    sourceCode: null,
    runtimeKinds: [
      'zulip_provider_observation_reconciliation',
      'whatsapp_provider_observation_reconciliation',
      'communication_provider_observation_projection'
    ],
    cadence: 'Event consumer ticks',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'whatsapp-runtime-restore',
    group: 'providers',
    label: 'WhatsApp runtime restore',
    description: 'Restores WhatsApp runtime state and projects runtime events after backend startup.',
    icon: 'tabler:brand-whatsapp',
    sourceCode: 'whatsapp',
    runtimeKinds: ['whatsapp_runtime_restore_reconciliation', 'whatsapp_runtime_event_projection'],
    cadence: 'Startup/runtime reconciliation',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'zoom-maintenance-sync',
    group: 'meetings',
    label: 'Zoom maintenance and recording sync',
    description: 'Refreshes Zoom tokens and imports recent recording metadata for review pipelines.',
    icon: 'tabler:video',
    sourceCode: 'zoom',
    runtimeKinds: ['zoom_token_maintenance', 'zoom_recording_sync'],
    cadence: '60s token tick, 300s recording sync',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'meeting-retention-cleanup',
    group: 'meetings',
    label: 'Meeting retention cleanup',
    description: 'Applies retention cleanup for Zoom and Yandex Telemost meeting artifacts.',
    icon: 'tabler:trash',
    sourceCode: null,
    runtimeKinds: ['zoom_retention_cleanup', 'yandex_telemost_retention_cleanup'],
    cadence: '3600s cleanup tick',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'meeting-projections',
    group: 'meetings',
    label: 'Meeting signal projections',
    description: 'Projects Zoom and Yandex Telemost calendar, participant and signal detections.',
    icon: 'tabler:calendar-stats',
    sourceCode: null,
    runtimeKinds: [
      'zoom_signal_detection',
      'zoom_calendar_matching',
      'zoom_participant_identity',
      'yandex_telemost_calendar_matching'
    ],
    cadence: 'Event consumer ticks',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'signal-hub-raw-dispatcher',
    group: 'signal-hub',
    label: 'Signal Hub raw signal dispatcher',
    description: 'Dispatches accepted raw signals from Signal Hub routes into consumers and projections.',
    icon: 'tabler:route',
    sourceCode: 'system',
    runtimeKinds: ['signal_hub_raw_signal_dispatcher'],
    cadence: 'Signal consumer tick',
    evidence: 'Signal Hub runtime states',
    controlSection: 'signal-hub'
  },
  {
    id: 'signal-replay-dispatcher',
    group: 'signal-hub',
    label: 'Signal replay dispatcher',
    description: 'Processes replay requests for durable Signal Hub history and projections.',
    icon: 'tabler:player-track-next',
    sourceCode: 'system',
    runtimeKinds: ['signal_replay_dispatcher'],
    cadence: 'Replay dispatcher tick',
    evidence: 'Signal Hub replay request API',
    controlSection: 'signal-hub'
  },
  {
    id: 'review-evidence-projections',
    group: 'projections',
    label: 'Review and evidence projections',
    description: 'Keeps persona evidence, identity review inbox and project link review effects current.',
    icon: 'tabler:git-merge',
    sourceCode: 'system',
    runtimeKinds: ['persona_derived_evidence', 'persona_identity_review_inbox', 'project_link_review_effects'],
    cadence: 'Event consumer ticks',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'realtime-transcript-workers',
    group: 'projections',
    label: 'Realtime transcript workers',
    description: 'Executes realtime transcript requests and projects transcript events into reviewable state.',
    icon: 'tabler:microphone-2',
    sourceCode: 'system',
    runtimeKinds: ['realtime_conversation_transcript_execution', 'realtime_conversation_transcript_projection'],
    cadence: 'Event consumer ticks',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'event-outbox-dispatcher',
    group: 'infrastructure',
    label: 'Event outbox dispatcher',
    description: 'Publishes queued domain events to the configured realtime/event bus when available.',
    icon: 'tabler:broadcast',
    sourceCode: 'system',
    runtimeKinds: ['event_outbox_dispatcher'],
    cadence: 'Event bus dispatcher tick',
    evidence: 'backend/src/application/bootstrap.rs',
    controlSection: 'signal-hub'
  },
  {
    id: 'realtime-transport',
    group: 'infrastructure',
    label: 'Realtime transport recovery',
    description: 'Tracks frontend realtime cursor health and recovery state for missed UI updates.',
    icon: 'tabler:cloud-data-connection',
    sourceCode: 'system',
    runtimeKinds: [],
    cadence: 'Realtime client lifecycle',
    evidence: 'frontend/src/shared/stores/realtimeStatus.ts',
    controlSection: null
  },
  {
    id: 'ai-model-catalog-sync',
    group: 'ai',
    label: 'AI model catalog sync',
    description: 'Refreshes provider model inventories used by action routing and model availability controls.',
    icon: 'tabler:sparkles',
    sourceCode: 'ai',
    runtimeKinds: [],
    cadence: 'Manual provider sync action',
    evidence: 'AI Hub provider model API',
    controlSection: 'ai'
  }
]

export function buildBackgroundJobRows(input: BackgroundJobBuildInput): BackgroundJobRow[] {
  return BACKGROUND_JOB_CATALOG.map((item) => enrichBackgroundJob(item, input))
}

export function buildBackgroundJobTabs(rows: BackgroundJobRow[]): BackgroundJobTab[] {
  return [
    { id: 'all', label: 'All', count: rows.length },
    ...BACKGROUND_JOB_GROUP_ORDER.map((group) => ({
      id: group,
      label: BACKGROUND_JOB_GROUP_LABELS[group],
      count: rows.filter((row) => row.group === group).length
    }))
  ]
}

export function buildBackgroundJobSummaryTiles(rows: BackgroundJobRow[]): BackgroundJobSummaryTile[] {
  const observedCount = rows.filter((row) => row.observedRuntimeCount > 0 || row.id === 'mail-background-sync').length
  const attentionCount = rows.filter((row) => row.tone === 'bad' || row.tone === 'warn').length
  return [
    {
      id: 'jobs',
      label: 'Jobs',
      value: String(rows.length),
      detail: 'Declared schedulers and consumers',
      icon: 'tabler:clock-cog',
      tone: 'neutral'
    },
    {
      id: 'observed',
      label: 'Observed',
      value: String(observedCount),
      detail: 'Have live status evidence',
      icon: 'tabler:activity',
      tone: observedCount > 0 ? 'good' : 'neutral'
    },
    {
      id: 'attention',
      label: 'Attention',
      value: String(attentionCount),
      detail: 'Warnings, errors or pending work',
      icon: 'tabler:alert-triangle',
      tone: attentionCount > 0 ? 'warn' : 'good'
    },
    {
      id: 'running',
      label: 'Running',
      value: String(rows.filter((row) => row.statusLabel === 'Running').length),
      detail: 'Reported active by runtime state',
      icon: 'tabler:player-play',
      tone: 'good'
    }
  ]
}

export function buildMailSyncStatusRows(statuses: MailSyncStatus[]): BackgroundMailSyncStatusRow[] {
  return statuses.map((status) => ({
    accountId: status.account_id,
    status: status.status,
    phase: status.phase,
    tone: mailStatusTone(status),
    progressLabel: mailProgressLabel(status),
    lastActivityLabel: formatJobTimestamp(latestTimestamp([
      status.last_updated_at,
      status.last_started_at,
      status.last_completed_at
    ])),
    nextRunLabel: formatJobTimestamp(status.next_run_at),
    throughputLabel: `${status.last_fetched_messages} fetched / ${status.last_projected_messages} projected`,
    errorLabel: status.last_error_message ?? ''
  }))
}

export function formatJobTimestamp(value: string | null): string {
  if (!value) return 'n/a'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toISOString().replace(/\.\d{3}Z$/, 'Z')
}

function enrichBackgroundJob(item: BackgroundJobCatalogItem, input: BackgroundJobBuildInput): BackgroundJobRow {
  if (item.id === 'mail-background-sync') return enrichMailBackgroundSync(item, input)
  if (item.id === 'signal-replay-dispatcher') return enrichSignalReplayDispatcher(item, input)
  if (item.id === 'realtime-transport') return enrichRealtimeTransport(item, input)
  if (item.id === 'ai-model-catalog-sync') return enrichAiModelSync(item, input)

  const runtimeStates = runtimeStatesForItem(item, input.runtimeStates)
  const health = item.sourceCode ? highestPriorityHealth(input.healthItems, item.sourceCode) : null
  const runtimeStatus = statusFromRuntimeStates(runtimeStates)
  const healthTone = health ? toneFromHealth(health.level) : 'neutral'
  const tone = worseTone(runtimeStatus.tone, healthTone)
  return {
    ...item,
    groupLabel: BACKGROUND_JOB_GROUP_LABELS[item.group],
    statusLabel: runtimeStatus.label,
    statusDetail: health?.summary ?? runtimeStatus.detail,
    tone,
    metric: runtimeStates.length > 0
      ? `${runtimeStates.length}/${item.runtimeKinds.length} runtimes observed`
      : `${item.runtimeKinds.length} runtimes declared`,
    lastActivityLabel: formatJobTimestamp(latestRuntimeTimestamp(runtimeStates)),
    nextRunLabel: 'runtime-owned',
    observedRuntimeCount: runtimeStates.length
  }
}

function enrichMailBackgroundSync(item: BackgroundJobCatalogItem, input: BackgroundJobBuildInput): BackgroundJobRow {
  const runtimeStates = runtimeStatesForItem(item, input.runtimeStates)
  const runtimeStatus = statusFromRuntimeStates(runtimeStates)
  if (input.mailStatusesLoading) {
    return rowFromItem(item, 'Loading', 'Mail sync status query is loading.', 'neutral', 'loading', runtimeStates)
  }
  if (input.mailStatusesError) {
    return rowFromItem(item, 'Unavailable', input.mailStatusesError, 'bad', 'status API failed', runtimeStates)
  }
  if (input.mailStatuses.length === 0) {
    return rowFromItem(item, 'No accounts', 'No mail account sync statuses were returned.', 'neutral', '0 accounts', runtimeStates)
  }

  const activeStatuses = input.mailStatuses.filter((status) =>
    ['queued', 'running', 'recoverable_full_resync_needed'].includes(status.status)
  )
  const failedStatuses = input.mailStatuses.filter((status) =>
    status.status === 'failed' || Boolean(status.last_error_code || status.last_error_message)
  )
  const nextRun = earliestTimestamp(input.mailStatuses.map((status) => status.next_run_at))
  const latest = latestTimestamp(input.mailStatuses.flatMap((status) => [
    status.last_updated_at,
    status.last_started_at,
    status.last_completed_at
  ]))
  const tone = failedStatuses.length > 0 ? 'bad' : activeStatuses.length > 0 ? 'good' : runtimeStatus.tone
  const statusLabel = failedStatuses.length > 0 ? 'Attention' : activeStatuses.length > 0 ? 'Running' : 'Scheduled'
  return {
    ...item,
    groupLabel: BACKGROUND_JOB_GROUP_LABELS[item.group],
    statusLabel,
    statusDetail: failedStatuses.length > 0
      ? `${failedStatuses.length} account sync status has errors`
      : `${input.mailStatuses.length} account sync schedules loaded`,
    tone,
    metric: `${input.mailStatuses.length} mail accounts`,
    lastActivityLabel: formatJobTimestamp(latest),
    nextRunLabel: formatJobTimestamp(nextRun),
    observedRuntimeCount: Math.max(runtimeStates.length, 1)
  }
}

function enrichSignalReplayDispatcher(item: BackgroundJobCatalogItem, input: BackgroundJobBuildInput): BackgroundJobRow {
  const base = enrichBackgroundRuntimeOnlyJob(item, input)
  if (input.replayPendingCount === 0) return base
  return {
    ...base,
    statusLabel: 'Replay pending',
    statusDetail: 'Signal Hub has replay requests that are not completed yet.',
    tone: worseTone(base.tone, 'warn'),
    metric: `${input.replayPendingCount} pending replay requests`
  }
}

function enrichBackgroundRuntimeOnlyJob(item: BackgroundJobCatalogItem, input: BackgroundJobBuildInput): BackgroundJobRow {
  const runtimeStates = runtimeStatesForItem(item, input.runtimeStates)
  const runtimeStatus = statusFromRuntimeStates(runtimeStates)
  return {
    ...item,
    groupLabel: BACKGROUND_JOB_GROUP_LABELS[item.group],
    statusLabel: runtimeStatus.label,
    statusDetail: runtimeStatus.detail,
    tone: runtimeStatus.tone,
    metric: runtimeStates.length > 0
      ? `${runtimeStates.length}/${item.runtimeKinds.length} runtimes observed`
      : `${item.runtimeKinds.length} runtimes declared`,
    lastActivityLabel: formatJobTimestamp(latestRuntimeTimestamp(runtimeStates)),
    nextRunLabel: 'runtime-owned',
    observedRuntimeCount: runtimeStates.length
  }
}

function enrichRealtimeTransport(item: BackgroundJobCatalogItem, input: BackgroundJobBuildInput): BackgroundJobRow {
  return {
    ...item,
    groupLabel: BACKGROUND_JOB_GROUP_LABELS[item.group],
    statusLabel: input.realtimeStatusLabel,
    statusDetail: input.realtimeStatus.error ?? `Transport ${input.realtimeStatus.transport}`,
    tone: toneFromRealtime(input.realtimeStatusTone),
    metric: input.realtimeStatus.lastEventId ? `cursor ${input.realtimeStatus.lastEventId}` : 'waiting for cursor',
    lastActivityLabel: formatJobTimestamp(input.realtimeStatus.updatedAt),
    nextRunLabel: 'client-owned',
    observedRuntimeCount: input.realtimeStatus.updatedAt ? 1 : 0
  }
}

function enrichAiModelSync(item: BackgroundJobCatalogItem, input: BackgroundJobBuildInput): BackgroundJobRow {
  return {
    ...item,
    groupLabel: BACKGROUND_JOB_GROUP_LABELS[item.group],
    statusLabel: input.aiBusy ? 'Syncing' : 'Idle',
    statusDetail: `${input.aiProviderCount} providers and ${input.aiModelCount} catalog models are loaded.`,
    tone: input.aiBusy ? 'warn' : 'neutral',
    metric: `${input.aiModelCount} models`,
    lastActivityLabel: 'manual',
    nextRunLabel: 'manual',
    observedRuntimeCount: input.aiProviderCount > 0 || input.aiModelCount > 0 ? 1 : 0
  }
}

function rowFromItem(
  item: BackgroundJobCatalogItem,
  statusLabel: string,
  statusDetail: string,
  tone: BackgroundJobTone,
  metric: string,
  runtimeStates: SignalHubRuntimeState[]
): BackgroundJobRow {
  return {
    ...item,
    groupLabel: BACKGROUND_JOB_GROUP_LABELS[item.group],
    statusLabel,
    statusDetail,
    tone,
    metric,
    lastActivityLabel: formatJobTimestamp(latestRuntimeTimestamp(runtimeStates)),
    nextRunLabel: 'n/a',
    observedRuntimeCount: runtimeStates.length
  }
}

function runtimeStatesForItem(item: BackgroundJobCatalogItem, runtimeStates: SignalHubRuntimeState[]): SignalHubRuntimeState[] {
  return runtimeStates.filter((runtime) => item.runtimeKinds.includes(runtime.runtime_kind))
}

function statusFromRuntimeStates(runtimeStates: SignalHubRuntimeState[]): { label: string; detail: string; tone: BackgroundJobTone } {
  if (runtimeStates.length === 0) {
    return {
      label: 'Declared',
      detail: 'No Signal Hub runtime heartbeat is currently exposed for this job.',
      tone: 'neutral'
    }
  }
  const failed = runtimeStates.find((runtime) =>
    Boolean(runtime.last_error_code || runtime.last_error_message_redacted) ||
    ['failed', 'error', 'blocked'].includes(runtime.state)
  )
  if (failed) {
    return {
      label: 'Attention',
      detail: failed.last_error_message_redacted ?? failed.last_error_code ?? 'Runtime reported an error.',
      tone: 'bad'
    }
  }
  if (runtimeStates.some((runtime) => ['running', 'active', 'connected', 'ready'].includes(runtime.state))) {
    return {
      label: 'Running',
      detail: 'Runtime state reports active processing.',
      tone: 'good'
    }
  }
  const paused = runtimeStates.find((runtime) =>
    ['paused', 'disabled', 'stopped', 'offline'].includes(runtime.state)
  )
  if (paused) {
    return {
      label: capitalize(paused.state),
      detail: 'Runtime state is not actively processing.',
      tone: 'warn'
    }
  }
  return {
    label: capitalize(runtimeStates[0]?.state ?? 'Observed'),
    detail: 'Runtime state was reported by Signal Hub.',
    tone: 'neutral'
  }
}

function mailStatusTone(status: MailSyncStatus): BackgroundJobTone {
  if (status.status === 'failed' || status.last_error_code || status.last_error_message) return 'bad'
  if (['queued', 'running', 'recoverable_full_resync_needed'].includes(status.status)) return 'good'
  if (status.status === 'skipped') return 'warn'
  return 'neutral'
}

function mailProgressLabel(status: MailSyncStatus): string {
  if (status.progress_percent !== null) return `${status.progress_percent}%`
  if (status.estimated_total_messages !== null) {
    return `${status.processed_messages}/${status.estimated_total_messages}`
  }
  return `${status.processed_messages} processed`
}

function highestPriorityHealth(healthItems: SignalHubHealth[], sourceCode: string): SignalHubHealth | null {
  const items = healthItems.filter((item) => item.source_code === sourceCode)
  return items.sort((a, b) => healthPriority(b.level) - healthPriority(a.level))[0] ?? null
}

function healthPriority(level: string): number {
  if (level === 'failed' || level === 'unhealthy') return 3
  if (level === 'degraded' || level === 'warning') return 2
  if (level === 'healthy') return 1
  return 0
}

function toneFromHealth(level: string): BackgroundJobTone {
  if (level === 'failed' || level === 'unhealthy') return 'bad'
  if (level === 'degraded' || level === 'warning') return 'warn'
  if (level === 'healthy') return 'good'
  return 'neutral'
}

function toneFromRealtime(tone: RealtimeStatusTone): BackgroundJobTone {
  if (tone === 'success') return 'good'
  if (tone === 'warning') return 'warn'
  if (tone === 'danger') return 'bad'
  return 'neutral'
}

function worseTone(left: BackgroundJobTone, right: BackgroundJobTone): BackgroundJobTone {
  const order: Record<BackgroundJobTone, number> = {
    neutral: 0,
    good: 1,
    warn: 2,
    bad: 3
  }
  return order[right] > order[left] ? right : left
}

function latestRuntimeTimestamp(runtimeStates: SignalHubRuntimeState[]): string | null {
  return latestTimestamp(runtimeStates.flatMap((runtime) => [
    runtime.last_heartbeat_at,
    runtime.last_error_at,
    runtime.last_started_at,
    runtime.last_stopped_at,
    runtime.updated_at
  ]))
}

function latestTimestamp(values: Array<string | null>): string | null {
  return timestampBy(values, (left, right) => left > right)
}

function earliestTimestamp(values: Array<string | null>): string | null {
  return timestampBy(values, (left, right) => left < right)
}

function timestampBy(values: Array<string | null>, compare: (left: number, right: number) => boolean): string | null {
  let selected: { value: string; time: number } | null = null
  for (const value of values) {
    if (!value) continue
    const time = Date.parse(value)
    if (!Number.isFinite(time)) continue
    if (!selected || compare(time, selected.time)) {
      selected = { value, time }
    }
  }
  return selected?.value ?? null
}

function capitalize(value: string): string {
  if (!value) return value
  return `${value.charAt(0).toUpperCase()}${value.slice(1)}`
}
