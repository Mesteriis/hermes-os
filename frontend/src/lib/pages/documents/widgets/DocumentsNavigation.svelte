<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-navigation" data-widget-hidden={!isWidgetVisible('documents-navigation')}>
	<WidgetEditChrome widgetId="documents-navigation" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<aside class="left-panels">
		<section class="panel info-card">
			<h2>{_('Smart Collections')}</h2>
			{#each ['Recently Added 48', 'Recently Opened 24', 'Important 32', 'Shared with Me 18', 'Requires Review 7', 'Contracts & Legal 23', 'Financial 15'] as item}
				<div class="collection-row">{_(item)}</div>
			{/each}
		</section>
		<section class="panel info-card">
			<h2>{_('My Folders')}</h2>
			{#each ['Hermes Hub', 'Projects', 'Personal', 'Work', 'Archive 2024', 'Clients', 'References'] as folder}
				<div class="collection-row"><Icon icon="tabler:folder" width="16" height="16" />{_(folder)}</div>
			{/each}
		</section>
	</aside>
</div>
