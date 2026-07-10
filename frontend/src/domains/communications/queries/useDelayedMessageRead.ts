import { onScopeDispose, toValue, watch, type MaybeRefOrGetter } from 'vue'

export const MESSAGE_READ_DELAY_MS = 2_000

export function useDelayedMessageRead(
  messageId: MaybeRefOrGetter<string | null | undefined>,
  markRead: (messageId: string) => Promise<unknown>,
  onError: (error: unknown) => void = () => undefined
) {
  let timer: ReturnType<typeof setTimeout> | undefined

  function cancelPendingMark() {
    if (timer === undefined) return
    clearTimeout(timer)
    timer = undefined
  }

  watch(
    () => toValue(messageId),
    (selectedMessageId) => {
      cancelPendingMark()
      if (!selectedMessageId) return

      timer = setTimeout(() => {
        timer = undefined
        void markRead(selectedMessageId).catch(onError)
      }, MESSAGE_READ_DELAY_MS)
    },
    { immediate: true }
  )

  onScopeDispose(cancelPendingMark)
}
