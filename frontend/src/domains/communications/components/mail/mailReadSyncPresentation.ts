import type { MailReadSyncStatus } from '../../types/mailSync'

export type MailReadSyncPresentation = {
  icon: string
  label: string
  tone: 'info' | 'warning' | 'danger' | 'muted'
}

export function mailReadSyncPresentation(
  status?: MailReadSyncStatus
): MailReadSyncPresentation | null {
  switch (status) {
    case 'queued':
      return { icon: 'tabler:clock', label: 'Read state queued for provider', tone: 'info' }
    case 'syncing':
      return { icon: 'tabler:loader-2', label: 'Synchronizing read state with provider', tone: 'info' }
    case 'retrying':
      return { icon: 'tabler:refresh-alert', label: 'Provider sync will retry', tone: 'warning' }
    case 'failed':
      return { icon: 'tabler:cloud-off', label: 'Provider sync failed; local state is preserved', tone: 'danger' }
    case 'awaiting_provider':
      return { icon: 'tabler:cloud-check', label: 'Provider accepted; awaiting reconciliation', tone: 'info' }
    case 'superseded':
      return { icon: 'tabler:history', label: 'Older read-state intent was superseded', tone: 'muted' }
    default:
      return null
  }
}
