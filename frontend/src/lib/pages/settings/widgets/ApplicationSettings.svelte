<script lang="ts">
	import Icon from '@iconify/svelte';
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { ApplicationSetting } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		applicationSettings: ApplicationSetting[];
		settingDrafts: Record<string, string>;
		isSettingsLoading: boolean;
		savingSettingKey: string | null;

		settingsByCategory: Record<string, ApplicationSetting[]>;
		onSaveSetting: (setting: ApplicationSetting) => Promise<void>;
		onUpdateSettingDraft: (key: string, value: string) => void;

		settingsCategoryLabelFn: (category: string) => string;
		settingDraftValueFn: (setting: ApplicationSetting) => string;
		settingHasChangedFn: (setting: ApplicationSetting) => boolean;
		settingAllowedValuesFn: (setting: ApplicationSetting) => string[];
		settingControlFn: (setting: ApplicationSetting) => string;
		settingMetadataFlagFn: (setting: ApplicationSetting, key: string) => boolean;
		settingMetadataTextFn: (setting: ApplicationSetting, key: string) => string;
		settingValueTextFn: (settingKey: string) => string;
		inputEventValueFn: (event: Event) => string;
		checkboxEventValueFn: (event: Event) => string;
	}

	let {
		applicationSettings,
		settingDrafts,
		isSettingsLoading,
		savingSettingKey,
		settingsByCategory,
		onSaveSetting,
		onUpdateSettingDraft,
		settingsCategoryLabelFn,
		settingDraftValueFn,
		settingHasChangedFn,
		settingAllowedValuesFn,
		settingControlFn,
		settingMetadataFlagFn,
		settingMetadataTextFn,
		settingValueTextFn,
		inputEventValueFn,
		checkboxEventValueFn
	}: Props = $props();

	function settingSelectOptions(setting: ApplicationSetting) {
		return settingAllowedValuesFn(setting).map((value) => ({
			value,
			label: settingsCategoryLabelFn(value)
		}));
	}
</script>

<div class="settings-layout">
	<section class="panel settings-list-panel settings-primary-pane">
		<header class="panel-title-row">
			<div><h2>{_('Application Settings')}</h2><p>{_('All non-secret settings except database connectivity; secret-like keys are rejected.')}</p></div>
		</header>
		{#if isSettingsLoading && applicationSettings.length === 0}
			<div class="empty-panel fill">{_('Loading settings...')}</div>
		{:else if Object.entries(settingsByCategory).length === 0}
			<div class="empty-panel fill">{_('No application settings are declared yet.')}</div>
		{:else}
			<div class="settings-category-list">
				{#each Object.entries(settingsByCategory) as [category, settings]}
					<section class="settings-category">
						<header>
							<h3>{settingsCategoryLabelFn(category)}</h3>
							<span>{settings.length}</span>
						</header>
						{#each settings as setting}
							<form class="setting-row" onsubmit={(event) => { event.preventDefault(); void onSaveSetting(setting); }}>
								<div class="setting-copy">
									<strong>{setting.label}</strong>
									<p>{setting.description}</p>
									<div class="setting-meta-row">
										<code>{setting.setting_key}</code>
										{#if settingMetadataFlagFn(setting, 'bootstrap')}<em>Bootstrap</em>{/if}
										{#if settingMetadataFlagFn(setting, 'restart_required')}<em>Restart</em>{/if}
										{#if settingMetadataTextFn(setting, 'env_var')}<em>{settingMetadataTextFn(setting, 'env_var')}</em>{/if}
									</div>
								</div>
								<div class="setting-control">
									{#if settingAllowedValuesFn(setting).length}
										<HermesSelect
											value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)}
											options={settingSelectOptions(setting)}
											placeholder={_('Select value')}
											searchPlaceholder={_('Search values...')}
											emptyLabel={_('No options')}
											ariaLabel={setting.label}
											disabled={!setting.is_editable}
											onChange={(nextValue) => onUpdateSettingDraft(setting.setting_key, nextValue)}
										/>
									{:else if setting.value_kind === 'boolean'}
										<label class="setting-toggle">
											<input type="checkbox" checked={(settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)) === 'true'} disabled={!setting.is_editable} onchange={(event) => onUpdateSettingDraft(setting.setting_key, checkboxEventValueFn(event))} />
											<span>{(settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)) === 'true' ? _('Enabled') : _('Disabled')}</span>
										</label>
									{:else if setting.value_kind === 'integer'}
										<input type="number" value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)} min={String(setting.metadata.min ?? '')} max={String(setting.metadata.max ?? '')} step={String(setting.metadata.step ?? 1)} disabled={!setting.is_editable} oninput={(event) => onUpdateSettingDraft(setting.setting_key, inputEventValueFn(event))} />
									{:else if setting.value_kind === 'json' || settingControlFn(setting) === 'textarea'}
										<textarea value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)} disabled={!setting.is_editable} rows="4" oninput={(event) => onUpdateSettingDraft(setting.setting_key, inputEventValueFn(event))}></textarea>
									{:else}
										<input value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)} placeholder={String(setting.metadata.placeholder ?? '')} disabled={!setting.is_editable} oninput={(event) => onUpdateSettingDraft(setting.setting_key, inputEventValueFn(event))} />
									{/if}
									<button type="submit" disabled={!setting.is_editable || savingSettingKey === setting.setting_key || !settingHasChangedFn(setting)}>
										{savingSettingKey === setting.setting_key ? _('Saving') : _('Save')}
									</button>
								</div>
							</form>
						{/each}
					</section>
				{/each}
			</div>
		{/if}
	</section>

	<aside class="settings-rail">
		<section class="panel info-card">
			<h2>{_('Runtime Source')}</h2>
			<div class="health-row"><span>{_('Backend bind')}</span><strong>{settingValueTextFn('server.http_addr')}</strong></div>
			<div class="health-row"><span>{_('Frontend API')}</span><strong>{settingValueTextFn('frontend.api_base_url')}</strong></div>
			<div class="health-row"><span>{_('AI configuration')}</span><strong>{_('AI Control Center')}</strong></div>
			<div class="health-row"><span>{_('Model routing')}</span><strong>{_('Capability slots')}</strong></div>
		</section>
		<section class="panel info-card">
			<h2>{_('Boundaries')}</h2>
			<ul class="detail-list">
				<li>{_('PostgreSQL stores declared setting values')}<em>JSONB</em></li>
				<li>{_('AI providers, models and routes live in AI Control Center')}<em>{_('Domain tables')}</em></li>
				<li>{_('Legacy ai.* settings are bootstrap fallback only')}<em>{_('Hidden')}</em></li>
				<li>{_('Database URL stays outside the panel')}<em>{_('Bootstrap')}</em></li>
				<li>{_('API token and vault key stay outside DB')}<em>{_('Secret boundary')}</em></li>
				<li>{_('Credentials stay in encrypted vault')}<em>{_('No secret values')}</em></li>
				<li>{_('Settings updates are audited')}<em>{_('No values in audit')}</em></li>
			</ul>
		</section>
	</aside>
</div>
