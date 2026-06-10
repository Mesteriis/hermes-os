<script lang="ts">
	import '$lib/pages/pages.css';
	import '$lib/styles/tokens.css';
	import '$lib/styles/app.css';
	import { onMount } from 'svelte';
	import { currentLocale, t, setLocale } from '$lib/i18n';
	import { activeCommunicationSection, activeSidebarRailGroupId, isSidebarRail, expandedSidebarGroupIds, isUserMenuOpen, navigateTo, navigateToCommunicationSection, toggleUserMenu, closeUserMenu, activeView, shellViewClass } from '$lib/stores/navigation';
	import { toggleNotificationsDrawer, notificationItems, notificationCount, openNotificationTarget, type NotificationItem } from '$lib/stores/notifications';
	import { shouldShowVaultOnboarding, vaultStatus as vaultStatusStore, vaultWizardStep, vaultWizardError, vaultWizardMessage, vaultEntropyEventsCount, vaultRecovery, isVaultActionSubmitting, vaultStatusError, continueVaultOnboarding } from '$lib/stores/vault';
	import { shellThemeClass } from '$lib/stores/theme';
	import { sidebarRootEntries } from '$lib/stores/sidebar';
	import { activeWidgetById, addableWidgetsForCurrentView, adjustWidgetGridValue, cancelLayoutEditing, closeAddWidgetDrawer, closeWidgetSettingsDrawer, handleWidgetGridInput, handleWidgetPanelSurfaceInput, hideWidget, isLayoutEditing, isLayoutSettingsSaving, isWidgetDrawerOpen, layoutDraft, moveWidgetInZone, openAddWidgetDrawer, resetCurrentViewLayout, resetWidgetGrid, resetWidgetPanelSurface, saveLayoutSettings, selectedLayoutWidget, selectedLayoutWidgetId, showWidget, startLayoutEditing, syncWidgetGridClasses, widgetGridMax, widgetGridMin, widgetGridValue, widgetPanelSurfaceOverrideValue, widgetPanelSurfaceValue, widgetZoneTitle } from '$lib/stores/layoutEditor';
	import VaultOnboarding from '$lib/components/vault/VaultOnboarding.svelte';
	import Sidebar from '$lib/components/shell/Sidebar.svelte';
	import Topbar from '$lib/components/shell/Topbar.svelte';
	import NotificationsDrawer from '$lib/components/shell/NotificationsDrawer.svelte';
	import LayoutEditControls from '$lib/components/shared/LayoutEditControls.svelte';
	import WidgetSettingsDrawer from '$lib/components/shared/WidgetSettingsDrawer.svelte';
	import AddWidgetDrawer from '$lib/components/shared/AddWidgetDrawer.svelte';
	import * as vaultService from '$lib/services/vault';
	import { saveFrontendLocaleSetting } from '$lib/api';
	import type { SidebarNavGroup, ResolvedSidebarItem } from '$lib/layout';
	import type { AppViewId } from '$lib/stores/navigation';

	let { children } = $props();
	const _ = (key: string) => t($currentLocale, key);

	let vaultEntropyState = $state<vaultService.VaultEntropyState>({
		lastEntropyEvent: null,
		entropyEvents: [],
		entropyBuffer: [],
		status: null
	});

	onMount(() => {
		void loadVaultStatus();
	});

	$effect(() => {
		const sync = () => syncWidgetGridClasses($activeWidgetById);
		if (typeof document === 'undefined') {
			return;
		}

		let pendingFrame: number | null = null;
		const scheduleSync = () => {
			if (pendingFrame !== null) {
				return;
			}

			pendingFrame = requestAnimationFrame(() => {
				pendingFrame = null;
				sync();
			});
		};

		sync();
		const workspaceRoot =
			document.querySelector<HTMLElement>('.workspace') ??
			document.querySelector<HTMLElement>('.desktop-shell') ??
			document.body;
		const widgetObserver = new MutationObserver(scheduleSync);
		widgetObserver.observe(workspaceRoot, {
			childList: true,
			subtree: true
		});

		window.addEventListener('resize', scheduleSync);

		return () => {
			window.removeEventListener('resize', scheduleSync);
			widgetObserver.disconnect();
			if (pendingFrame !== null) {
				cancelAnimationFrame(pendingFrame);
			}
		};
	});

	$effect(() => {
		if (!$isLayoutEditing || ($selectedLayoutWidgetId && !$activeWidgetById.has($selectedLayoutWidgetId))) {
			selectedLayoutWidgetId.set(null);
		}
	});

	function toggleSidebarRail() {
		isSidebarRail.update(v => !v);
		activeSidebarRailGroupId.set(null);
	}

	function handleToggleGroup(group: SidebarNavGroup) {
		if ($isSidebarRail) {
			activeSidebarRailGroupId.set($activeSidebarRailGroupId === group.id ? null : group.id);
			return;
		}
		expandedSidebarGroupIds.update(ids =>
			ids.includes(group.id)
				? ids.filter(id => id !== group.id)
				: [...ids, group.id]
		);
	}

	function handleSelectItem(item: ResolvedSidebarItem<any>) {
		if (item.kind === 'primary') {
			navigateTo(item.primary.id as AppViewId);
			return;
		}
		navigateToCommunicationSection(item.section.id);
	}

	function formatDateTime(value: string | null) {
		if (!value) return '';
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return '';
		return new Intl.DateTimeFormat('en', {
			month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
		}).format(date);
	}

	function exitApplication() {
		isUserMenuOpen.set(false);
		if (typeof window !== 'undefined') window.close();
	}

	async function loadVaultStatus() {
		const result = await vaultService.loadV1Status();
		vaultEntropyState = { ...vaultEntropyState, status: result.status };
		vaultStatusError.set(result.error);
	}

	async function handleVaultEntropyMove(event: MouseEvent) {
		const { state, shouldFlush } = await vaultService.handleVaultEntropyMove(event, vaultEntropyState);
		vaultEntropyState = state;
		if (shouldFlush) {
			await flushVaultEntropy();
		}
	}

	async function flushVaultEntropy() {
		const { state, error } = await vaultService.flushVaultEntropy(vaultEntropyState);
		vaultEntropyState = state;
		if (error) vaultStatusError.set(error);
	}

	async function createSecureVault() {
		const { state, error } = await vaultService.createSecureVault(vaultEntropyState);
		vaultEntropyState = state;
		if (error) vaultStatusError.set(error);
	}

	async function unlockSecureVault() {
		const { state, error } = await vaultService.unlockSecureVault(vaultEntropyState);
		vaultEntropyState = state;
		if (error) vaultStatusError.set(error);
	}

	async function exportRecoveryMaterial() {
		const { error } = await vaultService.exportRecoveryMaterial();
		if (error) vaultStatusError.set(error);
	}
</script>

<div class="viewport-guard" role="alert" aria-live="polite">
	<strong>{_('Minimum window size is 800 x 600')}</strong>
	<span>{_('Increase the Hermes Hub window size to continue.')}</span>
</div>

<main class={`desktop-shell ${$shellViewClass} ${$shellThemeClass}`} class:sidebar-rail={$isSidebarRail}>
	{#if $shouldShowVaultOnboarding}
		<VaultOnboarding
			wizardStep={$vaultWizardStep}
			status={$vaultStatusStore}
			statusError={$vaultStatusError}
			entropyEventsCount={$vaultEntropyEventsCount}
			wizardError={$vaultWizardError}
			wizardMessage={$vaultWizardMessage}
			recovery={$vaultRecovery}
			isActionSubmitting={$isVaultActionSubmitting}
			onStartWizard={vaultService.startVaultWizard}
			onCreateVault={createSecureVault}
			onUnlockVault={unlockSecureVault}
			onExportRecovery={exportRecoveryMaterial}
			onContinue={continueVaultOnboarding}
			onEntropyMove={handleVaultEntropyMove}
		/>
	{/if}
	<Sidebar
		sidebarRootEntries={$sidebarRootEntries}
		expandedSidebarGroupIds={$expandedSidebarGroupIds}
		onSelectItem={handleSelectItem}
		onToggleGroup={handleToggleGroup}
		onToggleRail={toggleSidebarRail}
		onSettings={() => navigateTo('settings')}
	/>

	{#if $isSidebarRail && $activeSidebarRailGroupId !== null}
		<button
			type="button"
			class="sidebar-rail-dropdown-backdrop"
			aria-label="Close sidebar menu"
			onclick={() => activeSidebarRailGroupId.set(null)}
		></button>
	{/if}

	<section class="workspace" aria-label={`${_($activeView.title)} workspace`}>
		<Topbar
			viewTitle={$activeView.title}
			viewSubtitle={$activeView.subtitle}
			notificationCount={$notificationCount}
			isUserMenuOpen={$isUserMenuOpen}
			isLayoutEditing={$isLayoutEditing}
			onToggleNotifications={() => { toggleNotificationsDrawer(); isUserMenuOpen.set(false); }}
			onToggleUserMenu={toggleUserMenu}
			onCloseUserMenu={closeUserMenu}
			onStartLayoutEditing={startLayoutEditing}
			onToggleLocale={async () => { const loc = $currentLocale === 'en' ? 'ru' : 'en'; setLocale(loc); try { await saveFrontendLocaleSetting(loc); } catch (_) {} }}
			onExit={exitApplication}
		/>

		<NotificationsDrawer
			notificationItems={$notificationItems}
			onOpenTarget={(notification) => openNotificationTarget(notification as NotificationItem)}
			{formatDateTime}
		/>

		<LayoutEditControls
			isLayoutEditing={$isLayoutEditing}
			isSaving={$isLayoutSettingsSaving}
			hasChanges={$layoutDraft !== null}
			onAddWidget={openAddWidgetDrawer}
			onCancel={cancelLayoutEditing}
			onReset={resetCurrentViewLayout}
			onSave={saveLayoutSettings}
		/>

		{@render children()}

		<WidgetSettingsDrawer
			isOpen={$isLayoutEditing && $selectedLayoutWidget !== null}
			widget={$selectedLayoutWidget}
			onClose={closeWidgetSettingsDrawer}
			{widgetGridValue}
			{widgetGridMin}
			{widgetGridMax}
			{adjustWidgetGridValue}
			{handleWidgetGridInput}
			{widgetPanelSurfaceValue}
			{widgetPanelSurfaceOverrideValue}
			{handleWidgetPanelSurfaceInput}
			{resetWidgetPanelSurface}
			{resetWidgetGrid}
			{moveWidgetInZone}
			{hideWidget}
			{widgetZoneTitle}
		/>

		<AddWidgetDrawer
			isOpen={$isLayoutEditing && $isWidgetDrawerOpen}
			widgets={$addableWidgetsForCurrentView}
			onClose={closeAddWidgetDrawer}
			onShowWidget={showWidget}
		/>
	</section>
</main>
