import type {
  MailListSearchBuilderField,
  MailListSearchBuilderFieldGroup,
  MailListSearchBuilderFieldItem,
  MailListSearchBuilderOperator,
  MailListSearchBuilderToggleItem
} from './mailSearchBuilder'

export const mailListSearchMatchModeItems: readonly MailListSearchBuilderToggleItem[] = [
  { value: 'all', label: 'all' },
  { value: 'any', label: 'any' }
]

export const mailListSearchOperatorItems: readonly MailListSearchBuilderToggleItem[] = [
  { value: 'contains', label: 'contains' },
  { value: 'equals', label: 'equals' },
  { value: 'is', label: 'is' },
  { value: 'is_not', label: 'is not' },
  { value: 'has', label: 'has' },
  { value: 'without', label: 'without' },
  { value: 'gte', label: '>=' },
  { value: 'lte', label: '<=' }
]

export const mailListSearchFieldGroups: readonly MailListSearchBuilderFieldGroup[] = [
  {
    id: 'text',
    label: 'Text',
    fields: [
      fieldItem('from', 'from', ['contains', 'equals'], 'Sender name or address'),
      fieldItem('recipients', 'to/cc', ['contains', 'equals'], 'Recipient address'),
      fieldItem('subject', 'subject', ['contains', 'equals'], 'Subject text'),
      fieldItem('body', 'body', ['contains', 'equals'], 'Message body'),
      fieldItem('all', 'all text', ['contains', 'equals'], 'Any projected text'),
      fieldItem('provider', 'provider id', ['contains', 'equals'], 'Provider record id')
    ]
  },
  {
    id: 'mail',
    label: 'Mail attrs',
    fields: [
      fieldItem('account', 'account', ['contains', 'equals'], 'Account label'),
      fieldItem('mailbox', 'mailbox', ['contains', 'equals'], 'Inbox, Spam, Archive'),
      fieldItem('workflow', 'status', ['is', 'is_not'], 'Workflow state', workflowPresets()),
      fieldItem('local', 'local state', ['is', 'is_not'], 'Local state', localStatePresets()),
      fieldItem('label', 'label', ['contains', 'equals'], 'Message label'),
      fieldItem('attachment', 'attachments', ['has', 'without'], 'Any attachment', yesNoPresets()),
      fieldItem('marker', 'marker', ['is', 'is_not'], 'Spam, phishing, important', markerPresets()),
      fieldItem('unread', 'unread', ['has', 'without'], 'Unread count', yesNoPresets()),
      fieldItem('action', 'action', ['has', 'without'], 'Open owner action', yesNoPresets()),
      fieldItem('delivery', 'delivery', ['contains', 'equals'], 'Delivery state'),
      fieldItem('importance', 'importance', ['is', 'gte', 'lte'], 'Score or band', importancePresets()),
      fieldItem('ai_category', 'AI category', ['contains', 'equals'], 'AI category')
    ]
  },
  {
    id: 'hermes',
    label: 'Hermes',
    fields: [
      fieldItem('entity', 'entity', ['contains', 'equals'], 'Entity kind or title', entityPresets()),
      fieldItem('task', 'task candidate', ['has', 'without'], 'Task candidate', yesNoPresets()),
      fieldItem('decision', 'decision', ['has', 'without'], 'Decision candidate', yesNoPresets()),
      fieldItem('document', 'document', ['has', 'without'], 'Document candidate', yesNoPresets()),
      fieldItem('deadline', 'deadline', ['has', 'without'], 'Deadline signal', yesNoPresets()),
      fieldItem('risk', 'risk', ['has', 'without'], 'Risk signal', yesNoPresets()),
      fieldItem('evidence', 'evidence', ['contains', 'equals'], 'Evidence kind')
    ]
  }
]

export function mailListSearchFieldItems(): readonly MailListSearchBuilderFieldItem[] {
  return mailListSearchFieldGroups.flatMap((group) => group.fields)
}

export function mailListSearchFieldItem(field: MailListSearchBuilderField): MailListSearchBuilderFieldItem {
  return mailListSearchFieldItems().find((item) => item.value === field) ?? mailListSearchFieldItems()[0]!
}

function fieldItem(
  value: MailListSearchBuilderField,
  label: string,
  operators: readonly MailListSearchBuilderOperator[],
  placeholder: string,
  presets?: readonly MailListSearchBuilderToggleItem[]
): MailListSearchBuilderFieldItem {
  return { value, label, operators, placeholder, presets }
}

function workflowPresets(): readonly MailListSearchBuilderToggleItem[] {
  return labelledToggleItems([
    ['new', 'new'],
    ['needs_action', 'needs action'],
    ['waiting', 'waiting'],
    ['reviewed', 'reviewed'],
    ['spam', 'spam'],
    ['archived', 'archived']
  ])
}

function localStatePresets(): readonly MailListSearchBuilderToggleItem[] {
  return toggleItems(['active', 'trash', 'all'])
}

function yesNoPresets(): readonly MailListSearchBuilderToggleItem[] {
  return toggleItems(['yes', 'no'])
}

function markerPresets(): readonly MailListSearchBuilderToggleItem[] {
  return toggleItems(['spam', 'phishing', 'important', 'blocked', 'archived'])
}

function importancePresets(): readonly MailListSearchBuilderToggleItem[] {
  return labelledToggleItems([
    ['high', 'high'],
    ['medium', 'medium'],
    ['low', 'low'],
    ['75', '>= 75']
  ])
}

function entityPresets(): readonly MailListSearchBuilderToggleItem[] {
  return labelledToggleItems([
    ['person', 'person'],
    ['organization', 'org'],
    ['project', 'project'],
    ['document', 'document'],
    ['decision', 'decision']
  ])
}

function toggleItems(values: readonly string[]): readonly MailListSearchBuilderToggleItem[] {
  return values.map((value) => ({ value, label: value }))
}

function labelledToggleItems(items: readonly (readonly [string, string])[]): readonly MailListSearchBuilderToggleItem[] {
  return items.map(([value, label]) => ({ value, label }))
}
