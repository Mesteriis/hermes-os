import type { MailProviderSemanticRole } from '../../../shared/mailSync/providerResources'
import type { MailSensitiveForwardingPolicyInput } from '../../../shared/mailSync/types'

export const semanticRoles: Array<{ value: MailProviderSemanticRole; label: string }> = [
  { value: 'inbox', label: 'Inbox' },
  { value: 'sent', label: 'Sent' },
  { value: 'drafts', label: 'Drafts' },
  { value: 'archive', label: 'Archive' },
  { value: 'trash', label: 'Trash' },
  { value: 'junk', label: 'Junk' },
  { value: 'all', label: 'All mail' },
  { value: 'flagged', label: 'Flagged' },
  { value: 'important', label: 'Important' },
  { value: 'user', label: 'User label' }
]

export function parseSemanticRole(value: string): MailProviderSemanticRole | null {
  return semanticRoles.find((role) => role.value === value)?.value ?? null
}

export const forwardingSeverities: readonly MailSensitiveForwardingPolicyInput['minimum_severity'][] = [
  'low',
  'medium',
  'high',
  'critical',
]

export function parseForwardingSeverity(
  value: string
): MailSensitiveForwardingPolicyInput['minimum_severity'] {
  return forwardingSeverities.find((severity) => severity === value) ?? 'high'
}

export function nullableLocalFolder(value: string): string | null {
  return value || null
}

export function commandStatusCount(
  counts: Array<{ status: string; count: number }> | undefined,
  status: string
): number {
  return counts?.find((item) => item.status === status)?.count ?? 0
}

export function formatCommunicationTimestamp(value: string | null): string {
  if (!value) return '—'
  const date = new Date(value)
  return Number.isFinite(date.getTime()) ? date.toLocaleString() : value
}

export function communicationMappingSourceLabel(source: string, t: (key: string) => string): string {
  return source === 'manual' ? t('Manual override') : t('Discovered')
}
