import { describe, expect, it } from 'vitest'

import { clientSurfaceCatalog, hasCompiledClientSurfaceAdapter } from './clientSurfaces'

describe('compiled client surface catalog', () => {
	it('keeps the recovery surface mountable while owner contract pages stay fail-closed', () => {
		const settings = clientSurfaceCatalog.find((surface) => surface.routeId === 'settings')
		const productSurfaces = clientSurfaceCatalog.filter((surface) => surface.routeId !== 'settings')

		expect(settings).toBeDefined()
		expect(hasCompiledClientSurfaceAdapter(settings!)).toBe(true)
		expect(productSurfaces).not.toHaveLength(0)
		expect(productSurfaces.every((surface) => !hasCompiledClientSurfaceAdapter(surface))).toBe(true)
	})
})
