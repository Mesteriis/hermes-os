import { describe, expect, it } from 'vitest'
import {
  newSensitiveForwardingPolicyDraft,
  sensitiveForwardingPolicyInput
} from './communicationsForwardingPresentation'

describe('communications forwarding presentation', () => {
  it('creates a draft scoped to the selected delivery account', () => {
    const draft = newSensitiveForwardingPolicyDraft('account-1')

    expect(draft.delivery_account_id).toBe('account-1')
    expect(draft.fixed_recipients).toEqual([])
    expect(draft.quiet_hours).toEqual({})
  })

  it('copies persisted policy fields into an editable input', () => {
    const input = sensitiveForwardingPolicyInput({
      policy_id: 'policy-1',
      source_account_id: 'source-1',
      delivery_account_id: 'delivery-1',
      name: 'Sensitive mail notification',
      enabled: true,
      include_message_body: true,
      include_attachments: false,
      fixed_recipients: ['owner@example.com'],
      minimum_severity: 'high',
      subject_template: 'Subject',
      body_template: 'Body',
      max_sends_per_hour: 3,
      quiet_hours: { timezone: 'UTC' },
      expires_at: null,
      updated_at: '2026-07-21T00:00:00Z'
    })

    expect(input).toEqual({
      policy_id: 'policy-1',
      delivery_account_id: 'delivery-1',
      name: 'Sensitive mail notification',
      enabled: true,
      include_message_body: true,
      include_attachments: false,
      fixed_recipients: ['owner@example.com'],
      minimum_severity: 'high',
      subject_template: 'Subject',
      body_template: 'Body',
      max_sends_per_hour: 3,
      quiet_hours: { timezone: 'UTC' },
      expires_at: null
    })
  })
})
