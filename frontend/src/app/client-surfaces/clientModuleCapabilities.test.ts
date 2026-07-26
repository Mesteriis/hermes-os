import { describe, expect, it } from 'vitest'

import { recoveryClientBootstrap } from '../../platform/gateway/clientBootstrap'
import { hasClientModuleCapability } from './clientModuleCapabilities'

describe('client module capability composition', () => {
	it('requires both enabled sections and the exact capability', () => {
		const bootstrap = Object.assign(new Map(recoveryClientBootstrap()), {
			systemStatus: [] as const,
			modules: [
				{ sectionsEnabled: false, capabilityIds: ['telegram.command.v1'] },
				{ sectionsEnabled: true, capabilityIds: ['telegram.query.v1'] },
			] as never,
		})

		expect(hasClientModuleCapability(bootstrap, 'telegram.command.v1')).toBe(false)
		expect(hasClientModuleCapability(bootstrap, 'telegram.query.v1')).toBe(true)
		expect(hasClientModuleCapability(bootstrap, 'whatsapp.query.v1')).toBe(false)
	})
})
