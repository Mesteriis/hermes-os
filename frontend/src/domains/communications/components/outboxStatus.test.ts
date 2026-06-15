import { describe, expect, it } from 'vitest'
import type { EmailOutboxItem } from '../types/communications'
import {
  outboxStatusPresentation,
  visibleOutboxStatusItems
} from './outboxStatus'

function outboxItem(overrides: Partial<EmailOutboxItem> = {}): EmailOutboxItem {
  return {
    outbox_id: 'outbox-1',
    account_id: 'account-1',
    draft_id: null,
    to_recipients: ['reader@example.com'],
    cc_recipients: [],
    bcc_recipients: [],
    subject: 'Quarterly update',
    body_text: 'Body',
    body_html: null,
    status: 'sent',
    scheduled_send_at: null,
    undo_deadline_at: null,
    send_attempts: 1,
    claimed_at: null,
    sent_at: '2026-06-15T09:00:00Z',
    provider_message_id: 'provider-message-1',
    last_error: null,
    metadata: {},
    created_at: '2026-06-15T08:59:00Z',
    updated_at: '2026-06-15T09:00:00Z',
    ...overrides
  }
}

describe('outbox status presentation', () => {
  it('prioritizes latest read receipt evidence for sent outbox items', () => {
    const item = outboxItem({
      metadata: {
        latest_read_receipt: {
          receipt_kind: 'read',
          read_at: '2026-06-15T09:10:00Z',
          source_kind: 'mdn'
        }
      }
    })

    expect(outboxStatusPresentation(item, new Date('2026-06-15T09:12:00Z'))).toMatchObject({
      title: 'Read',
      detail: 'Read at Jun 15, 09:10',
      tone: 'success',
      icon: 'tabler:mail-check'
    })
  })

  it('shows provider delivery failure evidence without exposing diagnostics', () => {
    const item = outboxItem({
      metadata: {
        delivery_status: {
          delivery_status: 'failed',
          smtp_status: '5.1.1',
          source_kind: 'dsn',
          diagnostic_code: 'smtp; private mailbox detail'
        }
      }
    })

    const presentation = outboxStatusPresentation(item, new Date('2026-06-15T09:12:00Z'))

    expect(presentation).toMatchObject({
      title: 'Delivery failed',
      detail: 'Provider reported SMTP 5.1.1',
      tone: 'danger',
      icon: 'tabler:alert-triangle'
    })
    expect(presentation.detail).not.toContain('private mailbox detail')
  })

  it('shows undo and retry timing for queued outbox records', () => {
    expect(outboxStatusPresentation(outboxItem({
      status: 'queued',
      undo_deadline_at: '2026-06-15T09:05:00Z',
      sent_at: null,
      provider_message_id: null
    }), new Date('2026-06-15T09:01:00Z'))).toMatchObject({
      title: 'Undo available',
      canUndo: true,
      tone: 'warning'
    })

    expect(outboxStatusPresentation(outboxItem({
      status: 'scheduled',
      scheduled_send_at: '2026-06-15T09:30:00Z',
      send_attempts: 2,
      last_error: 'SMTP send failed',
      sent_at: null,
      provider_message_id: null
    }), new Date('2026-06-15T09:01:00Z'))).toMatchObject({
      title: 'Retry scheduled',
      detail: 'Retry at Jun 15, 09:30',
      canUndo: false,
      tone: 'warning'
    })
  })

  it('filters out terminal sent items without fresh delivery evidence from the compact strip', () => {
    const items = visibleOutboxStatusItems([
      outboxItem({ outbox_id: 'sent-plain' }),
      outboxItem({
        outbox_id: 'sent-read',
        metadata: {
          latest_read_receipt: {
            receipt_kind: 'read',
            read_at: '2026-06-15T09:10:00Z',
            source_kind: 'mdn'
          }
        }
      }),
      outboxItem({
        outbox_id: 'queued',
        status: 'queued',
        sent_at: null,
        provider_message_id: null
      })
    ], 4)

    expect(items.map((item) => item.outbox_id)).toEqual(['queued', 'sent-read'])
  })
})
