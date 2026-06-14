import { onMounted, onUnmounted } from 'vue'

type KeyHandler = {
  key: string
  ctrl?: boolean
  meta?: boolean
  shift?: boolean
  handler: () => void
}

export function useKeyboard(handlers: KeyHandler[]): void {
  function handleKeydown(event: KeyboardEvent): void {
    for (const h of handlers) {
      const ctrl = h.ctrl ?? false
      const meta = h.meta ?? false
      const shift = h.shift ?? false

      if (
        event.key === h.key &&
        event.ctrlKey === ctrl &&
        event.metaKey === meta &&
        event.shiftKey === shift
      ) {
        event.preventDefault()
        h.handler()
        return
      }
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })
}

export function useEscapeKey(callback: () => void): void {
  useKeyboard([{ key: 'Escape', handler: callback }])
}
