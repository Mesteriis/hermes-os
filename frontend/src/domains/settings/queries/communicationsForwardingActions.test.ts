import { describe, expect, it, vi } from 'vitest'
import {
  deleteSensitiveForwardingPolicyAction,
  saveSensitiveForwardingPolicyAction
} from './communicationsForwardingActions'
import type { MailSensitiveForwardingPolicyInput } from '../../../shared/mailSync/types'

describe('communications forwarding actions', () => {
  it('rejects incomplete forwarding drafts before mutation', async () => {
    const dependencies = dependenciesFor()
    const draft = forwardingDraft({ fixed_recipients: [] })

    const result = await saveSensitiveForwardingPolicyAction('account-1', draft, dependencies)

    expect(result).toBeNull()
    expect(dependencies.setError).toHaveBeenCalledWith(
      'Sensitive forwarding requires a delivery account, name and fixed recipients.'
    )
    expect(dependencies.upsertPolicy).not.toHaveBeenCalled()
  })

  it('returns saved policies and reports success', async () => {
    const dependencies = dependenciesFor()
    const draft = forwardingDraft({ fixed_recipients: ['owner@example.com'] })
    dependencies.upsertPolicy.mockResolvedValue([{ policy_id: 'policy-1' }])

    const result = await saveSensitiveForwardingPolicyAction('account-1', draft, dependencies)

    expect(result).toEqual([{ policy_id: 'policy-1' }])
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Sensitive forwarding policy saved')
  })

  it('deletes the selected policy and reports success', async () => {
    const dependencies = dependenciesFor()

    await expect(
      deleteSensitiveForwardingPolicyAction('account-1', 'policy-1', dependencies)
    ).resolves.toBe(true)

    expect(dependencies.deletePolicy).toHaveBeenCalledWith({
      accountId: 'account-1',
      policyId: 'policy-1'
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Sensitive forwarding policy deleted')
  })
})

function dependenciesFor() {
  return {
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    upsertPolicy: vi.fn(),
    deletePolicy: vi.fn().mockResolvedValue(undefined)
  }
}

function forwardingDraft(
  overrides: Partial<MailSensitiveForwardingPolicyInput>
): MailSensitiveForwardingPolicyInput {
  return {
    delivery_account_id: 'account-1',
    name: 'Sensitive mail notification',
    enabled: false,
    include_message_body: false,
    include_attachments: false,
    fixed_recipients: ['owner@example.com'],
    minimum_severity: 'high',
    subject_template: 'Sensitive mail alert: {{severity}}',
    body_template: 'Hermes detected a sensitive message.',
    max_sends_per_hour: 3,
    quiet_hours: {},
    expires_at: null,
    ...overrides
  }
}
