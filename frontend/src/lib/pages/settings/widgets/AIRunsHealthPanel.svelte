<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import { aiPromptEvalRuns } from '$lib/stores/aiSettings';

	const _ = (key: string) => t($currentLocale, key);
</script>

<section class="ai-panel-section">
	<header>
		<h3>{_('Runs / health')}</h3>
		<p>{_('Prompt evaluation runs and provider health signals.')}</p>
	</header>
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
