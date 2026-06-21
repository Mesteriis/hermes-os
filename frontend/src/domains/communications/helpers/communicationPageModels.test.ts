import { describe, expect, it } from 'vitest'
import type { CommunicationMessageSummary, CommunicationDraft, ThreadMessage } from '../types/communications'
import {
  aiSummaryContractFromMetadata,
  composeFormToSendRequest,
  draftToComposeForm,
  emptyCommunicationMessageInsight,
  communicationMessageLabelsFromMetadata,
  communicationMessageSnoozeUntilFromMetadata,
  communicationKnowledgeSectionsFromSummaryContract,
  communicationExtractionSectionsFromInsight,
  forwardComposeForm,
  newComposeForm,
  replyComposeForm,
  replyAllComposeForm,
  threadReplyComposeForm
} from './communicationPageModels'

function message(overrides: Partial<CommunicationMessageSummary> = {}): CommunicationMessageSummary {
  return {
    message_id: 'msg-1',
    raw_record_id: 'raw-1',
    account_id: 'account-1',
    provider_record_id: 'provider-1',
    subject: 'Quarterly update',
    sender: 'alice@example.com',
    recipients: ['owner@example.com'],
    body_text_preview: 'Preview',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    channel_kind: 'email',
    conversation_id: null,
    sender_display_name: null,
    delivery_state: 'received',
    workflow_state: 'new',
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    ai_summary_generated_at: null,
    message_metadata: {},
    attachment_count: 0,
    local_state: 'active',
    local_state_changed_at: null,
    ...overrides
  }
}

function draft(): CommunicationDraft {
  return {
    draft_id: 'draft-1',
    account_id: 'account-1',
    persona_id: null,
    to_recipients: ['to@example.com'],
    cc_recipients: ['cc@example.com'],
    bcc_recipients: ['bcc@example.com'],
    subject: 'Draft subject',
    body_text: 'Draft body',
    body_html: null,
    in_reply_to: 'provider-1',
    references: [],
    status: 'draft',
    scheduled_send_at: null,
    send_attempts: 0,
    last_error: null,
    metadata: {},
    created_at: '2026-06-15T10:00:00Z',
    updated_at: '2026-06-15T10:01:00Z'
  }
}

function threadMessage(overrides: Partial<ThreadMessage> = {}): ThreadMessage {
  return {
    message_id: 'thread-msg-1',
    provider_record_id: 'provider-thread-1',
    account_id: 'account-1',
    subject: 'Quarterly update',
    sender: 'Ada <ada@example.com>',
    sender_display_name: 'Ada',
    body_text: 'Line one\nLine two with <angle>',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    workflow_state: 'new',
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    delivery_state: 'received',
    attachment_count: 0,
    attachments: [],
    ...overrides
  }
}

describe('mail page model helpers', () => {
  it('creates an empty message insight shell for the selected message', () => {
    expect(emptyCommunicationMessageInsight('msg-1')).toMatchObject({
      messageId: 'msg-1',
      tasks: [],
      notes: [],
      translation: null
    })
  })

  it('extracts structured AI summary contracts from message metadata safely', () => {
    expect(aiSummaryContractFromMetadata({
      ai_summary_contract: {
        key_points: ['Contract review'],
        action_items: ['Reply by Friday'],
        risks: ['Payment risk'],
        deadlines: ['Friday'],
        event_candidates: [{ title: 'Review meeting', evidence: 'Meeting on Monday' }],
        persona_candidates: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }],
        organization_candidates: [{ title: 'Acme Corp', evidence: 'acme.example' }],
        document_candidates: [{ title: 'MSA attachment', evidence: 'attached MSA' }],
        agreement_candidates: [{ title: 'NDA', evidence: 'review NDA' }]
      }
    })).toEqual({
      key_points: ['Contract review'],
      action_items: ['Reply by Friday'],
      risks: ['Payment risk'],
      deadlines: ['Friday'],
      event_candidates: [{ title: 'Review meeting', evidence: 'Meeting on Monday' }],
      persona_candidates: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }],
      organization_candidates: [{ title: 'Acme Corp', evidence: 'acme.example' }],
      document_candidates: [{ title: 'MSA attachment', evidence: 'attached MSA' }],
      agreement_candidates: [{ title: 'NDA', evidence: 'review NDA' }]
    })

    expect(aiSummaryContractFromMetadata({
      ai_summary_contract: {
        key_points: ['ok', 42],
        action_items: 'not-array',
        risks: [],
        deadlines: [null],
        event_candidates: ['legacy event candidate', { title: 42 }],
        persona_candidates: [{ title: 'Ada', evidence: 42 }],
        organization_candidates: null,
        document_candidates: [{ title: 'Doc' }],
        agreement_candidates: [{ evidence: 'Missing title' }]
      }
    })).toEqual({
      key_points: ['ok'],
      action_items: [],
      risks: [],
      deadlines: [],
      event_candidates: [{ title: 'legacy event candidate', evidence: 'legacy event candidate' }],
      persona_candidates: [{ title: 'Ada', evidence: '' }],
      organization_candidates: [],
      document_candidates: [{ title: 'Doc', evidence: '' }],
      agreement_candidates: []
    })

    expect(aiSummaryContractFromMetadata({})).toBeNull()
  })

  it('builds mail knowledge review sections from AI summary candidates', () => {
    const contract = aiSummaryContractFromMetadata({
      ai_summary_contract: {
        key_points: [],
        action_items: [],
        risks: [],
        deadlines: [],
        event_candidates: [{ title: 'Review meeting', evidence: 'Monday 10:00' }],
        persona_candidates: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }],
        organization_candidates: [{ title: 'Acme Corp', evidence: 'acme.example' }],
        document_candidates: [{ title: 'MSA attachment', evidence: 'attached MSA' }],
        agreement_candidates: [{ title: 'NDA', evidence: 'review NDA' }]
      }
    })

    expect(communicationKnowledgeSectionsFromSummaryContract(contract)).toEqual([
      {
        kind: 'event',
        title: 'Event candidates',
        items: [{ title: 'Review meeting', evidence: 'Monday 10:00' }]
      },
      {
        kind: 'persona',
        title: 'Persona candidates',
        items: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }]
      },
      {
        kind: 'organization',
        title: 'Organization candidates',
        items: [{ title: 'Acme Corp', evidence: 'acme.example' }]
      },
      {
        kind: 'document',
        title: 'Document candidates',
        items: [{ title: 'MSA attachment', evidence: 'attached MSA' }]
      },
      {
        kind: 'agreement',
        title: 'Agreement candidates',
        items: [{ title: 'NDA', evidence: 'review NDA' }]
      }
    ])
  })

  it('extracts message labels and snooze metadata safely', () => {
    expect(communicationMessageLabelsFromMetadata({
      labels: ['finance', ' urgent ', 42, '', 'finance']
    })).toEqual(['finance', 'urgent'])

    expect(communicationMessageLabelsFromMetadata({ labels: 'finance' })).toEqual([])
    expect(communicationMessageSnoozeUntilFromMetadata({ snooze_until: '2026-06-20T10:00:00Z' }))
      .toBe('2026-06-20T10:00:00Z')
    expect(communicationMessageSnoozeUntilFromMetadata({ snooze_until: 42 })).toBeNull()
  })

  it('builds review sections for extracted mail task and note candidates', () => {
    const sections = communicationExtractionSectionsFromInsight({
      ...emptyCommunicationMessageInsight('msg-1'),
      tasks: [
        {
          title: 'Send signed amendment',
          due_date: '2026-06-20',
          assignee: 'Ada',
          priority: 'high',
          source: 'Please send the signed amendment by Friday.'
        }
      ],
      notes: [
        {
          title: 'Commercial terms',
          content: 'Discount applies after renewal.',
          tags: ['contract', 'renewal'],
          source: 'Renewal clause'
        }
      ]
    })

    expect(sections).toEqual([
      {
        kind: 'task',
        title: 'Task candidates',
        items: [
          {
            title: 'Send signed amendment',
            meta: ['Due 2026-06-20', 'Assignee Ada', 'Priority high'],
            body: 'Please send the signed amendment by Friday.'
          }
        ]
      },
      {
        kind: 'note',
        title: 'Note candidates',
        items: [
          {
            title: 'Commercial terms',
            meta: ['contract', 'renewal'],
            body: 'Discount applies after renewal.'
          }
        ]
      }
    ])

    expect(communicationExtractionSectionsFromInsight(null)).toEqual([])
  })

  it('builds compose form models for new, reply and persisted draft flows', () => {
    expect(newComposeForm('account-1', 'draft-new')).toMatchObject({
      mode: 'compose',
      draftId: 'draft-new',
      accountId: 'account-1',
      subject: ''
    })
    expect(replyComposeForm(message(), 'fallback-account', 'draft-reply')).toMatchObject({
      mode: 'reply',
      draftId: 'draft-reply',
      accountId: 'account-1',
      toText: 'alice@example.com',
      subject: 'Re: Quarterly update',
      inReplyTo: 'provider-1'
    })
    expect(replyAllComposeForm(message({ recipients: ['owner@example.com', 'team@example.com'] }), 'fallback-account', 'draft-reply-all')).toMatchObject({
      mode: 'reply',
      draftId: 'draft-reply-all',
      toText: 'alice@example.com',
      ccText: 'owner@example.com, team@example.com',
      subject: 'Re: Quarterly update',
      inReplyTo: 'provider-1'
    })
    expect(forwardComposeForm(message(), 'fallback-account', 'draft-forward')).toMatchObject({
      mode: 'forward',
      draftId: 'draft-forward',
      toText: '',
      subject: 'Fwd: Quarterly update'
    })
    expect(draftToComposeForm(draft())).toMatchObject({
      draftId: 'draft-1',
      toText: 'to@example.com',
      ccText: 'cc@example.com',
      bccText: 'bcc@example.com',
      body: 'Draft body'
    })
  })

  it('builds quoted rich compose models for thread message replies', () => {
    const form = threadReplyComposeForm(
      threadMessage(),
      'fallback-account',
      'draft-thread-reply',
      '<p>Inline draft</p>'
    )

    expect(form).toMatchObject({
      mode: 'reply',
      draftId: 'draft-thread-reply',
      accountId: 'account-1',
      toText: 'Ada <ada@example.com>',
      subject: 'Re: Quarterly update',
      bodyFormat: 'html',
      inReplyTo: 'provider-thread-1'
    })
    expect(form.body).toContain('On 2026-06-15T10:00:00Z, Ada <ada@example.com> wrote:')
    expect(form.body).toContain('Inline draft')
    expect(form.body).toContain('> Line one')
    expect(form.bodyHtml ?? '').toContain('<p>Inline draft</p>')
    expect(form.bodyHtml ?? '').toContain('<blockquote')
    expect(form.bodyHtml ?? '').toContain('Line one<br>Line two with &lt;angle&gt;')
  })

  it('converts compose models into provider-write send requests', () => {
    const form = threadReplyComposeForm(
      threadMessage(),
      'fallback-account',
      'draft-thread-reply',
      '<p>Inline draft</p>'
    )
    const request = composeFormToSendRequest(form)

    expect(request).toMatchObject({
      account_id: 'account-1',
      to: ['ada@example.com'],
      subject: 'Re: Quarterly update',
      draft_id: 'draft-thread-reply',
      in_reply_to: 'provider-thread-1',
      confirmed_provider_write: true
    })
    expect(request.body_html).toContain('<p>Inline draft</p>')
  })
})
