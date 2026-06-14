import { onMounted, onUnmounted, type Ref } from 'vue'

export function useClickOutside(
  elRef: Ref<HTMLElement | null>,
  callback: () => void,
  options?: { excludeElRef?: Ref<HTMLElement | null> }
): void {
  function handleClick(event: MouseEvent): void {
    const el = elRef.value
    const excludeEl = options?.excludeElRef?.value

    if (!el) return
    if (el.contains(event.target as Node)) return
    if (excludeEl?.contains(event.target as Node)) return

    callback()
  }

  onMounted(() => {
    document.addEventListener('click', handleClick, true)
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClick, true)
  })
}
