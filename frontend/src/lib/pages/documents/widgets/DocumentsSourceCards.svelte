<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-source-cards" data-widget-hidden={!isWidgetVisible('documents-source-cards')}>
	<WidgetEditChrome widgetId="documents-source-cards" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="document-source-cards">
		{#each ['Google Drive 1,243', 'OneDrive 812', 'Dropbox 342', 'Notion 256'] as source, index}
			<div class="source-card">
				<span class="round-icon"><Icon icon={['tabler:brand-google-drive','tabler:brand-office','tabler:brand-dropbox','tabler:brand-notion'][index]} width="24" height="24" /></span>
				<strong>{source.split(' ')[0]}</strong>
				<em>{source.split(' ')[1]}</em>
			</div>
		{/each}
	</div>
</div>
