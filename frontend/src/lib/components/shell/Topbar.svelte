<script lang="ts">
	import './topbar.css';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { isNotificationsDrawerOpen } from '$lib/stores/notifications';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		viewTitle: string;
		viewSubtitle: string;
		notificationCount: number;
		isUserMenuOpen: boolean;
		isLayoutEditing: boolean;
		onToggleNotifications: () => void;
		onToggleUserMenu: () => void;
		onCloseUserMenu: () => void;
		onStartLayoutEditing: () => void;
		onToggleLocale: () => void;
		onExit: () => void;
	}

	let {
		viewTitle,
		viewSubtitle,
		notificationCount,
		isUserMenuOpen,
		isLayoutEditing,
		onToggleNotifications,
		onToggleUserMenu,
		onCloseUserMenu,
		onStartLayoutEditing,
		onToggleLocale,
		onExit
	}: Props = $props();
</script>

<header class="topbar">
	<div class="topbar-title">
		<h1>{_(viewTitle)}</h1>
		<p>{_(viewSubtitle)}</p>
	</div>
	<div class="top-actions">
		<button
			type="button"
			class="icon-button"
			class:active={$isNotificationsDrawerOpen}
			aria-label="Open notifications"
			aria-expanded={$isNotificationsDrawerOpen}
			aria-controls="notifications-drawer"
			title="Open notifications"
			onclick={onToggleNotifications}
		>
			<Icon icon="tabler:bell" width="18" height="18" />
			{#if notificationCount > 0}
				<i>{notificationCount}</i>
			{/if}
		</button>
		<div class="user-menu-shell">
			<button
				type="button"
				class="menu-button"
				aria-haspopup="menu"
				aria-expanded={isUserMenuOpen}
				aria-controls="user-menu"
				onclick={onToggleUserMenu}
				title="Open user menu"
			>
				<Icon icon="tabler:menu-2" width="20" height="20" />
			</button>
			{#if isUserMenuOpen}
				<button type="button" class="user-menu-backdrop" aria-label="Close user menu" onclick={onCloseUserMenu}></button>
				<div id="user-menu" class="user-menu" role="menu" aria-label="User menu">
					<button type="button" role="menuitem" onclick={onStartLayoutEditing} disabled={isLayoutEditing}>
						<Icon icon="tabler:layout-dashboard" width="16" height="16" />
						<span>{_('Constructor Mode')}</span>
					</button>
					<button type="button" role="menuitem" onclick={onToggleLocale}>
						<Icon icon="tabler:language" width="16" height="16" />
						<span>{_('Switch language')}</span>
					</button>
					<div class="user-menu-separator" role="separator"></div>
					<button type="button" role="menuitem" onclick={onExit}>
						<Icon icon="tabler:logout" width="16" height="16" />
						<span>{_('Exit')}</span>
					</button>
				</div>
			{/if}
		</div>
	</div>
</header>
