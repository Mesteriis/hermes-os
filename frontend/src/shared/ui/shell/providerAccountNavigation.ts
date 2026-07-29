export type ProviderAccountNavigationTone = 'mail' | 'telegram'

export type ProviderAccountNavigationEntry = {
	accountId: string
	label: string
}

export type ProviderAccountNavigationSnapshot = {
	channelId: ProviderAccountNavigationTone
	entries: readonly ProviderAccountNavigationEntry[]
	loading: boolean
	selectedAccountId: string
}

export type ProviderAccountNavigationItem = {
	id: string
	label: string
	icon?: string
	iconTone?: ProviderAccountNavigationTone
	disabled?: boolean
	loading?: boolean
}

export type ProviderAccountNavigationLevel = {
	id: string
	label: string
	currentItem: ProviderAccountNavigationItem
	items: readonly ProviderAccountNavigationItem[]
}

export function providerAccountNavigationLevel(
	channelId: ProviderAccountNavigationTone,
	snapshot?: ProviderAccountNavigationSnapshot,
): ProviderAccountNavigationLevel {
	const loading = snapshot?.loading ?? true
	const items = loading
		? [loadingItem(channelId)]
		: accountItems(channelId, snapshot?.entries ?? [])
	const selectedRouteId = snapshot?.selectedAccountId
		? accountRouteId(channelId, snapshot.selectedAccountId)
		: allAccountsRouteId(channelId)
	const currentItem = items.find((item) => item.id === selectedRouteId) ?? items[0]!

	return {
		id: `navigation-level-${channelId}-accounts`,
		label: channelId === 'mail' ? 'Выбор почтового ящика' : 'Выбор аккаунта Telegram',
		currentItem,
		items,
	}
}

export function providerAccountIdFromRoute(
	channelId: ProviderAccountNavigationTone,
	routeId: string,
): string | undefined {
	if (routeId === allAccountsRouteId(channelId)) return ''
	const prefix = `communications-${channelId}-account:`
	if (!routeId.startsWith(prefix)) return undefined
	return decodeURIComponent(routeId.slice(prefix.length))
}

function accountItems(
	channelId: ProviderAccountNavigationTone,
	entries: readonly ProviderAccountNavigationEntry[],
): readonly ProviderAccountNavigationItem[] {
	const allItem: ProviderAccountNavigationItem = {
		id: allAccountsRouteId(channelId),
		label: channelId === 'mail' ? 'Все ящики' : 'Все аккаунты',
		icon: channelId === 'mail' ? 'tabler:inbox' : 'tabler:users',
		iconTone: channelId,
	}
	const accountIcon = channelId === 'mail' ? 'tabler:mail-opened' : 'tabler:user-circle'
	return [
		allItem,
		...entries.map((entry) => ({
			id: accountRouteId(channelId, entry.accountId),
			label: entry.label,
			icon: accountIcon,
			iconTone: channelId,
		})),
	]
}

function loadingItem(channelId: ProviderAccountNavigationTone): ProviderAccountNavigationItem {
	return {
		id: `communications-${channelId}-accounts:loading`,
		label: '',
		iconTone: channelId,
		disabled: true,
		loading: true,
	}
}

function allAccountsRouteId(channelId: ProviderAccountNavigationTone): string {
	return `communications-${channelId}-accounts:all`
}

function accountRouteId(channelId: ProviderAccountNavigationTone, accountId: string): string {
	return `communications-${channelId}-account:${encodeURIComponent(accountId)}`
}
