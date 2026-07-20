import { describe, expect, it } from 'vitest'

import { buildAppNavbarHealthTree, problemHealthGroupIds } from './appNavbarHealthTree'

describe('buildAppNavbarHealthTree', () => {
	it('groups the detailed health checks by platform topology', () => {
		const tree = buildAppNavbarHealthTree([
			{ id: 'network', label: 'Connection', status: 'healthy', detail: '8 ms round-trip' },
			{ id: 'backend-1', label: 'Kernel', status: 'healthy', detail: 'healthy' },
			{ id: 'backend-5', label: 'Vault', status: 'degraded', detail: 'awaiting_key' },
			{ id: 'backend-9', label: 'NATS', status: 'unavailable', detail: 'not_admitted' },
		])

		expect(tree.map((item) => item.label)).toEqual(['Connection', 'Core', 'Data', 'Runtime'])
		expect(tree[1]?.status).toBe('healthy')
		expect(tree[2]?.status).toBe('degraded')
		expect(tree[3]?.status).toBe('unavailable')
		expect(tree[2]?.children?.[0]).toMatchObject({ label: 'Vault', static: true })
		expect(problemHealthGroupIds(tree)).toEqual(['health-data', 'health-runtime'])
	})
})
