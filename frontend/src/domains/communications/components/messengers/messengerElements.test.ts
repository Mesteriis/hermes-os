import { describe, expect, it } from 'vitest'
import {
  messengerItemsForView,
  messengerListViewOptions,
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
