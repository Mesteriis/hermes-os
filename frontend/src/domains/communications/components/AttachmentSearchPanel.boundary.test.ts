import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('AttachmentSearchPanel boundary', () => {
  it('uses Vee/Zod forms, TanStack Query and TanStack Table without direct fetch', () => {
    const source = readFileSync(
      new URL('./AttachmentSearchPanel.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain("from 'vee-validate'")
    expect(source).toContain('../forms/attachmentSearchForm')
    expect(source).toContain('setFieldValue')
    expect(source).toContain('useAttachmentSearchQuery')
    expect(source).toContain('useVueTable')
    expect(source).toContain('getCoreRowModel')
    expect(source).toContain('useVirtualizer')
    expect(source).toContain('fetchNextPage')
    expect(source).toContain('hasNextPage')
    expect(source).toContain('useAttachmentSearchResultPrefetch')
    expect(source).toContain('@mouseenter="handleResultPrefetch(tableRows[virtualRow.index].original)"')
    expect(source).toContain('@focus="handleResultPrefetch(tableRows[virtualRow.index].original)"')
    expect(source).toContain('accountId: string | null')
    expect(source).toContain('@submit.prevent="submitSearch"')
    expect(source).not.toContain('../api/communications')
    expect(source).not.toContain('fetch(')
  })
})
