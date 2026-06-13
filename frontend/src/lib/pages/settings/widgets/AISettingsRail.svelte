<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import {
		apiProviders,
		builtInProviders,
		cliProviders,
		customPrompts,
		aiPromptEvalRuns,
		systemPrompts
	} from '$lib/stores/aiSettings';

	const _ = (key: string) => t($currentLocale, key);
</script>

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
