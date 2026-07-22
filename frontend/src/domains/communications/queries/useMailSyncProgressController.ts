import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import { useCommunicationActionNotifications } from './communicationActionNotifications'
import {
  formatMailSyncAge,
  mailSyncDetail,
  mailSyncFailureKey,
  mailSyncFailureNotificationBody,
  mailSyncBadgeLabel,
  mailSyncIcon,
  mailSyncIsRunning,
  mailSyncIsStale,
  mailSyncPhaseLabel,
  mailSyncProgressClass,
  mailSyncProgressPercent,
  mailSyncTimestampMs,
  mailSyncTitle,
  MAIL_SYNC_STALE_AFTER_MS,
} from '../components/mail/mailSyncProgressPresentation'
import type { MailSyncStatus } from '../types/communications'

type MailSyncProgressControllerProps = Readonly<{
  status?: MailSyncStatus | null
}>

interface MailSyncProgressControllerActions {
  visibilityChange(visible: boolean): void
}

const TICK_MS = 30 * 1000
const FAILED_EXIT_START_MS = 120
const FAILED_EXIT_DURATION_MS = 560

export function useMailSyncProgressController(
  props: MailSyncProgressControllerProps,
  actions: MailSyncProgressControllerActions,
) {
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

  const running = computed(() => mailSyncIsRunning(props.status?.status))
  const failed = computed(() => props.status?.status === 'failed')
  const failureKey = computed(() => mailSyncFailureKey(props.status))
  const failedVisible = computed(() => failed.value && failureKey.value !== dismissedFailureKey.value)
  const visible = computed(() => running.value || failedVisible.value)
  const lastMovementMs = computed(() => mailSyncTimestampMs(props.status?.last_updated_at ?? props.status?.last_started_at))
  const stale = computed(() => mailSyncIsStale(props.status, nowMs.value, MAIL_SYNC_STALE_AFTER_MS))
  const progressPercent = computed(() => mailSyncProgressPercent(props.status))
  const indeterminate = computed(() => running.value && progressPercent.value === null)
  const progressClass = computed(() => mailSyncProgressClass({
    failed: failed.value,
    failureKey: failureKey.value,
    exitingFailureKey: exitingFailureKey.value,
    stale: stale.value,
    running: running.value,
    indeterminate: indeterminate.value,
  }))
  const icon = computed(() => mailSyncIcon(failed.value, stale.value))
  const title = computed(() => mailSyncTitle(failed.value, stale.value, t))
  const subtitle = computed(() => {
    const status = props.status
    if (!status) return ''
    return mailSyncPhaseLabel(status.phase, t)
  })
  const detail = computed(() => mailSyncDetail(props.status, failed.value, t))
  const movementLabel = computed(() => {
    if (!lastMovementMs.value) return t('No sync movement yet')
    const ageMs = Math.max(0, nowMs.value - lastMovementMs.value)
    return `${t('updated')} ${formatMailSyncAge(ageMs, t)}`
  })
  const badgeLabel = computed(() => mailSyncBadgeLabel(
    failed.value,
    stale.value,
    progressPercent.value,
    t,
  ))

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
    { immediate: true },
  )

  watch(
    visible,
    (isVisible) => {
      actions.visibilityChange(isVisible)
    },
    { immediate: true },
  )

  function publishFailureNotification(key: string): void {
    const status = props.status
    if (!status || notifiedFailureKeys.has(key)) return

    notifiedFailureKeys.add(key)
    notifications.error(
      t('Mail sync failed'),
      mailSyncFailureNotificationBody(status, t),
      undefined,
      `mail-sync:${key}`,
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

  return {
    t,
    running,
    failed,
    failureKey,
    failedVisible,
    visible,
    lastMovementMs,
    stale,
    progressPercent,
    indeterminate,
    progressClass,
    icon,
    title,
    subtitle,
    detail,
    movementLabel,
    badgeLabel,
  }
}
