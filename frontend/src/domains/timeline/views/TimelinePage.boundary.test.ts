import { existsSync, readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('TimelinePage boundary', () => {
  it('preserves timeline orchestration after removing the legacy TimelinePage Vue layer', () => {
    const appViewSource = readFileSync(new URL('../../../app/views/TimelineView.vue', import.meta.url), 'utf8')
    const surfaceSource = readFileSync(new URL('../queries/useTimelinePageSurface.ts', import.meta.url), 'utf8')
    const storeSource = readFileSync(new URL('../stores/timeline.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('./TimelinePage.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/TimelineStream.vue', import.meta.url))).toBe(false)
    expect(existsSync(new URL('../components/TimelineFilters.vue', import.meta.url))).toBe(false)

    expect(appViewSource).toContain('Timeline UI removed after logic extraction. Rebuild pending new design language.')
    expect(appViewSource).toContain('Timeline logic is preserved')

    expect(surfaceSource).toContain('useTimelineMessagesQuery')
    expect(surfaceSource).toContain('useTimelineStore')
    expect(surfaceSource).toContain('store.setMessages')
    expect(surfaceSource).toContain('store.setLoading')
    expect(storeSource).toContain('filteredMessages')
    expect(storeSource).toContain('toggleFilter')
    expect(storeSource).toContain('setMessages')
  })
})
