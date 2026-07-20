import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { ClientSurfaceAvailabilityStateV1 } from '../../gen/hermes/gateway/v1/client_bootstrap_pb'
import { clientSurfaceCatalog } from '../../platform/client-runtime/clientSurfaces'
import { recoveryClientBootstrap } from '../../platform/gateway/clientBootstrap'
import { buildClientRouteTree } from './useClientNavigationSurface'

describe('compiled client navigation', () => {
	it('keeps every product route disabled in the recovery shell', () => {
		const tree = buildClientRouteTree(recoveryClientBootstrap())
		const productRoutes = flattenNavigationTree(tree).filter((item) => item.id !== 'settings')

		expect(productRoutes.every((item) => item.disabled)).toBe(true)
		expect(tree.find((item) => item.id === 'settings')?.disabled).toBe(false)
	})

	it('fails closed when Gateway marks a route available without a compiled owner adapter', () => {
		const bootstrap = Object.assign(new Map(recoveryClientBootstrap()), { modules: [] as const, systemStatus: [] as const })
		for (const surface of clientSurfaceCatalog) {
			if (surface.routeId === 'settings') continue
			bootstrap.set(surface.routeId, {
				state: ClientSurfaceAvailabilityStateV1.AVAILABLE,
				reasonCode: '',
				available: true,
			})
		}

		const tree = buildClientRouteTree(bootstrap)
		const productRoutes = flattenNavigationTree(tree).filter((item) => item.id !== 'settings')

		expect(productRoutes.every((item) => item.disabled)).toBe(true)
		expect(productRoutes.every((item) => item.disabledReason === 'client_route_adapter_unavailable')).toBe(true)
	})

	it('does not retain the legacy navbar or Communications facade as an active fallback', () => {
		const appLayoutSource = readFileSync(new URL('../layout/AppLayoutRoot.vue', import.meta.url), 'utf8')
		const navigationSource = readFileSync(new URL('./useClientNavigationSurface.ts', import.meta.url), 'utf8')

		expect(appLayoutSource).not.toContain('useAppLayoutNavbarSurface')
		expect(appLayoutSource).not.toContain('useCommunicationsViewSurface')
		expect(navigationSource).not.toContain('useAppLayoutNavbarSurface')
		expect(navigationSource).not.toContain('useCommunicationsWorkspaceSurface')
		expect(navigationSource).not.toContain('useCommunicationsPageSurface')
	})
})

type NavigationTreeItem = {
	id: string
	disabled?: boolean
	disabledReason?: string
	children?: readonly NavigationTreeItem[]
}

function flattenNavigationTree(nodes: readonly NavigationTreeItem[]): NavigationTreeItem[] {
	return nodes.flatMap((node) => [node, ...(node.children ? flattenNavigationTree(node.children) : [])])
}
