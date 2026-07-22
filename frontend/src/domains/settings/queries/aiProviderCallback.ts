export interface AiProviderCallbackMessage {
  providerId: string | null
}

export function parseAiProviderCallbackMessage(
  data: unknown
): AiProviderCallbackMessage | null {
  if (
    typeof data !== 'object' ||
    data === null ||
    !('type' in data) ||
    data.type !== 'hermes:ai-provider-connected'
  ) {
    return null
  }
  return {
    providerId: 'providerId' in data && typeof data.providerId === 'string'
      ? data.providerId
      : null
  }
}
