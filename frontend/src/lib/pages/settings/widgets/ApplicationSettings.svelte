<script lang="ts">
	import Icon from '@iconify/svelte';
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
</script>

<div class="settings-layout">
	<section class="panel settings-list-panel settings-primary-pane">
		<header class="panel-title-row">
			<div><h2>Application Settings</h2><p>All non-secret settings except database connectivity; secret-like keys are rejected.</p></div>
		</header>
		{#if isSettingsLoading && applicationSettings.length === 0}
			<div class="empty-panel fill">Loading settings...</div>
		{:else if Object.entries(settingsByCategory).length === 0}
			<div class="empty-panel fill">No application settings are declared yet.</div>
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
										<select value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)} disabled={!setting.is_editable} onchange={(event) => onUpdateSettingDraft(setting.setting_key, inputEventValueFn(event))}>
											{#each settingAllowedValuesFn(setting) as value}
												<option value={value}>{settingsCategoryLabelFn(value)}</option>
											{/each}
										</select>
									{:else if setting.value_kind === 'boolean'}
										<label class="setting-toggle">
											<input type="checkbox" checked={(settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)) === 'true'} disabled={!setting.is_editable} onchange={(event) => onUpdateSettingDraft(setting.setting_key, checkboxEventValueFn(event))} />
											<span>{(settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)) === 'true' ? 'Enabled' : 'Disabled'}</span>
										</label>
									{:else if setting.value_kind === 'integer'}
										<input type="number" value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)} min={String(setting.metadata.min ?? '')} max={String(setting.metadata.max ?? '')} step={String(setting.metadata.step ?? 1)} disabled={!setting.is_editable} oninput={(event) => onUpdateSettingDraft(setting.setting_key, inputEventValueFn(event))} />
									{:else if setting.value_kind === 'json' || settingControlFn(setting) === 'textarea'}
										<textarea value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)} disabled={!setting.is_editable} rows="4" oninput={(event) => onUpdateSettingDraft(setting.setting_key, inputEventValueFn(event))}></textarea>
									{:else}
										<input value={settingDrafts[setting.setting_key] ?? settingDraftValueFn(setting)} placeholder={String(setting.metadata.placeholder ?? '')} disabled={!setting.is_editable} oninput={(event) => onUpdateSettingDraft(setting.setting_key, inputEventValueFn(event))} />
									{/if}
									<button type="submit" disabled={!setting.is_editable || savingSettingKey === setting.setting_key || !settingHasChangedFn(setting)}>
										{savingSettingKey === setting.setting_key ? 'Saving' : 'Save'}
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
			<h2>Runtime Source</h2>
			<div class="health-row"><span>Backend bind</span><strong>{settingValueTextFn('server.http_addr')}</strong></div>
			<div class="health-row"><span>Frontend API</span><strong>{settingValueTextFn('frontend.api_base_url')}</strong></div>
			<div class="health-row"><span>AI Provider</span><strong>{settingValueTextFn('ai.provider')}</strong></div>
			<div class="health-row"><span>Ollama URL</span><strong>{settingValueTextFn('ai.ollama_base_url')}</strong></div>
			<div class="health-row"><span>OmniRoute URL</span><strong>{settingValueTextFn('ai.omniroute_base_url')}</strong></div>
			<div class="health-row"><span>Chat</span><strong>{settingValueTextFn('ai.chat_model')} / {settingValueTextFn('ai.omniroute_chat_model')}</strong></div>
			<div class="health-row"><span>Embedding</span><strong>{settingValueTextFn('ai.embedding_model')} / {settingValueTextFn('ai.omniroute_embedding_model')}</strong></div>
		</section>
		<section class="panel info-card">
			<h2>Boundaries</h2>
			<ul class="detail-list">
				<li>PostgreSQL stores declared setting values<em>JSONB</em></li>
				<li>Database URL stays outside the panel<em>Bootstrap</em></li>
				<li>API token and vault key stay outside DB<em>Secret boundary</em></li>
				<li>Credentials stay in encrypted vault<em>No secret values</em></li>
				<li>Settings updates are audited<em>No values in audit</em></li>
			</ul>
		</section>
	</aside>
</div>
