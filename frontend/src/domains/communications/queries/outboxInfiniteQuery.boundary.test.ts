import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('outbox infinite query boundary', () => {
	it('uses TanStack infinite query cursor loading for outbox server state', () => {
		const source = readFileSync(
			new URL('./mailOperationQueries.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useInfiniteQuery')
		expect(source).toContain('fetchOutboxItems(toValue(accountId), toValue(status), 100, pageParam)')
		expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
		expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
	})

	it('exposes load-more state from the outbox strip hook without API ownership in the component', () => {
		const hookSource = readFileSync(
			new URL('./outboxStatusStrip.ts', import.meta.url),
			'utf8'
		)
		const surfaceSource = readFileSync(
			new URL('./useCommunicationsPageSurface.ts', import.meta.url),
			'utf8'
		)
		const presentationSource = readFileSync(
			new URL('../components/outboxStatus.ts', import.meta.url),
			'utf8'
		)

		expect(hookSource).toContain('hasMoreOutboxItems')
		expect(hookSource).toContain('loadMoreOutboxItems')
		expect(hookSource).toContain('prefetchMoreOutboxItems')
		expect(hookSource).toContain('outboxQuery.fetchNextPage()')
		expect(surfaceSource).toContain('loadMoreOutboxItems')
		expect(surfaceSource).toContain('prefetchMoreOutboxItems')
		expect(surfaceSource).toContain('hasMoreOutboxItems')
		expect(presentationSource).toContain('outboxStatusPresentation')
		expect(presentationSource).not.toContain('fetch(')
		expect(presentationSource).not.toContain('ApiClient')
	})
})
