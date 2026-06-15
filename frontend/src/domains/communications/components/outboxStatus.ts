import type { EmailOutboxItem } from '../types/communications'

export type OutboxStatusTone = 'neutral' | 'success' | 'warning' | 'danger' | 'muted'

export type OutboxStatusPresentation = {
  title: string
  detail: string
  tone: OutboxStatusTone
  icon: string
  canUndo: boolean
  isVisible: boolean
}

type JsonObject = Record<string, unknown>

const UTC_DATE_FORMAT = new Intl.DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  hourCycle: 'h23',
  timeZone: 'UTC'
})

export function outboxStatusPresentation(
  item: EmailOutboxItem,
  now: Date = new Date()
): OutboxStatusPresentation {
  const readReceipt = objectField(item.metadata, 'latest_read_receipt')
  if (readReceipt && stringField(readReceipt, 'receipt_kind') === 'read') {
    return {
      title: 'Read',
      detail: timestampDetail('Read at', stringField(readReceipt, 'read_at')),
      tone: 'success',
      icon: 'tabler:mail-check',
      canUndo: false,
      isVisible: true
    }
  }

  const deliveryStatus = objectField(item.metadata, 'delivery_status')
  const deliveryStatusValue = deliveryStatus ? stringField(deliveryStatus, 'delivery_status') : null
  if (deliveryStatusValue === 'failed') {
    const smtpStatus = stringField(deliveryStatus, 'smtp_status')
    return {
      title: 'Delivery failed',
      detail: smtpStatus ? `Provider reported SMTP ${smtpStatus}` : 'Provider reported delivery failure',
      tone: 'danger',
      icon: 'tabler:alert-triangle',
      canUndo: false,
      isVisible: true
    }
  }
  if (deliveryStatusValue === 'delayed') {
    return {
      title: 'Delivery delayed',
      detail: timestampDetail('Provider update', stringField(deliveryStatus, 'recorded_at')),
      tone: 'warning',
      icon: 'tabler:clock-exclamation',
      canUndo: false,
      isVisible: true
    }
  }
  if (deliveryStatusValue === 'delivered') {
    return {
      title: 'Delivered',
      detail: timestampDetail('Confirmed at', stringField(deliveryStatus, 'recorded_at')),
      tone: 'success',
      icon: 'tabler:circle-check',
      canUndo: false,
      isVisible: true
    }
  }

  if (canUndo(item, now)) {
    return {
      title: 'Undo available',
      detail: timestampDetail('Until', item.undo_deadline_at),
      tone: 'warning',
      icon: 'tabler:arrow-back-up',
      canUndo: true,
      isVisible: true
    }
  }

  if (item.status === 'scheduled' && item.send_attempts > 1 && item.last_error) {
    return {
      title: 'Retry scheduled',
      detail: timestampDetail('Retry at', item.scheduled_send_at),
      tone: 'warning',
      icon: 'tabler:refresh-alert',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'scheduled') {
    return {
      title: 'Scheduled',
      detail: timestampDetail('Sends at', item.scheduled_send_at),
      tone: 'neutral',
      icon: 'tabler:calendar-time',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'queued') {
    return {
      title: 'Queued',
      detail: 'Waiting for delivery',
      tone: 'neutral',
      icon: 'tabler:send-2',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'sending') {
    return {
      title: 'Sending',
      detail: 'Provider handoff in progress',
      tone: 'neutral',
      icon: 'tabler:loader-2',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'failed') {
    return {
      title: 'Send failed',
      detail: item.last_error || 'Delivery worker failed',
      tone: 'danger',
      icon: 'tabler:alert-circle',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'canceled') {
    return {
      title: 'Canceled',
      detail: 'Undo send canceled delivery',
      tone: 'muted',
      icon: 'tabler:circle-off',
      canUndo: false,
      isVisible: true
    }
  }

  return {
    title: 'Sent',
    detail: item.provider_message_id ? 'Provider accepted message' : 'Sent',
    tone: 'muted',
    icon: 'tabler:mail-forward',
    canUndo: false,
    isVisible: false
  }
}

export function visibleOutboxStatusItems(
  items: EmailOutboxItem[],
  maxItems = 6,
  now: Date = new Date()
): EmailOutboxItem[] {
  return [...items]
    .filter((item) => outboxStatusPresentation(item, now).isVisible)
    .sort((a, b) => statusPriority(a) - statusPriority(b) || timestampMillis(b.updated_at) - timestampMillis(a.updated_at))
    .slice(0, maxItems)
}

function objectField(value: JsonObject, field: string): JsonObject | null {
  const nested = value[field]
  if (!nested || typeof nested !== 'object' || Array.isArray(nested)) return null
  return nested as JsonObject
}

function stringField(value: JsonObject | null, field: string): string | null {
  const nested = value?.[field]
  return typeof nested === 'string' && nested.trim() ? nested : null
}

function canUndo(item: EmailOutboxItem, now: Date): boolean {
  if (item.status !== 'queued' && item.status !== 'scheduled') return false
  if (!item.undo_deadline_at) return false
  return timestampMillis(item.undo_deadline_at) >= now.getTime()
}

function timestampDetail(prefix: string, value: string | null): string {
  if (!value) return prefix
  const timestamp = timestampMillis(value)
  if (!Number.isFinite(timestamp)) return prefix
  return `${prefix} ${UTC_DATE_FORMAT.format(new Date(timestamp))}`
}

function timestampMillis(value: string): number {
  const timestamp = Date.parse(value)
  return Number.isFinite(timestamp) ? timestamp : 0
}

function statusPriority(item: EmailOutboxItem): number {
  if (item.status === 'failed') return 0
  if (item.status === 'sending') return 1
  if (item.status === 'queued' || item.status === 'scheduled') return 2
  if (objectField(item.metadata, 'latest_read_receipt') || objectField(item.metadata, 'delivery_status')) return 3
  if (item.status === 'canceled') return 4
  return 5
}
