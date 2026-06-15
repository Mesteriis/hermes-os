import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('CommunicationsConversationList boundary', () => {
	it('renders server-backed thread rows and exposes thread pagination controls', () => {
		const source = readFileSync(
			new URL('./CommunicationsConversationList.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain('threads: MailThreadSummary[]')
		expect(source).toContain('selectedThreadId: string')
		expect(source).toContain('accountId: string')
		expect(source).toContain('v-for="thread in threads"')
		expect(source).toContain('thread.message_count')
		expect(source).toContain('useThreadMessagesPrefetch')
		expect(source).toContain('handleThreadPrefetch')
		expect(source).toContain('@mouseenter="handleThreadPrefetch(thread)"')
		expect(source).toContain('@focus="handleThreadPrefetch(thread)"')
		expect(source).toContain("selectThread: [thread: MailThreadSummary]")
		expect(source).toContain("loadMoreThreads: []")
		expect(source).toContain('hasThreadNextPage')
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
	})
})
