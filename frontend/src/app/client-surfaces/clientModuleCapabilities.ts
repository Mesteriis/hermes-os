import type { ClientBootstrapSnapshot } from '../../platform/gateway/clientBootstrap'

export function hasClientModuleCapability(
	bootstrap: ClientBootstrapSnapshot,
	capabilityId: string,
): boolean {
	return bootstrap.modules.some((module) =>
		module.sectionsEnabled && module.capabilityIds.includes(capabilityId),
	)
}
