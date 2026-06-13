<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import {
		aiCapabilitySlots,
		aiModels,
		aiPrompts,
		aiProviders,
		aiRoutes
	} from '$lib/stores/aiSettings';

	const _ = (key: string) => t($currentLocale, key);

	function routeModelLabel(slot: string): string {
		const route = $aiRoutes.find((item) => item.capability_slot === slot);
		if (!route) return 'Not assigned';
		const model = $aiModels.find(
			(item) => item.provider_id === route.provider_id && item.model_key === route.model_key
		);
		return model ? `${model.display_name} / ${route.provider_id}` : route.model_key;
	}
</script>

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
	<header>
		<h3>{_('Capability Routes')}</h3>
		<p>{_('Hermes resolves models by capability slot, not by one global model.')}</p>
	</header>
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
