<script lang="ts">
	import '$lib/styles/tokens.css';
	import { currentLocale, t } from '$lib/i18n';
	import '$lib/styles/app.css';

	let { children } = $props();
	const _ = (key: string) => t($currentLocale, key);
</script>

<div class="viewport-guard" role="alert" aria-live="polite">
	<strong>{_('Minimum window size is 800 x 600')}</strong>
	<span>{_('Increase the Hermes Hub window size to continue.')}</span>
</div>

{@render children()}
