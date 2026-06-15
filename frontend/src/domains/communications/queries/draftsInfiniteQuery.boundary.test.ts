import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('drafts infinite query boundary', () => {
  it('uses TanStack infinite query cursor loading for drafts', () => {
    const source = readFileSync(new URL('./mailOperationQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useInfiniteQuery<DraftListResponse')
    expect(source).toContain("queryKey: computed(() => ['communications-drafts', toValue(accountId)]")
    expect(source).toContain('initialPageParam: null')
    expect(source).toContain('fetchDrafts(toValue(accountId), undefined, 50, pageParam)')
    expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
    expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
  })
})
