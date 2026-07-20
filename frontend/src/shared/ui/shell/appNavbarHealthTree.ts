import type { TreeItemData } from '../Navigation.types'

export type AppNavbarHealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unavailable'

export type AppNavbarHealthCheck = {
	id: string
	label: string
	status: AppNavbarHealthStatus
	detail: string
}

const healthGroups = [
	{ id: 'health-core', label: 'Core', checks: ['backend-1', 'backend-2', 'backend-3', 'backend-4', 'backend-15'] },
	{ id: 'health-data', label: 'Data', checks: ['backend-5', 'backend-6', 'backend-7', 'backend-8', 'backend-13'] },
	{ id: 'health-runtime', label: 'Runtime', checks: ['backend-9', 'backend-10', 'backend-11', 'backend-12', 'backend-14'] },
] as const

export function buildAppNavbarHealthTree(checks: readonly AppNavbarHealthCheck[]): TreeItemData[] {
	const checkById = new Map(checks.map((check) => [check.id, check]))
	const network = checkById.get('network')
	const items = network ? [toTreeItem(network)] : []
	for (const group of healthGroups) {
		const children = group.checks
			.map((id) => checkById.get(id))
			.filter((check): check is AppNavbarHealthCheck => check !== undefined)
			.map(toTreeItem)
		if (children.length === 0) continue
		items.push({
			id: group.id,
			label: group.label,
			status: aggregateHealthStatus(children),
			children,
		})
	}
	return items
}

export function problemHealthGroupIds(items: readonly TreeItemData[]): string[] {
	return items
		.filter((item) => item.children?.length && item.status !== 'healthy')
		.map((item) => item.id)
}

function toTreeItem(check: AppNavbarHealthCheck): TreeItemData {
	return {
		id: check.id,
		label: check.label,
		detail: check.detail,
		status: check.status,
		static: true,
	}
}

function aggregateHealthStatus(items: readonly TreeItemData[]): AppNavbarHealthStatus {
	if (items.some((item) => item.status === 'unhealthy')) return 'unhealthy'
	if (items.some((item) => item.status === 'degraded')) return 'degraded'
	if (items.some((item) => item.status === 'unavailable')) return 'unavailable'
	return 'healthy'
}
