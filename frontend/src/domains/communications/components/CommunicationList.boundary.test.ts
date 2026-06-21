import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationList keyboard multi-select boundary', () => {
  it('supports keyboard selection without direct API access', () => {
    const source = readFileSync(new URL('./CommunicationList.vue', import.meta.url), 'utf8')

    expect(source).toContain('handleKeydown')
    expect(source).toContain('@keydown="handleKeydown"')
    expect(source).toContain('tabindex="0"')
    expect(source).toContain('role="listbox"')
    expect(source).toContain('aria-multiselectable="true"')
    expect(source).toContain("event.code === 'Space'")
    expect(source).toContain("event.key.toLowerCase() === 'a'")
    expect(source).toContain('event.metaKey || event.ctrlKey')
    expect(source).toContain("event.key === 'Escape'")
    expect(source).toContain("event.key === 'ArrowDown'")
    expect(source).toContain("event.key === 'ArrowUp'")
    expect(source).toContain("emit('toggleSelection', current.message_id, event.shiftKey)")
    expect(source).toContain("emit('selectVisible', visibleMessageIds.value)")
    expect(source).toContain("emit('clearSelection')")
    expect(source).toContain("emit('toggleSelection', next.message_id, true)")
    expect(source).not.toMatch(/\bfetch\s*\(/)
    expect(source).not.toContain('ApiClient')
  })
})
