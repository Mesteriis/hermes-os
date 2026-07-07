<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Icon } from '@/shared/ui'
import { useCommunicationActionNotifications } from '../../queries/communicationActionNotifications'
import type { MailSyncStatus } from '../../types/communications'
import '../communicationDomainElements.css'

const STALE_AFTER_MS = 2 * 60 * 1000
const TICK_MS = 30 * 1000
const FAILED_EXIT_START_MS = 120
const FAILED_EXIT_DURATION_MS = 560

const props = defineProps<{
  status?: MailSyncStatus | null
}>()

const emit = defineEmits<{
  'visibility-change': [visible: boolean]
}>()

const { t } = useI18n()
const notifications = useCommunicationActionNotifications()
const nowMs = ref(Date.now())
const dismissedFailureKey = ref<string | null>(null)
const exitingFailureKey = ref<string | null>(null)
const notifiedFailureKeys = new Set<string>()
let timer: number | undefined
let failedExitStartTimer: number | undefined
let failedExitEndTimer: number | undefined

onMounted(() => {
  timer = window.setInterval(() => {
    nowMs.value = Date.now()
  }, TICK_MS)
})

onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer)
  clearFailedExitTimers()
})

const running = computed(() => isRunningStatus(props.status?.status))
const failed = computed(() => props.status?.status === 'failed')
const failureKey = computed(() => mailSyncFailureKey(props.status))
const failedVisible = computed(() => failed.value && failureKey.value !== dismissedFailureKey.value)
const visible = computed(() => running.value || failedVisible.value)
const lastMovementMs = computed(() =>
  timestampMs(props.status?.last_updated_at ?? props.status?.last_started_at)
)
const stale = computed(() => {
  if (!running.value || !lastMovementMs.value) return false
  return nowMs.value - lastMovementMs.value > STALE_AFTER_MS
})
const progressPercent = computed(() => {
  if (props.status?.progress_mode !== 'determinate') return null
  if (typeof props.status.progress_percent !== 'number') return null
  return Math.min(100, Math.max(0, props.status.progress_percent))
})
const indeterminate = computed(() => running.value && progressPercent.value === null)
const progressClass = computed(() => [
  'mail-sync-progress',
  failed.value && 'mail-sync-progress--failed',
  failureKey.value !== null &&
    exitingFailureKey.value === failureKey.value &&
    'mail-sync-progress--exiting',
  stale.value && 'mail-sync-progress--warning',
  running.value && !stale.value && 'mail-sync-progress--active',
  indeterminate.value && 'mail-sync-progress--indeterminate'
])
const icon = computed(() => {
  if (failed.value) return 'tabler:alert-circle'
  if (stale.value) return 'tabler:alert-triangle'
  return 'tabler:loader-2'
})
const title = computed(() => {
  if (failed.value) return t('Mail sync failed')
  if (stale.value) return t('Mail sync needs attention')
  return t('Loading mail')
})
const subtitle = computed(() => {
  const status = props.status
  if (!status) return ''
  return phaseLabel(status.phase)
})
const detail = computed(() => {
  const status = props.status
  if (!status) return ''
  const parts = [`${t('processed')} ${status.processed_messages}`]
  if (typeof status.estimated_total_messages === 'number') {
    parts.push(`${t('of')} ${status.estimated_total_messages}`)
  }
  if (status.current_batch_size > 0) {
    parts.push(`${t('batch')} ${status.current_batch_size}`)
  }
  if (failed.value && status.last_error_message) {
    parts.push(status.last_error_message)
  }
  if (status.last_fetched_messages > 0 || status.last_projected_messages > 0) {
    parts.push(`${t('fetched')} ${status.last_fetched_messages}`)
    parts.push(`${t('projected')} ${status.last_projected_messages}`)
  }
  return parts.join(' · ')
})
const movementLabel = computed(() => {
  if (!lastMovementMs.value) return t('No sync movement yet')
  const ageMs = Math.max(0, nowMs.value - lastMovementMs.value)
  return `${t('updated')} ${formatAge(ageMs)}`
})
const badgeLabel = computed(() => {
  if (failed.value) return t('failed')
  if (stale.value) return t('stalled')
  if (progressPercent.value !== null) return `${progressPercent.value}%`
  return t('running')
})

watch(
  failureKey,
  (key) => {
    clearFailedExitTimers()
    exitingFailureKey.value = null

    if (!key) {
      if (running.value) dismissedFailureKey.value = null
      return
    }

    if (dismissedFailureKey.value === key) return

    publishFailureNotification(key)
    scheduleFailureDismiss(key)
  },
  { immediate: true }
)

watch(
  visible,
  (isVisible) => {
    emit('visibility-change', isVisible)
  },
  { immediate: true }
)

function isRunningStatus(status: string | undefined): boolean {
  return (
    status === 'queued' ||
    status === 'running' ||
    status === 'recoverable_full_resync_needed'
  )
}

function phaseLabel(phase: string): string {
  switch (phase) {
    case 'listing':
      return t('listing mailboxes')
    case 'fetching':
    case 'fetch':
      return t('fetching messages')
    case 'projecting':
    case 'project':
      return t('projecting messages')
    case 'persons_graph':
      return t('updating graph')
    case 'completed':
      return t('completed')
    case 'failed':
      return t('failed')
    default:
      return phase || t('idle')
  }
}

function publishFailureNotification(key: string): void {
  const status = props.status
  if (!status || notifiedFailureKeys.has(key)) return

  notifiedFailureKeys.add(key)
  notifications.error(
    t('Mail sync failed'),
    mailSyncFailureNotificationBody(status),
    undefined,
    `mail-sync:${key}`
  )
}

function scheduleFailureDismiss(key: string): void {
  failedExitStartTimer = window.setTimeout(() => {
    if (failureKey.value === key) exitingFailureKey.value = key
  }, FAILED_EXIT_START_MS)

  failedExitEndTimer = window.setTimeout(() => {
    if (failureKey.value !== key) return
    dismissedFailureKey.value = key
    exitingFailureKey.value = null
  }, FAILED_EXIT_START_MS + FAILED_EXIT_DURATION_MS)
}

function clearFailedExitTimers(): void {
  if (failedExitStartTimer) window.clearTimeout(failedExitStartTimer)
  if (failedExitEndTimer) window.clearTimeout(failedExitEndTimer)
  failedExitStartTimer = undefined
  failedExitEndTimer = undefined
}

function mailSyncFailureKey(status: MailSyncStatus | null | undefined): string | null {
  if (!status || status.status !== 'failed') return null

  return [
    status.account_id,
    status.last_started_at ?? status.last_updated_at ?? 'unknown-start',
    status.last_error_code ?? 'unknown-code',
    status.last_error_message ?? 'unknown-error',
  ].join(':')
}

function mailSyncFailureNotificationBody(status: MailSyncStatus): string {
  const parts = [
    phaseLabel(status.phase),
    `${t('processed')} ${status.processed_messages}`,
  ]

  if (status.last_error_message) parts.push(status.last_error_message)

  return parts.join(' · ')
}

function timestampMs(value: string | null | undefined): number | null {
  if (!value) return null
  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? parsed : null
}

function formatAge(ageMs: number): string {
  const seconds = Math.floor(ageMs / 1000)
  if (seconds < 60) return t('just now')
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes} ${t('min ago')}`
  const hours = Math.floor(minutes / 60)
  return `${hours} ${t('h ago')}`
}
</script>

<template>
	<section v-if="status" :class="progressClass" aria-live="polite">
		<span class="mail-sync-progress__ambient" aria-hidden="true" />
		<div class="mail-sync-progress__top">
			<span class="mail-sync-progress__orb" aria-hidden="true">
				<Icon :icon="icon" size="1rem" class="mail-sync-progress__icon" />
			</span>
			<div class="mail-sync-progress__copy">
				<strong class="mail-sync-progress__title">{{ title }}</strong>
				<span class="mail-sync-progress__subtitle">{{ subtitle }}</span>
			</div>
			<span class="mail-sync-progress__badge">{{ badgeLabel }}</span>
		</div>
		<progress
			v-if="progressPercent !== null"
			class="mail-sync-progress__bar"
			:value="progressPercent"
			max="100"
			:aria-label="title"
		/>
		<progress
			v-else
			class="mail-sync-progress__bar"
			:aria-label="title"
		/>
		<div class="mail-sync-progress__meta">
			<span>{{ detail }}</span>
			<span>{{ movementLabel }}</span>
		</div>
	</section>
</template>
