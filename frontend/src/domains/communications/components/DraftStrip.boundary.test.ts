import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('DraftStrip virtualization boundary', () => {
  it('renders drafts through TanStack Virtual without direct API access', () => {
    const source = readFileSync(new URL('./DraftStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain("from '@tanstack/vue-virtual'")
    expect(source).toContain('useVirtualizer')
    expect(source).toContain('draftVirtualizer')
    expect(source).toContain('virtualDraftRows')
    expect(source).toContain('draftVirtualTotalSize')
    expect(source).toContain('v-for="virtualRow in virtualDraftRows"')
    expect(source).toContain('hasMore')
    expect(source).toContain('loadMore')
    expect(source).toContain("emit('loadMore')")
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
