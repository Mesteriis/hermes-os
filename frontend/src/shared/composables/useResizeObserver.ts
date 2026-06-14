import { onMounted, onUnmounted, ref, type Ref } from 'vue'

export function useResizeObserver(
  elRef: Ref<HTMLElement | null>,
  callback: (entry: ResizeObserverEntry) => void
): { width: Ref<number>; height: Ref<number> } {
  const width = ref(0)
  const height = ref(0)
  let observer: ResizeObserver | null = null

  onMounted(() => {
    const el = elRef.value
    if (!el) return

    observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        width.value = entry.contentRect.width
        height.value = entry.contentRect.height
        callback(entry)
      }
    })

    observer.observe(el)
  })

  onUnmounted(() => {
    observer?.disconnect()
  })

  return { width, height }
}
