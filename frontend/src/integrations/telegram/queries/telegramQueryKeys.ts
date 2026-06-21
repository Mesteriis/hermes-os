export const telegramQueryKeys = {
  capabilities: ['integrations', 'telegram', 'capabilities'] as const,
  accountCapabilities: ['integrations', 'telegram', 'account-capabilities'] as const,
  accounts: ['integrations', 'telegram', 'accounts'] as const,
  chats: ['integrations', 'telegram', 'provider-conversations'] as const,
  folders: ['integrations', 'telegram', 'provider-folders'] as const,
  chatDetail: ['integrations', 'telegram', 'provider-conversation-detail'] as const,
  chatMembers: ['integrations', 'telegram', 'provider-conversation-members'] as const,
  runtime: ['integrations', 'telegram', 'runtime'] as const,
  calls: ['integrations', 'telegram', 'provider-calls'] as const,
  callTranscript: ['integrations', 'telegram', 'provider-call-transcript'] as const,
}
