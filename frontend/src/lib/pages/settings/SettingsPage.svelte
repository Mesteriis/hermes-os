<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { openAccountDrawer } from '$lib/stores/accountWizard';
	import {
		accountProviderIcon,
		accountProviderLabel,
		accountUpdatedLabel,
		addSidebarGroup,
		applicationSettings,
		calendarAccounts,
		cancelSidebarSettingsEditing,
		checkboxEventValue,
		effectiveSidebarSettings,
		emailProviderAccounts,
		formatDateTime,
		hasSidebarChanges,
		inputEventValue,
		isSettingsLoading,
		isSidebarSettingsSaving,
		moveSidebarGroup,
		moveSidebarItem,
		moveSidebarItemToGroup,
		moveSidebarRootItem,
		newSidebarGroupLabel,
		providerAccounts,
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
		telegramProviderAccounts,
		toggleSidebarGroupSeparator,
		toggleSidebarItemHidden,
		updateNewSidebarGroupLabel,
		updateSettingDraft,
		updateSidebarGroupLabel,
		whatsappProviderAccounts
	} from '$lib/stores/settings';

	import AppearanceSettings from './widgets/AppearanceSettings.svelte';
	import LanguageSettings from './widgets/LanguageSettings.svelte';
	import ApplicationSettings from './widgets/ApplicationSettings.svelte';
	import SidebarSettingsWidget from './widgets/SidebarSettings.svelte';
	import AccountsSettings from './widgets/AccountsSettings.svelte';

	const _ = (key: string) => t($currentLocale, key);

	function openAccountWizard(target?: string) {
		openAccountDrawer(target as Parameters<typeof openAccountDrawer>[0]);
	}
</script>

{#if $settingsActionMessage}
	<p class="setup-state success">{$settingsActionMessage}</p>
{/if}
{#if $settingsError}
	<p class="inline-error">{$settingsError}</p>
{/if}

<div class="section-tabs settings-tabs" aria-label="Settings sections">
	<button type="button" class:active={$selectedSettingsSection === 'appearance'} onclick={() => selectedSettingsSection.set('appearance')}>
		<Icon icon="tabler:palette" width="16" height="16" />{_('Appearance')}
	</button>
	<button type="button" class:active={$selectedSettingsSection === 'application'} onclick={() => selectedSettingsSection.set('application')}>
		<Icon icon="tabler:adjustments-horizontal" width="16" height="16" />Application
	</button>
	<button type="button" class:active={$selectedSettingsSection === 'sidebar'} onclick={() => selectedSettingsSection.set('sidebar')}>
		<Icon icon="tabler:layout-sidebar" width="16" height="16" />Sidebar
	</button>
	<button type="button" class:active={$selectedSettingsSection === 'accounts'} onclick={() => selectedSettingsSection.set('accounts')}>
		<Icon icon="tabler:users" width="16" height="16" />Accounts <em>{$providerAccounts.length}</em>
	</button>
	<button type="button" class:active={$selectedSettingsSection === 'language'} onclick={() => selectedSettingsSection.set('language')}>
		<Icon icon="tabler:language" width="16" height="16" />Language
	</button>
</div>

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
{:else}
	<AccountsSettings
		providerAccounts={$providerAccounts}
		calendarAccounts={$calendarAccounts}
		emailProviderAccounts={$emailProviderAccounts}
		telegramProviderAccounts={$telegramProviderAccounts}
		whatsappProviderAccounts={$whatsappProviderAccounts}
		onOpenAccountDrawer={openAccountWizard}
		accountProviderIconFn={accountProviderIcon}
		accountProviderLabelFn={accountProviderLabel}
		accountUpdatedLabelFn={accountUpdatedLabel}
		formatDateTimeFn={formatDateTime}
	/>
{/if}
