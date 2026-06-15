import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('thread infinite query boundary', () => {
	it('uses TanStack infinite query cursor loading for thread server state', () => {
		const source = readFileSync(
			new URL('./mailCoreQueries.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useInfiniteQuery')
		expect(source).toContain('fetchThreads(toValue(accountId), 50, pageParam)')
		expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
		expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
	})

	it('keeps thread message loading behind a dedicated query hook', () => {
		const source = readFileSync(
			new URL('./mailCoreQueries.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useThreadMessagesQuery')
		expect(source).toContain('fetchThreadMessages(')
		expect(source).toContain('communications-thread-messages')
		expect(source).toContain('enabled: computed(() => Boolean(toValue(accountId) && toValue(subject)))')
	})
})
