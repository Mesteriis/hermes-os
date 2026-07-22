import type { MailContentEgressSettings } from '../../../shared/mailSync/types'

interface ContentEgressActionDependencies {
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  updateContentEgressSettings: (request: {
    accountId: string
    settings: Partial<MailContentEgressSettings>
  }) => Promise<unknown>
}

export async function updateMailContentEgress(
  accountId: string | null,
  permission: keyof MailContentEgressSettings,
  enabled: boolean,
  dependencies: ContentEgressActionDependencies
): Promise<void> {
  if (!accountId) return

  dependencies.clearMessages()
  try {
    await dependencies.updateContentEgressSettings({
      accountId,
      settings: { [permission]: enabled }
    })
    dependencies.setActionMessage('Mail content access preference saved')
  } catch (error) {
    dependencies.setError(
      error instanceof Error ? error.message : 'Mail content access preference update failed'
    )
  }
}
