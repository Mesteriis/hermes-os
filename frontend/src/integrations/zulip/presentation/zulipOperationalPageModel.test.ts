import { describe, expect, it } from 'vitest'

import { buildZulipOperationStatusCard } from './zulipOperationalPageModel'

describe('Zulip operational page model', () => {
	it('maps provider status without inventing missing completion data', () => {
		expect(buildZulipOperationStatusCard({
			operationId: 'operation-1',
			accountId: 'account-1',
			outcome: 'accepted',
			providerMessageId: 42n,
			requestedAtUnixSeconds: 1_700_000_000n,
		} as never)).toMatchObject({
			operationId: 'operation-1',
			outcome: 'accepted',
			providerMessageId: '42',
			completedAt: 'Pending',
		})
	})

	it('keeps an absent status empty', () => {
		expect(buildZulipOperationStatusCard(null)).toBeNull()
	})
})
