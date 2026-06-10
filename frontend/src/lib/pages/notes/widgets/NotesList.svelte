<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	type Note = { title: string; body: string; source: string; tag: string; time: string; icon: string };

	interface Props {
		notes: Note[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { notes, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-list" data-widget-hidden={!isWidgetVisible('notes-list')}>
	<WidgetEditChrome widgetId="notes-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="notes-main-list">
		<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search notes..." /></label>
		{#each notes as note}
			<article class="note-card">
				<span class="round-icon"><Icon icon={note.icon} width="22" height="22" /></span>
				<div>
					<strong>{note.title}</strong>
					<p>{note.body}</p>
					<div class="note-meta">
						<span>{note.source}</span><em>{note.tag}</em><time>{note.time}</time>
					</div>
				</div>
			</article>
		{/each}
	</div>
</div>
