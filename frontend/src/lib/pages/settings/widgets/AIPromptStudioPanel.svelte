<script lang="ts">
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { AiPromptTemplate } from '$lib/api';
	import {
		aiCapabilitySlots,
		aiModels,
		aiPrompts,
		aiProviders,
		createAiPrompt,
		createAiPromptVersion,
		customPrompts,
		isAiSettingsSaving,
		testAiPrompt
	} from '$lib/stores/aiSettings';
	import {
		PROMPT_ENTITY_SCOPES,
		buildModelSelectGroups,
		categoryLabel,
		modelFromSelectValue,
		modelSelectValue,
		type ModelSelectGroup
	} from '$lib/services/aiSettings';

	const _ = (key: string) => t($currentLocale, key);

	type HermesSelectOption = {
		value: string;
		label: string;
		eyebrow?: string;
		description?: string;
		meta?: string;
		disabled?: boolean;
		disabledReason?: string;
	};

	type HermesSelectGroup = {
		label: string;
		options: HermesSelectOption[];
	};

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

	let allModelGroups = $derived(buildModelSelectGroups($aiModels, $aiProviders));
	let promptScopeOptions = $derived(
		PROMPT_ENTITY_SCOPES.map((scope) => ({
			value: scope,
			label: categoryLabel(scope)
		}))
	);
	let promptCapabilityOptions = $derived(
		$aiCapabilitySlots.map((slot) => ({
			value: slot.slot,
			label: _(slot.label),
			description: _(slot.description),
			meta: slot.requires_embedding_dimension
				? `${slot.requires_embedding_dimension} ${_('dimensions required')}`
				: undefined
		}))
	);
	let customPromptOptions = $derived(
		$customPrompts.map((prompt) => promptSelectOption(prompt))
	);
	let allPromptOptions = $derived($aiPrompts.map((prompt) => promptSelectOption(prompt)));

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

	function promptSelectOption(prompt: AiPromptTemplate): HermesSelectOption {
		return {
			value: prompt.prompt_id,
			label: prompt.name,
			description: categoryLabel(prompt.entity_scope),
			meta: prompt.is_system ? _('System') : _('Custom')
		};
	}

	function modelGroupsToHermesGroups(groups: ModelSelectGroup[]): HermesSelectGroup[] {
		return groups.map((group) => ({
			label: group.label,
			options: group.options.map((option) => ({
				value: option.value,
				label: `${option.providerLabel} / ${option.model.display_name}`,
				eyebrow: option.privacyLabel,
				description: option.model.model_key,
				meta: option.capabilityLabel,
				disabled: Boolean(option.disabledReason),
				disabledReason: option.disabledReason
			}))
		}));
	}
</script>

<section class="ai-panel-section">
	<header>
		<h3>{_('Prompt Studio')}</h3>
		<p>{_('System prompts are read-only; custom prompts can be versioned and evaluated.')}</p>
	</header>
	<div class="ai-prompt-workbench">
		<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitPrompt(); }}>
			<label><span>{_('Name')}</span><input value={promptName} oninput={(event) => (promptName = (event.currentTarget as HTMLInputElement).value)} /></label>
			<label>
				<span>{_('Entity')}</span>
				<HermesSelect
					value={promptScope}
					options={promptScopeOptions}
					placeholder={_('Entity')}
					searchPlaceholder={_('Search entities...')}
					emptyLabel={_('No options')}
					ariaLabel={_('Entity')}
					onChange={(nextValue) => (promptScope = nextValue)}
				/>
			</label>
			<label>
				<span>{_('Capability')}</span>
				<HermesSelect
					value={promptCapability}
					options={promptCapabilityOptions}
					placeholder={_('Capability')}
					searchPlaceholder={_('Search capabilities...')}
					emptyLabel={_('No options')}
					ariaLabel={_('Capability')}
					onChange={(nextValue) => (promptCapability = nextValue)}
				/>
			</label>
			<label><span>{_('Description')}</span><input value={promptDescription} oninput={(event) => (promptDescription = (event.currentTarget as HTMLInputElement).value)} /></label>
			<button type="submit" class="primary-button" disabled={!promptName.trim() || $isAiSettingsSaving}>{_('Create prompt')}</button>
		</form>

		<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitPromptVersion(); }}>
			<label>
				<span>{_('Prompt')}</span>
				<HermesSelect
					value={promptVersionPromptId}
					options={customPromptOptions}
					placeholder={_('Select prompt')}
					searchPlaceholder={_('Search prompts...')}
					emptyLabel={_('No options')}
					ariaLabel={_('Prompt')}
					onChange={(nextValue) => (promptVersionPromptId = nextValue)}
				/>
			</label>
			<label><span>{_('Version label')}</span><input value={promptVersionLabel} oninput={(event) => (promptVersionLabel = (event.currentTarget as HTMLInputElement).value)} /></label>
			<label><span>{_('Variables')}</span><input value={promptVersionVariables} placeholder="entity,summary,question" oninput={(event) => (promptVersionVariables = (event.currentTarget as HTMLInputElement).value)} /></label>
			<label class="wide"><span>{_('Prompt body')}</span><textarea rows="6" value={promptVersionBody} oninput={(event) => (promptVersionBody = (event.currentTarget as HTMLTextAreaElement).value)}></textarea></label>
			<button type="submit" class="primary-button" disabled={!promptVersionPromptId || !promptVersionBody.trim() || $isAiSettingsSaving}>{_('Save version')}</button>
		</form>

		<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitPromptTest(); }}>
			<label>
				<span>{_('Prompt')}</span>
				<HermesSelect
					value={promptTestPromptId}
					options={allPromptOptions}
					placeholder={_('Select prompt')}
					searchPlaceholder={_('Search prompts...')}
					emptyLabel={_('No options')}
					ariaLabel={_('Prompt')}
					onChange={(nextValue) => (promptTestPromptId = nextValue)}
				/>
			</label>
			<label><span>{_('Version id')}</span><input value={promptTestVersionId} oninput={(event) => (promptTestVersionId = (event.currentTarget as HTMLInputElement).value)} /></label>
			<label>
				<span>{_('Model')}</span>
				<HermesSelect
					value={promptTestModelValue}
					groups={modelGroupsToHermesGroups(allModelGroups)}
					placeholder={_('Select model')}
					searchPlaceholder={_('Search models...')}
					emptyLabel={_('No options')}
					ariaLabel={_('Model')}
					onChange={(nextValue) => (promptTestModelValue = nextValue)}
				/>
			</label>
			<label class="wide"><span>{_('Variables JSON')}</span><textarea rows="4" value={promptTestVariablesJson} oninput={(event) => (promptTestVariablesJson = (event.currentTarget as HTMLTextAreaElement).value)}></textarea></label>
			<button type="submit" class="primary-button" disabled={!promptTestPromptId || !promptTestModelValue || $isAiSettingsSaving}>{_('Run test')}</button>
			{#if promptTestOutput}
				<pre class="ai-prompt-output">{promptTestOutput}</pre>
			{/if}
		</form>
	</div>
</section>
