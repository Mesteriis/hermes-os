import { messageTime } from '../stores/communications'
import type {
  CommunicationMessageDetailItem,
  CommunicationMessageSummary,
} from '../types/communications'
import type {
  MailInspectorSemanticFact,
  MailInspectorTopic,
} from '../components/mail/mailInspector'

type InspectableMessage = CommunicationMessageDetailItem | CommunicationMessageSummary

export function mailInspectorTopics(message: InspectableMessage): MailInspectorTopic[] {
  return message.ai_category
    ? [{ id: 'ai-category', label: message.ai_category, tone: 'info' }]
    : []
}

export function mailInspectorFacts(
  message: InspectableMessage,
  attachmentTotal: number
): MailInspectorSemanticFact[] {
  const facts: MailInspectorSemanticFact[] = [
    { id: 'account', label: 'Account', value: message.account_id },
    { id: 'workflow', label: 'Workflow', value: message.workflow_state },
    { id: 'delivery', label: 'Delivery', value: message.delivery_state },
    { id: 'attachments', label: 'Attachments', value: String(attachmentTotal) },
  ]

  if (message.importance_score !== null) {
    facts.push({
      id: 'importance',
      label: 'Importance',
      value: String(message.importance_score),
      tone: message.importance_score >= 75 ? 'warning' : 'neutral',
    })
  }
  if (message.occurred_at) {
    facts.push({ id: 'occurred-at', label: 'Observed', value: messageTime(message.occurred_at) })
  }
  return facts
}
