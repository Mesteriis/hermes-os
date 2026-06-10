<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	type Doc = { name: string; source: string; project: string; type: string; date: string; size: string; icon: string; tone: string };

	interface Props {
		documents: Doc[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { documents, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-list" data-widget-hidden={!isWidgetVisible('documents-list')}>
	<WidgetEditChrome widgetId="documents-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="document-main-list">
		<div class="document-filter-bar">
			<div class="segmented"><button type="button" class="segmented active">All</button><button type="button" class="segmented">Shared</button><button type="button" class="segmented">Recent</button></div>
			<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search documents..." /></label>
		</div>
		{#each documents as doc}
			<article class="document-row">
				<span class={`round-icon ${doc.tone}`}><Icon icon={doc.icon} width="20" height="20" /></span>
				<div><strong>{doc.name}</strong><small>{doc.source} · {doc.project} · {doc.type} · {doc.size}</small></div>
				<time>{doc.date}</time>
			</article>
		{/each}
	</div>
</div>
