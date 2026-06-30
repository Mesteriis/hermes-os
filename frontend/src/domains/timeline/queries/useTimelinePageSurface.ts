import { watch } from 'vue'
import { useTimelineMessagesQuery } from './useTimelineQuery'
import { useTimelineStore } from '../stores/timeline'

export function useTimelinePageSurface() {
  const store = useTimelineStore()
  const messagesQuery = useTimelineMessagesQuery()

  watch(messagesQuery.data, (messages) => {
    if (!messages) return
    store.setMessages(messages)
    store.setLoading(false)
  })

  watch(messagesQuery.isLoading, (isLoading) => {
    store.setLoading(isLoading)
  })

  return {
    messages: messagesQuery.data,
    store
  }
}
