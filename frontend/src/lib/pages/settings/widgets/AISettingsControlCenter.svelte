<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		aiCapabilitySlots,
		aiModels,
		aiPromptEvalRuns,
		aiPrompts,
		aiProviderPresets,
		aiProviders,
		aiRouteSearch,
		aiRoutes,
		aiSettingsActionMessage,
		aiSettingsError,
		apiProviders,
		builtInProviders,
		cliProviders,
		createAiPrompt,
		createAiPromptVersion,
		createAiProviderFromPreset,
		customPrompts,
		grantAiProviderConsent,
		isAiSettingsLoading,
		isAiSettingsSaving,
		loadAiSettingsControlCenter,
		saveAiModelRoute,
		selectedAiSettingsPanel,
		selectedRouteDrafts,
		syncAiProviderModels,
		systemPrompts,
		testAiPrompt,
		testAiProvider,
		updateRouteDraft
	} from '$lib/stores/aiSettings';
	import {
		AI_SETTINGS_SECTIONS,
		PROMPT_ENTITY_SCOPES,
		buildModelSelectGroups,
		categoryLabel,
		modelFromSelectValue,
		modelSelectValue,
		providerPrivacyLabel,
		providerStatusTone,
		type AiSettingsPanel
	} from '$lib/services/aiSettings';

	const _ = (key: string) => t($currentLocale, key);

	let apiPresetKey = $state('omniroute');
	let apiDisplayName = $state('OmniRoute');
	let apiBaseUrl = $state('https://ai.sh-inc.ru/v1');
	let apiKey = $state('');
	let apiConsent = $state(false);
	let cliPresetKey = $state('codex');
	let promptName = $state('');
	let promptScope = $state('communication');
	let promptCapability = $state('mail_intelligence');
	let promptDescription = $state('');
	let promptVersionPromptId = $state('');
	let promptVersionLabel = $state('');
	let promptVersionBody = $state('');
	let promptVersionVariables = $state('');
	let promptTestPromptId = $state('');
	let promptTestVersionId = $state('');
	let promptTestModelValue = $state('');
	let promptTestVariablesJson = $state('{"entity":"Communication","summary":"Needs reply"}');
	let promptTestOutput = $state('');

	let selectedApiPreset = $derived(
		$aiProviderPresets.find(
			(preset) => preset.provider_kind === 'api' && preset.provider_key === apiPresetKey
		) ?? null
	);
	let selectedCliPreset = $derived(
		$aiProviderPresets.find(
			(preset) => preset.provider_kind === 'cli' && preset.provider_key === cliPresetKey
		) ?? null
	);
	let allModelGroups = $derived(buildModelSelectGroups($aiModels, $aiProviders, $aiRouteSearch));

	onMount(() => {
		if ($aiProviders.length === 0 && !$isAiSettingsLoading) {
			void loadAiSettingsControlCenter();
		}
	});

	$effect(() => {
		if (selectedApiPreset) {
			apiDisplayName = apiDisplayName || selectedApiPreset.display_name;
			apiBaseUrl = apiBaseUrl || selectedApiPreset.base_url || '';
		}
	});

	$effect(() => {
		if (!promptVersionPromptId && $aiPrompts.length > 0) {
			promptVersionPromptId = $aiPrompts.find((prompt) => !prompt.is_system)?.prompt_id ?? $aiPrompts[0].prompt_id;
		}
		if (!promptTestPromptId && $aiPrompts.length > 0) {
			promptTestPromptId = $aiPrompts[0].prompt_id;
		}
		if (!promptTestModelValue && $aiModels.length > 0) {
			promptTestModelValue = modelSelectValue($aiModels[0]);
		}
	});

	function selectPanel(panel: AiSettingsPanel) {
		selectedAiSettingsPanel.set(panel);
	}

	function selectApiPreset(key: string) {
		apiPresetKey = key;
		const preset = $aiProviderPresets.find(
			(item) => item.provider_kind === 'api' && item.provider_key === key
		);
		if (preset) {
			apiDisplayName = preset.display_name;
			apiBaseUrl = preset.base_url ?? '';
			apiConsent = preset.privacy !== 'remote';
		}
	}

	async function submitApiProvider() {
		if (!selectedApiPreset) return;
		await createAiProviderFromPreset(selectedApiPreset, {
			display_name: apiDisplayName,
			base_url: apiBaseUrl,
			api_key: apiKey,
			remote_context_consent: apiConsent
		});
		apiKey = '';
	}

	async function submitCliProvider() {
		if (!selectedCliPreset) return;
		await createAiProviderFromPreset(selectedCliPreset);
	}

	async function submitPrompt() {
		if (!promptName.trim()) return;
		await createAiPrompt({
			name: promptName.trim(),
			entity_scope: promptScope,
			capability_slot: promptCapability,
			description: promptDescription.trim() || undefined
		});
		promptName = '';
		promptDescription = '';
	}

	async function submitPromptVersion() {
		if (!promptVersionPromptId || !promptVersionBody.trim()) return;
		await createAiPromptVersion(promptVersionPromptId, {
			version_label: promptVersionLabel.trim() || undefined,
			body_template: promptVersionBody,
			variables: promptVersionVariables
				.split(',')
				.map((item) => item.trim())
				.filter(Boolean)
		});
		promptVersionLabel = '';
		promptVersionBody = '';
		promptVersionVariables = '';
	}

	async function submitPromptTest() {
		const model = modelFromSelectValue(promptTestModelValue, $aiModels);
		if (!promptTestPromptId || !model) return;
		let variables: Record<string, unknown> = {};
		try {
			variables = JSON.parse(promptTestVariablesJson || '{}') as Record<string, unknown>;
		} catch {
			promptTestOutput = 'Invalid variables JSON';
			return;
		}
		const run = await testAiPrompt(promptTestPromptId, {
			prompt_version_id: promptTestVersionId.trim() || undefined,
			provider_id: model.provider_id,
			model_key: model.model_key,
			variables,
			source_refs: []
		});
		promptTestOutput = run?.output_text ?? '';
	}

	function routeModelLabel(slot: string): string {
		const route = $aiRoutes.find((item) => item.capability_slot === slot);
		if (!route) return 'Not assigned';
		const model = $aiModels.find(
			(item) => item.provider_id === route.provider_id && item.model_key === route.model_key
		);
		return model ? `${model.display_name} / ${route.provider_id}` : route.model_key;
	}
</script>

<div class="ai-settings-layout">
	<section class="ai-settings-main">
		<header class="settings-workbench-header">
			<div>
				<h2>{_('AI Control Center')}</h2>
				<p>{_('Providers, model routes and prompts for Hermes capabilities.')}</p>
			</div>
			<div class="ai-settings-actions">
				<button type="button" class="ghost-button" onclick={() => void loadAiSettingsControlCenter()} disabled={$isAiSettingsLoading}>
					<Icon icon="tabler:refresh" width="16" height="16" />{_('Refresh')}
				</button>
			</div>
		</header>

		<div class="ai-settings-body">
			<nav class="ai-settings-tabs" aria-label={_('AI settings sections')}>
				{#each AI_SETTINGS_SECTIONS as section}
					<button
						type="button"
						class:active={$selectedAiSettingsPanel === section.id}
						onclick={() => selectPanel(section.id)}
					>
						<Icon icon={section.icon} width="16" height="16" />
						<span>{_(section.label)}</span>
					</button>
				{/each}
			</nav>

			{#if $aiSettingsActionMessage}
				<p class="setup-state success">{$aiSettingsActionMessage}</p>
			{/if}
			{#if $aiSettingsError}
				<p class="inline-error">{$aiSettingsError}</p>
			{/if}

			{#if $isAiSettingsLoading && $aiProviders.length === 0}
				<div class="empty-panel fill">{_('Loading AI settings.')}</div>
			{:else if $selectedAiSettingsPanel === 'overview'}
				<section class="ai-overview-grid">
					<article class="ai-control-card">
						<span>{_('Providers')}</span>
						<strong>{$aiProviders.length}</strong>
						<small>{_('Built-in, CLI and API accounts')}</small>
					</article>
					<article class="ai-control-card">
						<span>{_('Models')}</span>
						<strong>{$aiModels.length}</strong>
						<small>{_('Live and curated inventory')}</small>
					</article>
					<article class="ai-control-card">
						<span>{_('Routes')}</span>
						<strong>{$aiRoutes.length}</strong>
						<small>{_('Capability slot assignments')}</small>
					</article>
					<article class="ai-control-card">
						<span>{_('Prompts')}</span>
						<strong>{$aiPrompts.length}</strong>
						<small>{_('System and custom templates')}</small>
					</article>
				</section>

				<section class="ai-panel-section">
					<header><h3>{_('Capability Routes')}</h3><p>{_('Hermes resolves models by capability slot, not by one global model.')}</p></header>
					<div class="ai-route-summary">
						{#each $aiCapabilitySlots as slot}
							<div class="ai-route-line">
								<div>
									<strong>{_(slot.label)}</strong>
									<small>{_(slot.description)}</small>
								</div>
								<em>{routeModelLabel(slot.slot)}</em>
							</div>
						{/each}
					</div>
				</section>
			{:else if $selectedAiSettingsPanel === 'built_in'}
				<section class="ai-panel-section">
					<header><h3>{_('Built-in Ollama')}</h3><p>{_('Local runtime is managed by Hermes; model downloads require explicit confirmation.')}</p></header>
					<div class="ai-provider-grid">
						{#each $builtInProviders as provider}
							<article class="ai-provider-card">
								<div class="ai-provider-title">
									<span class={`integration-status ${providerStatusTone(provider)}`}>{provider.status}</span>
									<strong>{provider.display_name}</strong>
									<small>{providerPrivacyLabel(provider)} · {provider.provider_id}</small>
								</div>
								<div class="ai-provider-actions">
									<button type="button" class="ghost-button" onclick={() => void testAiProvider(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Test')}</button>
									<button type="button" class="ghost-button" onclick={() => void syncAiProviderModels(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Sync models')}</button>
								</div>
							</article>
						{:else}
							<div class="empty-panel fill">{_('Built-in Ollama provider will appear after migration repair runs.')}</div>
						{/each}
					</div>
				</section>
			{:else if $selectedAiSettingsPanel === 'cli'}
				<section class="ai-panel-section">
					<header><h3>{_('CLI Agents')}</h3><p>{_('Provider bridges use allowlisted fixed commands and do not run autonomous workflows.')}</p></header>
					<form class="ai-wizard-row" onsubmit={(event) => { event.preventDefault(); void submitCliProvider(); }}>
						<label>
							<span>{_('Preset')}</span>
							<select value={cliPresetKey} onchange={(event) => (cliPresetKey = (event.currentTarget as HTMLSelectElement).value)}>
								{#each $aiProviderPresets.filter((preset) => preset.provider_kind === 'cli') as preset}
									<option value={preset.provider_key}>{preset.display_name}</option>
								{/each}
							</select>
						</label>
						<button type="submit" class="primary-button" disabled={!selectedCliPreset || $isAiSettingsSaving}>{_('Add CLI bridge')}</button>
					</form>
					<div class="ai-provider-grid">
						{#each $cliProviders as provider}
							<article class="ai-provider-card">
								<div class="ai-provider-title">
									<span class={`integration-status ${providerStatusTone(provider)}`}>{provider.status}</span>
									<strong>{provider.display_name}</strong>
									<small>{String(provider.config.command_preset ?? provider.provider_key)} · fixed argv bridge</small>
								</div>
								<div class="ai-provider-actions">
									<button type="button" class="ghost-button" onclick={() => void testAiProvider(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Test')}</button>
								</div>
							</article>
						{/each}
					</div>
				</section>
			{:else if $selectedAiSettingsPanel === 'api'}
				<section class="ai-panel-section">
					<header><h3>{_('API Providers')}</h3><p>{_('Remote providers are opt-in and API keys are written only to the host vault.')}</p></header>
					<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitApiProvider(); }}>
						<label>
							<span>{_('Preset')}</span>
							<select value={apiPresetKey} onchange={(event) => selectApiPreset((event.currentTarget as HTMLSelectElement).value)}>
								{#each $aiProviderPresets.filter((preset) => preset.provider_kind === 'api') as preset}
									<option value={preset.provider_key}>{preset.display_name}</option>
								{/each}
							</select>
						</label>
						<label>
							<span>{_('Display name')}</span>
							<input value={apiDisplayName} oninput={(event) => (apiDisplayName = (event.currentTarget as HTMLInputElement).value)} />
						</label>
						<label>
							<span>{_('Base URL')}</span>
							<input value={apiBaseUrl} oninput={(event) => (apiBaseUrl = (event.currentTarget as HTMLInputElement).value)} />
						</label>
						<label>
							<span>{_('API key')}</span>
							<input type="password" value={apiKey} autocomplete="off" oninput={(event) => (apiKey = (event.currentTarget as HTMLInputElement).value)} />
						</label>
						<label class="ai-consent-toggle">
							<input type="checkbox" checked={apiConsent} onchange={(event) => (apiConsent = (event.currentTarget as HTMLInputElement).checked)} />
							<span>{_('Allow this provider to receive selected remote context')}</span>
						</label>
						<button type="submit" class="primary-button" disabled={!selectedApiPreset || !apiConsent || $isAiSettingsSaving}>{_('Connect API provider')}</button>
					</form>
					<div class="ai-provider-grid">
						{#each $apiProviders as provider}
							<article class="ai-provider-card">
								<div class="ai-provider-title">
									<span class={`integration-status ${providerStatusTone(provider)}`}>{provider.status}</span>
									<strong>{provider.display_name}</strong>
									<small>{provider.provider_key} · {provider.consent_state}</small>
								</div>
								<div class="ai-provider-actions">
									{#if provider.consent_state !== 'granted'}
										<button type="button" class="ghost-button" onclick={() => void grantAiProviderConsent(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Grant consent')}</button>
									{/if}
									<button type="button" class="ghost-button" onclick={() => void testAiProvider(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Test')}</button>
									<button type="button" class="ghost-button" onclick={() => void syncAiProviderModels(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Sync models')}</button>
								</div>
							</article>
						{/each}
					</div>
				</section>
			{:else if $selectedAiSettingsPanel === 'routing'}
				<section class="ai-panel-section">
					<header><h3>{_('Model Routing')}</h3><p>{_('Assign provider models to capability slots with searchable grouped model selectors.')}</p></header>
					<label class="ai-search-box">
						<Icon icon="tabler:search" width="16" height="16" />
						<input value={$aiRouteSearch} placeholder={_('Search models...')} oninput={(event) => aiRouteSearch.set((event.currentTarget as HTMLInputElement).value)} />
					</label>
					<div class="ai-routing-list">
						{#each $aiCapabilitySlots as slot}
							{@const groups = buildModelSelectGroups($aiModels, $aiProviders, $aiRouteSearch, slot)}
							<form class="ai-route-editor" onsubmit={(event) => { event.preventDefault(); void saveAiModelRoute(slot.slot); }}>
								<div>
									<strong>{_(slot.label)}</strong>
									<small>{_(slot.description)}</small>
									{#if slot.requires_embedding_dimension}
										<em>{slot.requires_embedding_dimension} dims required</em>
									{/if}
								</div>
								<select value={$selectedRouteDrafts[slot.slot] ?? ''} onchange={(event) => updateRouteDraft(slot.slot, (event.currentTarget as HTMLSelectElement).value)}>
									<option value="">{_('Select model')}</option>
									{#each groups as group}
										<optgroup label={group.label}>
											{#each group.options as option}
												<option value={option.value} disabled={Boolean(option.disabledReason)}>
													{option.providerLabel} / {option.model.display_name} · {option.privacyLabel}{option.disabledReason ? ` · ${option.disabledReason}` : ''}
												</option>
											{/each}
										</optgroup>
									{/each}
								</select>
								<button type="submit" class="primary-button" disabled={!$selectedRouteDrafts[slot.slot] || $isAiSettingsSaving}>{_('Save')}</button>
							</form>
						{/each}
					</div>
				</section>
			{:else if $selectedAiSettingsPanel === 'prompts'}
				<section class="ai-panel-section">
					<header><h3>{_('Prompt Studio')}</h3><p>{_('System prompts are read-only; custom prompts can be versioned and evaluated.')}</p></header>
					<div class="ai-prompt-workbench">
						<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitPrompt(); }}>
							<label><span>{_('Name')}</span><input value={promptName} oninput={(event) => (promptName = (event.currentTarget as HTMLInputElement).value)} /></label>
							<label>
								<span>{_('Entity')}</span>
								<select value={promptScope} onchange={(event) => (promptScope = (event.currentTarget as HTMLSelectElement).value)}>
									{#each PROMPT_ENTITY_SCOPES as scope}
										<option value={scope}>{categoryLabel(scope)}</option>
									{/each}
								</select>
							</label>
							<label>
								<span>{_('Capability')}</span>
								<select value={promptCapability} onchange={(event) => (promptCapability = (event.currentTarget as HTMLSelectElement).value)}>
									{#each $aiCapabilitySlots as slot}
										<option value={slot.slot}>{_(slot.label)}</option>
									{/each}
								</select>
							</label>
							<label><span>{_('Description')}</span><input value={promptDescription} oninput={(event) => (promptDescription = (event.currentTarget as HTMLInputElement).value)} /></label>
							<button type="submit" class="primary-button" disabled={!promptName.trim() || $isAiSettingsSaving}>{_('Create prompt')}</button>
						</form>

						<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitPromptVersion(); }}>
							<label>
								<span>{_('Prompt')}</span>
								<select value={promptVersionPromptId} onchange={(event) => (promptVersionPromptId = (event.currentTarget as HTMLSelectElement).value)}>
									{#each $aiPrompts.filter((prompt) => !prompt.is_system) as prompt}
										<option value={prompt.prompt_id}>{prompt.name}</option>
									{/each}
								</select>
							</label>
							<label><span>{_('Version label')}</span><input value={promptVersionLabel} oninput={(event) => (promptVersionLabel = (event.currentTarget as HTMLInputElement).value)} /></label>
							<label><span>{_('Variables')}</span><input value={promptVersionVariables} placeholder="entity,summary,question" oninput={(event) => (promptVersionVariables = (event.currentTarget as HTMLInputElement).value)} /></label>
							<label class="wide"><span>{_('Prompt body')}</span><textarea rows="6" value={promptVersionBody} oninput={(event) => (promptVersionBody = (event.currentTarget as HTMLTextAreaElement).value)}></textarea></label>
							<button type="submit" class="primary-button" disabled={!promptVersionPromptId || !promptVersionBody.trim() || $isAiSettingsSaving}>{_('Save version')}</button>
						</form>

						<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitPromptTest(); }}>
							<label>
								<span>{_('Prompt')}</span>
								<select value={promptTestPromptId} onchange={(event) => (promptTestPromptId = (event.currentTarget as HTMLSelectElement).value)}>
									{#each $aiPrompts as prompt}
										<option value={prompt.prompt_id}>{prompt.name}</option>
									{/each}
								</select>
							</label>
							<label><span>{_('Version id')}</span><input value={promptTestVersionId} oninput={(event) => (promptTestVersionId = (event.currentTarget as HTMLInputElement).value)} /></label>
							<label>
								<span>{_('Model')}</span>
								<select value={promptTestModelValue} onchange={(event) => (promptTestModelValue = (event.currentTarget as HTMLSelectElement).value)}>
									{#each allModelGroups as group}
										<optgroup label={group.label}>
											{#each group.options as option}
												<option value={option.value} disabled={Boolean(option.disabledReason)}>{option.providerLabel} / {option.model.display_name}</option>
											{/each}
										</optgroup>
									{/each}
								</select>
							</label>
							<label class="wide"><span>{_('Variables JSON')}</span><textarea rows="4" value={promptTestVariablesJson} oninput={(event) => (promptTestVariablesJson = (event.currentTarget as HTMLTextAreaElement).value)}></textarea></label>
							<button type="submit" class="primary-button" disabled={!promptTestPromptId || !promptTestModelValue || $isAiSettingsSaving}>{_('Run test')}</button>
							{#if promptTestOutput}
								<pre class="ai-prompt-output">{promptTestOutput}</pre>
							{/if}
						</form>
					</div>
				</section>
			{:else}
				<section class="ai-panel-section">
					<header><h3>{_('Runs / health')}</h3><p>{_('Prompt evaluation runs and provider health signals.')}</p></header>
					<div class="ai-routing-list">
						{#each $aiPromptEvalRuns as run}
							<article class="ai-route-line">
								<div>
									<strong>{run.prompt_id}</strong>
									<small>{run.provider_id} / {run.model_key}</small>
								</div>
								<em>{run.score ?? 'No score'}</em>
							</article>
						{:else}
							<div class="empty-panel fill">{_('No prompt eval runs yet.')}</div>
						{/each}
					</div>
				</section>
			{/if}
		</div>
	</section>

	<aside class="settings-rail ai-settings-rail">
		<section class="panel info-card">
			<h2>{_('AI Boundary')}</h2>
			<ul class="detail-list">
				<li>{_('API keys')}<em>{_('Host vault only')}</em></li>
				<li>{_('Remote providers')}<em>{_('Explicit consent')}</em></li>
				<li>{_('CLI agents')}<em>{_('Fixed argv bridge')}</em></li>
				<li>{_('Embeddings')}<em>2560 dims</em></li>
			</ul>
		</section>
		<section class="panel info-card">
			<h2>{_('Providers')}</h2>
			<div class="health-row"><span>{_('Built-in')}</span><strong>{$builtInProviders.length}</strong></div>
			<div class="health-row"><span>{_('CLI')}</span><strong>{$cliProviders.length}</strong></div>
			<div class="health-row"><span>{_('API')}</span><strong>{$apiProviders.length}</strong></div>
		</section>
		<section class="panel info-card">
			<h2>{_('Prompt Library')}</h2>
			<div class="health-row"><span>{_('System')}</span><strong>{$systemPrompts.length}</strong></div>
			<div class="health-row"><span>{_('Custom')}</span><strong>{$customPrompts.length}</strong></div>
			<div class="health-row"><span>{_('Evaluations')}</span><strong>{$aiPromptEvalRuns.length}</strong></div>
		</section>
	</aside>
</div>
