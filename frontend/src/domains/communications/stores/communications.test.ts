import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  communicationSectionWorkflowState,
  communicationWorkflowStateSectionId,
  useCommunicationsStore,
} from './communications'
import type {
  CommunicationMessageSummary,
  MailSyncStatus,
} from '../types/communications'

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('communication section workflow mapping', () => {
  it('maps UI section ids to backend workflow states', () => {
    expect(communicationSectionWorkflowState('unified')).toBe('')
    expect(communicationSectionWorkflowState('inbox')).toBe('new')
    expect(communicationSectionWorkflowState('needs_reply')).toBe(
      'needs_action'
    )
    expect(communicationSectionWorkflowState('waiting')).toBe('waiting')
    expect(communicationSectionWorkflowState('done')).toBe('done')
    expect(communicationSectionWorkflowState('archived')).toBe('archived')
  })

  it('maps backend workflow states back to UI section ids', () => {
    expect(communicationWorkflowStateSectionId('')).toBe('unified')
    expect(communicationWorkflowStateSectionId('new')).toBe('inbox')
    expect(communicationWorkflowStateSectionId('needs_action')).toBe(
      'needs_reply'
    )
    expect(communicationWorkflowStateSectionId('waiting')).toBe('waiting')
    expect(communicationWorkflowStateSectionId('done')).toBe('done')
    expect(communicationWorkflowStateSectionId('archived')).toBe('archived')
  })
})

describe('communications multi-select state', () => {
  it('toggles selected message ids and clears selections', () => {
    const store = useCommunicationsStore()

    store.toggleMessageSelection('msg-1')
    store.toggleMessageSelection('msg-2')
    store.toggleMessageSelection('msg-1')

    expect(store.selectedMessageIds).toEqual(['msg-2'])
    expect(store.selectedMessageIdSet.has('msg-2')).toBe(true)

    store.clearMessageSelection()

    expect(store.selectedMessageIds).toEqual([])
    expect(store.selectedMessageIdSet.size).toBe(0)
  })

  it('selects a visible message range from the last selection anchor', () => {
    const store = useCommunicationsStore()
    store.setMessages([
      messageSummary('msg-1'),
      messageSummary('msg-2'),
      messageSummary('msg-3'),
      messageSummary('msg-4'),
    ])

    store.toggleMessageSelection('msg-1')
    store.toggleMessageSelection('msg-4', true)

    expect(store.selectedMessageIds).toEqual([
      'msg-1',
      'msg-2',
      'msg-3',
      'msg-4',
    ])
  })

  it('selects the current visible message id set for keyboard select-all', () => {
    const store = useCommunicationsStore()
    store.setMessages([
      messageSummary('msg-1'),
      messageSummary('msg-2'),
      messageSummary('msg-3'),
    ])

    store.selectVisibleMessages(['msg-2', 'msg-1', 'msg-2', ''])

    expect(store.selectedMessageIds).toEqual(['msg-2', 'msg-1'])

    store.toggleMessageSelection('msg-3', true)

    expect(store.selectedMessageIds).toEqual(['msg-1', 'msg-2', 'msg-3'])
  })
})

describe('communications mail account selection', () => {
  it('keeps the all-account selection when sync statuses refresh without an explicit route account', () => {
    const store = useCommunicationsStore()

    store.setMailSyncStatuses([
      mailSyncStatus('account-1'),
      mailSyncStatus('account-2'),
    ])

    expect(store.selectedMailAccountId).toBe('')
  })

  it('preserves an explicit selected account when sync statuses refresh', () => {
    const store = useCommunicationsStore()
    store.setSelectedMailAccountId('account-2')

    store.setMailSyncStatuses([
      mailSyncStatus('account-1'),
      mailSyncStatus('account-2'),
    ])

    expect(store.selectedMailAccountId).toBe('account-2')
  })

  it('clears the selected message context when the selected account changes', () => {
    const store = useCommunicationsStore()
    store.setMessages([messageSummary('msg-1')])
    store.selectMessage(0)
    store.setMessageDetail({ message: detailedMessage('msg-1'), attachments: [] })
    store.toggleMessageSelection('msg-1')

    store.setSelectedMailAccountId('account-2')

    expect(store.selectedMailAccountId).toBe('account-2')
    expect(store.selectedCommunicationMessageId).toBe('')
    expect(store.selectedCommunicationDetail).toBeNull()
    expect(store.selectedMessageIds).toEqual([])
  })
})

function messageSummary(messageId: string): CommunicationMessageSummary {
  return {
    message_id: messageId,
    raw_record_id: `raw-${messageId}`,
    account_id: 'account-1',
    provider_record_id: `provider-${messageId}`,
    subject: `Subject ${messageId}`,
    sender: 'sender@example.com',
    recipients: ['recipient@example.com'],
    body_text_preview: 'Preview',
    occurred_at: null,
    projected_at: '2026-06-14T00:00:00Z',
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
  }
}

function detailedMessage(messageId: string) {
  return {
    ...messageSummary(messageId),
    body_text: 'Body',
    body_html: null,
    local_state_reason: null,
  }
}

function mailSyncStatus(accountId: string): MailSyncStatus {
  return {
    account_id: accountId,
    status: 'idle',
    phase: 'idle',
    progress_mode: 'none',
    progress_percent: null,
    processed_messages: 0,
    estimated_total_messages: null,
    current_batch_size: 0,
    last_started_at: null,
    last_updated_at: null,
    last_completed_at: null,
    next_run_at: null,
    last_error_code: null,
    last_error_message: null,
    last_fetched_messages: 0,
    last_projected_messages: 0,
    last_upserted_persons: 0,
    last_upserted_organizations: 0,
  }
}
