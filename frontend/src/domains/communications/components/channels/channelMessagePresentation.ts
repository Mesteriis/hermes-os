import type { CommunicationConversationMessageModel } from '../communicationDomainElements'

type ChannelMessage = Pick<CommunicationConversationMessageModel, 'direction' | 'author' | 'timestamp' | 'meta'>

export function channelMessageAuthor(message: ChannelMessage): string | undefined {
  return message.direction === 'system' ? undefined : message.author
}

export function channelMessageTimestamp(message: ChannelMessage): string | undefined {
  return message.direction === 'system' ? undefined : message.timestamp
}

export function channelMessageMeta(message: ChannelMessage): string | undefined {
  return message.direction === 'system' ? undefined : message.meta
}
