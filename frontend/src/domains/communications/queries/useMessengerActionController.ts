import type { MessengerConversationRuntimeAction } from '@/shared/communications/types/messengerRuntimeActions'

interface MessengerActionControllerProps {
  isActionRunning?: boolean
}

export interface MessengerActionControllerActions {
  conversationAction: (action: MessengerConversationRuntimeAction) => void
  toggleInspector: () => void
}

export function useMessengerActionController(
  _props: Readonly<MessengerActionControllerProps>,
  actions: MessengerActionControllerActions,
) {
  void _props

  function handleMarkRead(): void {
    actions.conversationAction('mark_read')
  }

  function handleMarkUnread(): void {
    actions.conversationAction('mark_unread')
  }

  function handlePin(): void {
    actions.conversationAction('pin')
  }

  function handleUnpin(): void {
    actions.conversationAction('unpin')
  }

  function handleMute(): void {
    actions.conversationAction('mute')
  }

  function handleUnmute(): void {
    actions.conversationAction('unmute')
  }

  function handleArchive(): void {
    actions.conversationAction('archive')
  }

  function handleUnarchive(): void {
    actions.conversationAction('unarchive')
  }

  function handleSyncLatest(): void {
    actions.conversationAction('sync_latest')
  }

  function handleSyncOlder(): void {
    actions.conversationAction('sync_older')
  }

  function handleToggleInspector(): void {
    actions.toggleInspector()
  }

  return {
    handleMarkRead,
    handleMarkUnread,
    handlePin,
    handleUnpin,
    handleMute,
    handleUnmute,
    handleArchive,
    handleUnarchive,
    handleSyncLatest,
    handleSyncOlder,
    handleToggleInspector,
  }
}
