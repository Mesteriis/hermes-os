import { describe, expect, it } from 'vitest'
import {
  messengerItemsForView,
  messengerConversationIsEmpty,
  messengerConversationIsTelegramEmpty,
  messengerListItemHasSecondarySignals,
  messengerListViewOptions,
  type MessengerConversationModel,
  type MessengerListItemModel,
} from './messengerElements'

const items: MessengerListItemModel[] = [
  {
    id: 'unread', channelKind: 'telegram', conversationKind: 'direct', title: 'Unread', subtitle: '', preview: '', timestampLabel: '', workflowState: 'reviewed', unreadCount: 2,
  },
  {
    id: 'pinned', channelKind: 'telegram', conversationKind: 'group', title: 'Pinned', subtitle: '', preview: '', timestampLabel: '', workflowState: 'reviewed', pinned: true,
  },
  {
    id: 'muted', channelKind: 'telegram', conversationKind: 'group', title: 'Muted', subtitle: '', preview: '', timestampLabel: '', workflowState: 'muted', muted: true,
  },
  {
    id: 'archived', channelKind: 'telegram', conversationKind: 'channel', title: 'Archived', subtitle: '', preview: '', timestampLabel: '', workflowState: 'archived',
  },
]

describe('messenger saved filters', () => {
  it('offers daily dialog-state filters', () => {
    const options = messengerListViewOptions(items, (value) => value)
    const savedFilters = options.find((option) => option.value === 'saved-filters')

    expect(savedFilters?.children?.map((option) => option.value)).toEqual(expect.arrayContaining([
      'messenger-filter:unread',
      'messenger-filter:pinned',
      'messenger-filter:muted',
      'messenger-filter:archived',
    ]))
  })

  it('filters the projected list by provider dialog state', () => {
    expect(messengerItemsForView(items, 'messenger-filter:unread').map((item) => item.id)).toEqual(['unread'])
    expect(messengerItemsForView(items, 'messenger-filter:pinned').map((item) => item.id)).toEqual(['pinned'])
    expect(messengerItemsForView(items, 'messenger-filter:muted').map((item) => item.id)).toEqual(['muted'])
    expect(messengerItemsForView(items, 'messenger-filter:archived').map((item) => item.id)).toEqual(['archived'])
  })
})

describe('messenger presentation selectors', () => {
  it('projects secondary signals and Telegram empty state', () => {
  expect(messengerListItemHasSecondarySignals({ ...items[0], attachmentCount: 1 })).toBe(true)
  expect(messengerListItemHasSecondarySignals({ ...items[0], unreadCount: undefined })).toBe(false)
  expect(messengerConversationIsTelegramEmpty(emptyTelegramConversation())).toBe(true)
  expect(messengerConversationIsEmpty(emptyWhatsAppConversation())).toBe(true)
  })
})

function emptyTelegramConversation(): MessengerConversationModel {
  return {
    id: 'telegram:empty', channelKind: 'telegram', kind: 'direct', title: '', subtitle: '',
    workflowState: 'reviewed', participantsLabel: '', facts: [], messages: [], draftPreview: '',
  }
}

function emptyWhatsAppConversation(): MessengerConversationModel {
  return {
    id: 'whatsapp:empty', channelKind: 'whatsapp', kind: 'direct', title: '', subtitle: '',
    workflowState: 'reviewed', participantsLabel: '', facts: [], messages: [], draftPreview: '',
  }
}
