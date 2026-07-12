import { describe, expect, it } from 'vitest'

import { mailItem } from './communicationMailWorkspaceModels'

describe('mailItem', () => {
  it('renders the provider-synchronized star separately from importance', () => {
    const item = mailItem({
      message_id: 'message-1',
      raw_record_id: 'raw-1',
      account_id: 'account-1',
      provider_record_id: 'provider-1',
      subject: 'Starred mail',
      sender: 'sender@example.test',
      recipients: [],
      body_text_preview: '',
      occurred_at: null,
      projected_at: '2026-07-12T00:00:00Z',
      channel_kind: 'email',
      conversation_id: null,
      sender_display_name: null,
      delivery_state: 'received',
      workflow_state: 'new',
      importance_score: 0,
      ai_category: null,
      ai_summary: null,
      ai_summary_generated_at: null,
      message_metadata: { starred: true },
      attachment_count: 0,
      local_state: 'active',
      local_state_changed_at: null,
    }, '')

    expect(item.markers).toContain('starred')
    expect(item.markers).not.toContain('important')
  })
})
