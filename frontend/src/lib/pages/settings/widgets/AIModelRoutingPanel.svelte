<script lang="ts">
	import Icon from '@iconify/svelte';
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		aiCapabilitySlots,
		aiModels,
		aiProviders,
		aiRouteSearch,
		isAiSettingsSaving,
		saveAiModelRoute,
		selectedRouteDrafts,
		updateRouteDraft
	} from '$lib/stores/aiSettings';
	import {
		buildModelSelectGroups,
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
		<h3>{_('Model Routing')}</h3>
		<p>{_('Assign provider models to capability slots with searchable grouped model selectors.')}</p>
	</header>
	<label class="ai-search-box">
		<Icon icon="tabler:search" width="16" height="16" />
		<input
			value={$aiRouteSearch}
			placeholder={_('Search models...')}
			oninput={(event) => aiRouteSearch.set((event.currentTarget as HTMLInputElement).value)}
		/>
	</label>
	<div class="ai-routing-list">
		{#each $aiCapabilitySlots as slot}
			{@const groups = modelGroupsToHermesGroups(buildModelSelectGroups($aiModels, $aiProviders, $aiRouteSearch, slot))}
			<form class="ai-route-editor" onsubmit={(event) => { event.preventDefault(); void saveAiModelRoute(slot.slot); }}>
				<div>
					<strong>{_(slot.label)}</strong>
					<small>{_(slot.description)}</small>
					{#if slot.requires_embedding_dimension}
						<em>{slot.requires_embedding_dimension} dims required</em>
					{/if}
				</div>
				<HermesSelect
					value={$selectedRouteDrafts[slot.slot] ?? ''}
					groups={groups}
					placeholder={_('Select model')}
					searchPlaceholder={_('Search models...')}
					emptyLabel={_('No options')}
					ariaLabel={_(slot.label)}
					onChange={(nextValue) => updateRouteDraft(slot.slot, nextValue)}
				/>
				<button type="submit" class="primary-button" disabled={!$selectedRouteDrafts[slot.slot] || $isAiSettingsSaving}>{_('Save')}</button>
			</form>
		{/each}
	</div>
</section>
