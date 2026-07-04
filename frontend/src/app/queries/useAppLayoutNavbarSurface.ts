import { computed, ref } from 'vue'
import {
	isUiThemeFamily,
	isUiThemeMode,
	themeSelectionToName,
	type UiThemeFamily,
	type UiThemeMode,
	type UiThemeName
} from '../../shared/ui/theme'

export type AppLayoutNavbarHealthStatus = 'healthy' | 'degraded' | 'unhealthy'

export type AppLayoutNavbarBreadcrumb = {
	id: string
	label: string
}

export type AppLayoutNavbarNavigationItem = {
	id: string
	label: string
	icon?: string
	iconTone?: AppLayoutNavbarNavigationIconTone
}

type AppLayoutNavbarNavigationIconTone =
	| 'accounts'
	| 'calendar'
	| 'channels'
	| 'communication'
	| 'dashboard'
	| 'documents'
	| 'knowledge'
	| 'mail'
	| 'review'
	| 'settings'
	| 'tasks'
	| 'telegram'
	| 'whatsapp'

export type AppLayoutNavbarNavigationLevel = {
	id: string
	label: string
	currentItem: AppLayoutNavbarNavigationItem
	items: readonly AppLayoutNavbarNavigationItem[]
}

type AppLayoutNavbarRouteNode = AppLayoutNavbarNavigationItem & {
	children?: readonly AppLayoutNavbarRouteNode[]
}

export type AppLayoutNavbarHealthCheck = {
	id: string
	label: string
	status: AppLayoutNavbarHealthStatus
	detail: string
}

export type AppLayoutNavbarToggleOption = {
	value: string
	label: string
}

export type AppLayoutNavbarThemeFamilyOption = {
	value: UiThemeFamily
	label: string
}

export type AppLayoutNavbarThemeModeOption = {
	value: UiThemeMode
	label: string
}

export type AppLayoutNavbarNotification = {
	id: string
	title: string
	body?: string
	sourceLabel: string
	timeLabel: string
	icon: string
	tone: 'info' | 'success' | 'warning' | 'danger'
}

export type AppLayoutNotificationToast = {
	id: string
	title: string
	description?: string
	variant: 'info' | 'success' | 'warning' | 'error'
}

export function useAppLayoutNavbarSurface() {
	const currentThemeFamily = ref<UiThemeFamily>('base')
	const currentThemeMode = ref<UiThemeMode>('light')
	const selectedRouteId = ref('telegram-all-accounts')
	const healthStatusLabelVisibleMs = 5000
	const notificationToastVisibleMs = 5000
	const currentTheme = computed<UiThemeName>(() => {
		return themeSelectionToName(currentThemeFamily.value, currentThemeMode.value)
	})

	const routeTree: readonly AppLayoutNavbarRouteNode[] = [
		{ id: 'dashboard', label: 'Dashboard', icon: 'tabler:layout-dashboard', iconTone: 'dashboard' },
		{
			id: 'communications',
			label: 'Communication',
			icon: 'tabler:messages',
			iconTone: 'communication',
			children: [
				{
					id: 'communication-all-channels',
					label: 'All channels',
					icon: 'tabler:message-circle',
					iconTone: 'channels'
				},
				{
					id: 'telegram',
					label: 'Telegram',
					icon: 'tabler:brand-telegram',
					iconTone: 'telegram',
					children: [
						{ id: 'telegram-all-accounts', label: 'All accounts', icon: 'tabler:users', iconTone: 'accounts' },
						{ id: 'telegram-account-1', label: 'Telegram account 1', icon: 'tabler:user-circle', iconTone: 'telegram' },
						{ id: 'telegram-account-2', label: 'Telegram account 2', icon: 'tabler:user-circle', iconTone: 'telegram' }
					]
				},
				{
					id: 'mail',
					label: 'Mail',
					icon: 'tabler:mail',
					iconTone: 'mail',
					children: [
						{ id: 'mail-all-accounts', label: 'All accounts', icon: 'tabler:users', iconTone: 'accounts' },
						{ id: 'mail-account-1', label: 'Mail account 1', icon: 'tabler:mail-opened', iconTone: 'mail' },
						{ id: 'mail-account-2', label: 'Mail account 2', icon: 'tabler:mail-opened', iconTone: 'mail' }
					]
				},
				{
					id: 'whatsapp',
					label: 'WhatsApp',
					icon: 'tabler:brand-whatsapp',
					iconTone: 'whatsapp',
					children: [
						{ id: 'whatsapp-all-accounts', label: 'All accounts', icon: 'tabler:users', iconTone: 'accounts' },
						{ id: 'whatsapp-account-1', label: 'WhatsApp account 1', icon: 'tabler:user-circle', iconTone: 'whatsapp' },
						{ id: 'whatsapp-account-2', label: 'WhatsApp account 2', icon: 'tabler:user-circle', iconTone: 'whatsapp' }
					]
				}
			]
		},
		{ id: 'review', label: 'Review', icon: 'tabler:clipboard-check', iconTone: 'review' },
		{ id: 'knowledge', label: 'Knowledge', icon: 'tabler:share', iconTone: 'knowledge' },
		{ id: 'tasks', label: 'Tasks', icon: 'tabler:checkbox', iconTone: 'tasks' },
		{ id: 'calendar', label: 'Calendar', icon: 'tabler:calendar', iconTone: 'calendar' },
		{ id: 'documents', label: 'Documents', icon: 'tabler:file-text', iconTone: 'documents' },
		{ id: 'settings', label: 'Settings', icon: 'tabler:settings', iconTone: 'settings' }
	]

	const selectedRoutePath = computed(() => {
		return findRoutePath(routeTree, selectedRouteId.value) ?? [routeTree[0]]
	})

	const breadcrumbs = computed<readonly AppLayoutNavbarBreadcrumb[]>(() => {
		return selectedRoutePath.value.map((item) => ({
			id: item.id,
			label: item.label
		}))
	})

	const navigationLevels = computed<readonly AppLayoutNavbarNavigationLevel[]>(() => {
		return selectedRoutePath.value.map((node, index, path) => {
			const parentNode = index === 0 ? undefined : path[index - 1]
			const siblings = parentNode?.children ?? routeTree

			return {
				id: `navigation-level-${index}`,
				label: navigationLevelLabel(index),
				currentItem: toNavigationItem(node),
				items: siblings.map(toNavigationItem)
			}
		})
	})

	const healthChecks = [
		{
			id: 'backend-api',
			label: 'Backend API',
			status: 'healthy',
			detail: 'HTTP healthcheck responds'
		},
		{
			id: 'event-spine',
			label: 'Event spine',
			status: 'healthy',
			detail: 'Realtime stream is connected'
		},
		{
			id: 'vault',
			label: 'Vault',
			status: 'degraded',
			detail: 'Unlock is required before provider sync'
		}
	] as const satisfies readonly AppLayoutNavbarHealthCheck[]

	const languageOptions: AppLayoutNavbarToggleOption[] = [
		{ value: 'ru', label: 'Русский' },
		{ value: 'en', label: 'English' }
	]

	const themeFamilyOptions: AppLayoutNavbarThemeFamilyOption[] = [
		{ value: 'base', label: 'Base' },
		{ value: 'hermes', label: 'Hermes' }
	]

	const themeModeOptions: AppLayoutNavbarThemeModeOption[] = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' }
	]

	const notifications = [
		{
			id: 'vault-unlock-required',
			title: 'Vault требует разблокировки',
			body: 'Провайдеры не смогут синхронизироваться, пока локальное хранилище секретов закрыто.',
			sourceLabel: 'System health',
			timeLabel: '2 мин назад',
			icon: 'tabler:lock-exclamation',
			tone: 'warning'
		},
		{
			id: 'review-candidates-ready',
			title: 'Новые кандидаты на review',
			body: '3 входящих сигнала готовы к проверке перед продвижением в память.',
			sourceLabel: 'Review',
			timeLabel: '18 мин назад',
			icon: 'tabler:inbox',
			tone: 'info'
		},
		{
			id: 'event-spine-connected',
			title: 'Event spine подключён',
			body: 'Realtime stream восстановил соединение и принимает новые события.',
			sourceLabel: 'Runtime',
			timeLabel: '1 ч назад',
			icon: 'tabler:activity',
			tone: 'success'
		}
	] as const satisfies readonly AppLayoutNavbarNotification[]

	const notificationToasts = computed<AppLayoutNotificationToast[]>(() => {
		return notifications.map((notification) => ({
			id: notification.id,
			title: notification.title,
			description: notification.body,
			variant: notificationToneToToastVariant(notification.tone)
		}))
	})

	function selectThemeFamily(value: string): void {
		if (!isUiThemeFamily(value)) return

		currentThemeFamily.value = value
	}

	function selectThemeMode(value: string): void {
		if (!isUiThemeMode(value)) return

		currentThemeMode.value = value
	}

	function selectNavigationItem(itemId: string): void {
		const path = findRoutePath(routeTree, itemId)
		const selectedNode = path?.at(-1)
		if (!selectedNode) return

		selectedRouteId.value = defaultRouteLeaf(selectedNode).id
	}

	return {
		breadcrumbs,
		healthChecks,
		navigationLevels,
		currentTheme,
		currentThemeFamily,
		currentThemeMode,
		currentLanguage: 'ru',
		languageOptions,
		notifications,
		notificationToasts,
		notificationToastVisibleMs,
		selectNavigationItem,
		selectThemeFamily,
		selectThemeMode,
		themeFamilyOptions,
		themeModeOptions,
		healthStatusLabelVisibleMs,
		notificationsCount: notifications.length
	}
}

function navigationLevelLabel(index: number): string {
	if (index === 0) return 'Main menu'
	if (index === 1) return 'Sub menu'
	return 'Elem'
}

function notificationToneToToastVariant(
	tone: AppLayoutNavbarNotification['tone']
): AppLayoutNotificationToast['variant'] {
	if (tone === 'danger') return 'error'

	return tone
}

function toNavigationItem(node: AppLayoutNavbarRouteNode): AppLayoutNavbarNavigationItem {
	return {
		id: node.id,
		label: node.label,
		icon: node.icon,
		iconTone: node.iconTone
	}
}

function defaultRouteLeaf(node: AppLayoutNavbarRouteNode): AppLayoutNavbarRouteNode {
	const firstChild = node.children?.[0]
	if (!firstChild) return node

	return defaultRouteLeaf(firstChild)
}

function findRoutePath(
	nodes: readonly AppLayoutNavbarRouteNode[],
	itemId: string,
	ancestors: readonly AppLayoutNavbarRouteNode[] = []
): AppLayoutNavbarRouteNode[] | undefined {
	for (const node of nodes) {
		const path = [...ancestors, node]
		if (node.id === itemId) {
			return path
		}

		const childPath = node.children ? findRoutePath(node.children, itemId, path) : undefined
		if (childPath) {
			return childPath
		}
	}

	return undefined
}
