import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('OutboxStatusStrip boundary', () => {
  it('renders existing query data without owning API or cache logic', () => {
    const source = readFileSync(
      new URL('./OutboxStatusStrip.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('visibleOutboxStatusItems')
    expect(source).toContain('outboxStatusPresentation')
    expect(source).toContain("undo: [outboxId: string]")
    expect(source).toContain("loadMore: []")
    expect(source).toContain("prefetchMore: []")
    expect(source).toContain('v-if="hasMore"')
    expect(source).toContain('@mouseenter="emit(\'prefetchMore\')"')
    expect(source).toContain('@focus="emit(\'prefetchMore\')"')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
    expect(source).not.toContain('useQuery')
  })
})
