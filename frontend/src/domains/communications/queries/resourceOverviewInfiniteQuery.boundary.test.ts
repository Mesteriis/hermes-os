import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('resource overview infinite query boundary', () => {
  it('uses TanStack infinite queries for mailbox resource lists', () => {
    const source = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useInfiniteQuery<SubscriptionListResponse')
    expect(source).toContain('useInfiniteQuery<SenderStatsListResponse')
    expect(source).toContain('fetchSubscriptions(toValue(accountId), 25, pageParam)')
    expect(source).toContain('fetchTopSenders(toValue(accountId), 25, pageParam)')
    expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
    expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
  })
})
