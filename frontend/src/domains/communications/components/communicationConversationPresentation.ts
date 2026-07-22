export function communicationConversationActiveMessage<T>(
  messages: readonly T[]
): T | undefined {
  return messages[messages.length - 1]
}
