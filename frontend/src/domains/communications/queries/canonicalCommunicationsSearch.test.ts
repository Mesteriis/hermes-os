import { beforeEach, describe, expect, it, vi } from 'vitest'

const query = vi.fn()

vi.mock('../../../platform/connect/communicationsQueryClient', () => ({
	getCommunicationsQueryConnectClient: () => ({ query }),
}))

import { searchCanonicalCommunications } from './canonicalCommunicationsSearch'

describe('canonical Communications search adapter', () => {
	beforeEach(() => {
		query.mockReset()
	})

	it('preserves the opaque continuation without exposing it to presentation', async () => {
		const nextCursor = new Uint8Array([9])
		query.mockResolvedValueOnce({
			errorCode: '',
			result: { case: 'searchCommunications', value: { hits: [], nextCursor } },
		})

		const page = await searchCanonicalCommunications('  exact token  ', 20, new Uint8Array([1]))

		expect(page.nextCursor).toEqual(nextCursor)
		expect(query).toHaveBeenCalledWith(expect.objectContaining({
			operation: {
				case: 'searchCommunications',
				value: {
					query: 'exact token',
					limit: 20,
					cursor: new Uint8Array([1]),
				},
			},
		}))
	})

	it('rejects an empty query and oversized cursor before transport', async () => {
		await expect(searchCanonicalCommunications('   ')).rejects.toThrow(RangeError)
		await expect(
			searchCanonicalCommunications('token', 20, new Uint8Array(65)),
		).rejects.toThrow(RangeError)
		expect(query).not.toHaveBeenCalled()
	})
})
