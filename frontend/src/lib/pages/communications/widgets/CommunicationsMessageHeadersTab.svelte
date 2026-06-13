<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	type HeaderEntry = {
		name: string;
		value: string;
	};

	interface Props {
		headers: HeaderEntry[];
	}

	let { headers }: Props = $props();
</script>

<div class="headers-table">
	{#each headers.slice(0, 24) as header}
		<span>{header.name}</span><strong>{header.value}</strong>
	{:else}
		<p>{_('No headers available')}</p>
	{/each}
</div>
