import { describe, expect, it, vi } from 'vitest'

import { listMailAccounts, resetMailAccountQueryConnectClientForTests } from './mailAccountQueryClient'

describe('mail account catalog query', () => {
	it('shares one in-flight request across provider consumers', async () => {
		resetMailAccountQueryConnectClientForTests()
		let resolveCatalog: ((value: { accounts: never[] }) => void) | undefined
		const list = vi.fn(() => new Promise<{ accounts: never[] }>((resolve) => {
			resolveCatalog = resolve
		}))
		const client = { list } as never

		const first = listMailAccounts(client)
		const second = listMailAccounts(client)
		resolveCatalog?.({ accounts: [] })

		expect(first).toBe(second)
		await expect(first).resolves.toEqual({ accounts: [] })
		expect(list).toHaveBeenCalledTimes(1)
	})
})
