import { describe, expect, it, vi } from 'vitest'

import { useCanonicalCommunicationContent } from './useCanonicalCommunicationContent'

describe('canonical Communications content lifecycle', () => {
	it('aborts the prior selection and fences its stale response', async () => {
		const requests: Array<{
			signal?: AbortSignal
			resolve: (value: Uint8Array) => void
		}> = []
		const read = vi.fn((_messageId: Uint8Array, signal?: AbortSignal) => (
			new Promise<Uint8Array>((resolve) => requests.push({ signal, resolve }))
		))
		const content = useCanonicalCommunicationContent(read)

		const first = content.open(new Uint8Array(16).fill(1))
		const second = content.open(new Uint8Array(16).fill(2))
		expect(requests[0]?.signal?.aborted).toBe(true)

		requests[0]?.resolve(new TextEncoder().encode('stale'))
		requests[1]?.resolve(new TextEncoder().encode('current'))
		await Promise.all([first, second])

		expect(content.model.value.status).toBe('ready')
		expect(content.model.value.bodyText).toBe('current')
	})

	it('fails invalid UTF-8 closed and clears content on close', async () => {
		const content = useCanonicalCommunicationContent(async () => (
			new Uint8Array([0xc3, 0x28])
		))
		await content.open(new Uint8Array(16))
		expect(content.model.value).toEqual({
			status: 'unavailable',
			statusMessage: 'Canonical message content is unavailable.',
			bodyText: '',
		})
		content.close()
		expect(content.model.value.status).toBe('idle')
	})
})
