import type {
  MailSensitiveForwardingPolicy,
  MailSensitiveForwardingPolicyInput
} from '../../../shared/mailSync/types'

interface ForwardingActionDependencies {
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  upsertPolicy: (request: {
    accountId: string
    policy: MailSensitiveForwardingPolicyInput
  }) => Promise<MailSensitiveForwardingPolicy[]>
  deletePolicy: (request: { accountId: string; policyId: string }) => Promise<void>
}

export async function saveSensitiveForwardingPolicyAction(
  accountId: string | null,
  draft: MailSensitiveForwardingPolicyInput,
  dependencies: ForwardingActionDependencies
): Promise<MailSensitiveForwardingPolicy[] | null> {
  if (!accountId) return null
  if (!draft.delivery_account_id || !draft.name.trim() || draft.fixed_recipients.length === 0) {
    dependencies.setError('Sensitive forwarding requires a delivery account, name and fixed recipients.')
    return null
  }
  if (!Number.isInteger(draft.max_sends_per_hour) || draft.max_sends_per_hour < 1) {
    dependencies.setError('Sensitive forwarding rate limit must be a positive integer.')
    return null
  }

  dependencies.clearMessages()
  try {
    const policies = await dependencies.upsertPolicy({ accountId, policy: draft })
    dependencies.setActionMessage('Sensitive forwarding policy saved')
    return policies
  } catch (error) {
    dependencies.setError(
      error instanceof Error ? error.message : 'Sensitive forwarding policy update failed'
    )
    return null
  }
}

export async function deleteSensitiveForwardingPolicyAction(
  accountId: string | null,
  policyId: string | null,
  dependencies: ForwardingActionDependencies
): Promise<boolean> {
  if (!accountId || !policyId) return false

  dependencies.clearMessages()
  try {
    await dependencies.deletePolicy({ accountId, policyId })
    dependencies.setActionMessage('Sensitive forwarding policy deleted')
    return true
  } catch (error) {
    dependencies.setError(
      error instanceof Error ? error.message : 'Sensitive forwarding policy deletion failed'
    )
    return false
  }
}
