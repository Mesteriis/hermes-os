<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { EmailAccountImportRequest } from '$lib/api';
	import { openAccountDrawer } from '$lib/stores/accountWizard';
	import {
		deleteMailAccount,
		emailAccountExportFilename,
		exportMailAccountSettings,
		importMailAccountSettings,
		logoutMailAccount,
		parseEmailAccountImportJson
	} from '$lib/services/accounts';
	import {
		addSidebarGroup,
		applicationSettings,
		cancelSidebarSettingsEditing,
		checkboxEventValue,
		effectiveSidebarSettings,
		formatDateTime,
		hasSidebarChanges,
		integrationViewModels,
			inputEventValue,
			isSettingsLoading,
			isSidebarSettingsSaving,
			loadSettingsWorkspace,
			moveSidebarGroup,
		moveSidebarItem,
		moveSidebarItemToGroup,
		moveSidebarRootItem,
		newSidebarGroupLabel,
		removeSidebarGroup,
		resetSidebarSettingsToDefault,
		saveSetting,
		saveSidebarSettings,
		selectedSettingsSection,
		settingAllowedValues,
		settingControl,
		settingDrafts,
		settingDraftValue,
		settingHasChanged,
		settingMetadataFlag,
		settingMetadataText,
		settingValueText,
		settingsActionMessage,
		settingsByCategory,
		settingsCategoryLabel,
		settingsError,
		sidebarConfigItem,
		sidebarError,
		sidebarGroupHasSeparatorBefore,
		sidebarGroupIdFromLabel,
		sidebarGroupLabel,
		sidebarHiddenNavItems,
		sidebarItemLabel,
		sidebarRootEntries,
		sidebarRootIndexForGroup,
		savingSettingKey,
		toggleSidebarGroupSeparator,
		toggleSidebarItemHidden,
		updateNewSidebarGroupLabel,
		updateSettingDraft,
		updateSidebarGroupLabel,
		type SettingsSection
	} from '$lib/stores/settings';

	import AppearanceSettings from './widgets/AppearanceSettings.svelte';
	import LanguageSettings from './widgets/LanguageSettings.svelte';
	import ApplicationSettings from './widgets/ApplicationSettings.svelte';
	import SidebarSettingsWidget from './widgets/SidebarSettings.svelte';
	import IntegrationsSettings from './widgets/IntegrationsSettings.svelte';
	import AISettingsControlCenter from './widgets/AISettingsControlCenter.svelte';

	const _ = (key: string) => t($currentLocale, key);

	function openAccountWizard(target?: string) {
		openAccountDrawer(target as Parameters<typeof openAccountDrawer>[0]);
	}

	type SettingsTreeItem = {
		id: SettingsSection;
		label: string;
		icon: string;
	};

	const settingsTreeGroups: Array<{ label: string; items: SettingsTreeItem[] }> = [
		{
			label: 'General',
			items: [
				{ id: 'application', label: 'Application', icon: 'tabler:adjustments-horizontal' },
				{ id: 'language', label: 'Language', icon: 'tabler:language' }
			]
		},
		{
			label: 'Interface',
			items: [
				{ id: 'appearance', label: 'Appearance', icon: 'tabler:palette' },
				{ id: 'sidebar', label: 'Sidebar', icon: 'tabler:layout-sidebar' }
			]
		},
		{
			label: 'Sources',
			items: [{ id: 'integrations', label: 'Integrations', icon: 'tabler:plug-connected' }]
		},
		{
			label: 'AI',
			items: [{ id: 'ai', label: 'AI Control Center', icon: 'tabler:sparkles' }]
		}
	];

	let selectedIntegrationId = $state<string | null>(null);

	$effect(() => {
		if ($integrationViewModels.length === 0) {
			selectedIntegrationId = null;
			return;
		}
		if (selectedIntegrationId && !$integrationViewModels.some((item) => item.integrationId === selectedIntegrationId)) {
			selectedIntegrationId = null;
		}
	});

	function selectIntegration(integrationId: string) {
		selectedIntegrationId = integrationId;
	}

	function closeIntegrationInspector() {
		selectedIntegrationId = null;
	}

	function downloadJsonFile(filename: string, content: string) {
		if (typeof document === 'undefined') return;
		const blob = new Blob([content], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const link = document.createElement('a');
		link.href = url;
		link.download = filename;
		document.body.appendChild(link);
		link.click();
		link.remove();
		URL.revokeObjectURL(url);
	}

	function failMailSettingsAction(message: string): never {
		settingsError.set(message);
		settingsActionMessage.set('');
		throw new Error(message);
	}

	async function handleExportMailAccount(accountId: string): Promise<void> {
		settingsError.set('');
		settingsActionMessage.set('');
		const result = await exportMailAccountSettings(accountId);
		if (result.error || !result.result) {
			failMailSettingsAction(result.error || 'Mail account export failed');
		}
		downloadJsonFile(
			emailAccountExportFilename(accountId, result.result.exported_at),
			JSON.stringify(result.result, null, 2)
		);
		settingsActionMessage.set('Mail account settings exported');
	}

	async function handleLogoutMailAccount(accountId: string): Promise<void> {
		settingsError.set('');
		settingsActionMessage.set('');
		const result = await logoutMailAccount(accountId);
		if (result.error || !result.result) {
			failMailSettingsAction(result.error || 'Mail account logout failed');
		}
		settingsActionMessage.set('Mail account logged out');
		await loadSettingsWorkspace();
	}

	async function handleDeleteMailAccount(accountId: string): Promise<void> {
		settingsError.set('');
		settingsActionMessage.set('');
		const result = await deleteMailAccount(accountId);
		if (result.error || !result.result) {
			failMailSettingsAction(result.error || 'Mail account delete failed');
		}
		settingsActionMessage.set('Mail account deleted');
		await loadSettingsWorkspace();
	}

	async function handleImportMailSettings(rawJson: string): Promise<void> {
		settingsError.set('');
		settingsActionMessage.set('');
		let request: EmailAccountImportRequest;
		try {
			request = parseEmailAccountImportJson(rawJson);
		} catch (error) {
			failMailSettingsAction(error instanceof Error ? error.message : 'Mail account import failed');
		}
		const result = await importMailAccountSettings(request);
		if (result.error || !result.result) {
			failMailSettingsAction(result.error || 'Mail account import failed');
		}
		settingsActionMessage.set('Imported mail settings');
		await loadSettingsWorkspace();
	}
</script>

{#if $settingsActionMessage}
	<p class="setup-state success">{$settingsActionMessage}</p>
{/if}
{#if $settingsError}
	<p class="inline-error">{$settingsError}</p>
{/if}

<div class="settings-workbench">
	<nav class="settings-tree" aria-label={_('Settings sections')}>
		{#each settingsTreeGroups as group}
			<section class="settings-tree-group">
				<h2>{_(group.label)}</h2>
				{#each group.items as item}
					<button
						type="button"
						class:active={$selectedSettingsSection === item.id}
						onclick={() => selectedSettingsSection.set(item.id)}
					>
						<Icon icon={item.icon} width="16" height="16" />
						<span>{_(item.label)}</span>
						{#if item.id === 'integrations'}
							<em>{$integrationViewModels.length}</em>
						{/if}
					</button>
				{/each}
			</section>
		{/each}
	</nav>

	<div class="settings-workbench-content">
		{#if $selectedSettingsSection === 'appearance'}
			<AppearanceSettings />
		{:else if $selectedSettingsSection === 'language'}
			<LanguageSettings />
		{:else if $selectedSettingsSection === 'application'}
			<ApplicationSettings
				applicationSettings={$applicationSettings}
				settingDrafts={$settingDrafts}
				isSettingsLoading={$isSettingsLoading}
				savingSettingKey={$savingSettingKey}
				settingsByCategory={$settingsByCategory}
				onSaveSetting={saveSetting}
				onUpdateSettingDraft={updateSettingDraft}
				settingsCategoryLabelFn={settingsCategoryLabel}
				settingDraftValueFn={settingDraftValue}
				settingHasChangedFn={settingHasChanged}
				settingAllowedValuesFn={settingAllowedValues}
				settingControlFn={settingControl}
				settingMetadataFlagFn={settingMetadataFlag}
				settingMetadataTextFn={settingMetadataText}
				settingValueTextFn={settingValueText}
				inputEventValueFn={inputEventValue}
				checkboxEventValueFn={checkboxEventValue}
			/>
		{:else if $selectedSettingsSection === 'sidebar'}
			<SidebarSettingsWidget
				sidebarError={$sidebarError}
				isSidebarSettingsSaving={$isSidebarSettingsSaving}
				newSidebarGroupLabel={$newSidebarGroupLabel}
				sidebarRootEntries={$sidebarRootEntries}
				sidebarHiddenNavItems={$sidebarHiddenNavItems}
				effectiveSidebarSettings={$effectiveSidebarSettings}
				hasSidebarChanges={$hasSidebarChanges}
				onCancelSidebarEditing={cancelSidebarSettingsEditing}
				onResetSidebar={resetSidebarSettingsToDefault}
				onSaveSidebar={saveSidebarSettings}
				onAddSidebarGroup={addSidebarGroup}
				onRemoveSidebarGroup={removeSidebarGroup}
				onMoveSidebarGroup={moveSidebarGroup}
				onMoveSidebarRootItem={moveSidebarRootItem}
				onMoveSidebarItem={moveSidebarItem}
				onMoveSidebarItemToGroup={moveSidebarItemToGroup}
				onToggleSidebarGroupSeparator={toggleSidebarGroupSeparator}
				onToggleSidebarItemHidden={toggleSidebarItemHidden}
				onUpdateSidebarGroupLabel={updateSidebarGroupLabel}
				onUpdateNewSidebarGroupLabel={updateNewSidebarGroupLabel}
				sidebarGroupLabelFn={sidebarGroupLabel}
				sidebarItemLabelFn={sidebarItemLabel}
				sidebarGroupHasSeparatorBeforeFn={sidebarGroupHasSeparatorBefore}
				sidebarRootIndexForGroupFn={sidebarRootIndexForGroup}
				sidebarGroupIdFromLabelFn={sidebarGroupIdFromLabel}
				sidebarConfigItemFn={sidebarConfigItem}
				inputEventValueFn={inputEventValue}
			/>
		{:else if $selectedSettingsSection === 'integrations'}
			<IntegrationsSettings
				integrations={$integrationViewModels}
				{selectedIntegrationId}
				onSelectIntegration={selectIntegration}
				onCloseIntegration={closeIntegrationInspector}
				onOpenAccountDrawer={openAccountWizard}
				onExportMailAccount={handleExportMailAccount}
				onLogoutMailAccount={handleLogoutMailAccount}
				onDeleteMailAccount={handleDeleteMailAccount}
				onImportMailSettings={handleImportMailSettings}
				formatDateTimeFn={formatDateTime}
			/>
		{:else}
			<AISettingsControlCenter />
		{/if}
	</div>
</div>
