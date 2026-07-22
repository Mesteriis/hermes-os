import type {
  MailSensitiveForwardingPolicy,
  MailSensitiveForwardingPolicyInput
} from '../../../shared/mailSync/types'

export function newSensitiveForwardingPolicyDraft(
  deliveryAccountId: string
): MailSensitiveForwardingPolicyInput {
  return {
    delivery_account_id: deliveryAccountId,
    name: 'Sensitive mail notification',
    enabled: false,
    include_message_body: false,
    include_attachments: false,
    fixed_recipients: [],
    minimum_severity: 'high',
    subject_template: 'Sensitive mail alert: {{severity}}',
    body_template: 'Hermes detected a sensitive message. Reference: {{message_id}}\n{{attachment_notice}}',
    max_sends_per_hour: 3,
    quiet_hours: {},
    expires_at: null
  }
}

export function sensitiveForwardingPolicyInput(
  policy: MailSensitiveForwardingPolicy
): MailSensitiveForwardingPolicyInput {
  return {
    policy_id: policy.policy_id,
    delivery_account_id: policy.delivery_account_id,
    name: policy.name,
    enabled: policy.enabled,
    include_message_body: policy.include_message_body,
    include_attachments: policy.include_attachments,
    fixed_recipients: [...policy.fixed_recipients],
    minimum_severity: policy.minimum_severity,
    subject_template: policy.subject_template,
    body_template: policy.body_template,
    max_sends_per_hour: policy.max_sends_per_hour,
    quiet_hours: { ...policy.quiet_hours },
    expires_at: policy.expires_at
  }
}
