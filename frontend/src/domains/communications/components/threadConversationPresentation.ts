import type { ThreadMessage } from '../types/communications'
import { splitThreadMessageBody } from './threadMessageBody'

export function defaultExpandedThreadMessageIds(messages: ThreadMessage[]): Set<string> {
  const latestMessage = messages.at(-1)
  return latestMessage ? new Set([latestMessage.message_id]) : new Set()
}

export function hasQuotedThreadMessages(messages: ThreadMessage[]): boolean {
  return messages.some((message) => splitThreadMessageBody(message.body_text).quotedText.length > 0)
}

export function summarizeThreadExpansion(
  messages: ThreadMessage[],
  expandedMessageIds: ReadonlySet<string>
): {
  expandedCount: number
  canExpandAll: boolean
  canCollapseAll: boolean
} {
  return {
    expandedCount: expandedMessageIds.size,
    canExpandAll: messages.some((message) => !expandedMessageIds.has(message.message_id)),
    canCollapseAll: expandedMessageIds.size > 0
  }
}
