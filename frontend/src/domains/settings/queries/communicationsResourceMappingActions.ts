import type {
  MailProviderResource,
  MailProviderResourceMappingUpdate
} from '../../../shared/mailSync/providerResources'

interface ResourceMappingActionDependencies {
  clearMessages: () => void
  setActionMessage: (message: string) => void
  setError: (message: string) => void
  updateProviderResourceMapping: (request: {
    accountId: string
    mappingId: string
    update: MailProviderResourceMappingUpdate
  }) => Promise<unknown>
}

export async function saveProviderResourceMappingAction(
  accountId: string | null,
  resource: MailProviderResource,
  update: MailProviderResourceMappingUpdate,
  dependencies: ResourceMappingActionDependencies
): Promise<void> {
  if (!accountId || !resource.writable) return

  dependencies.clearMessages()
  try {
    await dependencies.updateProviderResourceMapping({
      accountId,
      mappingId: resource.mapping_id,
      update
    })
    dependencies.setActionMessage('Mail provider mapping saved')
  } catch (error) {
    dependencies.setError(
      error instanceof Error ? error.message : 'Mail provider mapping update failed'
    )
  }
}
