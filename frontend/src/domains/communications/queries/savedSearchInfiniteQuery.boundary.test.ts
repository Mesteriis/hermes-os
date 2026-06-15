import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('saved-search infinite query boundary', () => {
  it('uses TanStack infinite query cursor loading for saved searches and smart folders', () => {
    const source = readFileSync(
      new URL('./mailWorkspaceQueries.ts', import.meta.url),
      'utf8'
    )

    expect(source).toContain('useInfiniteQuery<')
    expect(source).toContain('fetchSavedSearches(toValue(isSmartFolder), toValue(accountId), 100, pageParam)')
    expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
    expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
  })
})
