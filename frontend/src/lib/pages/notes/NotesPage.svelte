<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import NotesSourceFilters from './widgets/NotesSourceFilters.svelte';
	import NotesList from './widgets/NotesList.svelte';
	import NotesInsights from './widgets/NotesInsights.svelte';

	const _ = (key: string) => t($currentLocale, key);

	type Note = { title: string; body: string; source: string; tag: string; time: string; icon: string };

	interface Props {
		notes: Note[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { notes, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<section class="notes-page">
	<div class="view-header">
		<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:notes" width="28" height="28" /></span><div><h1>Notes</h1><p>All your notes from connected sources</p></div></div>
	</div>
	<div class="notes-layout">
		<NotesSourceFilters {isLayoutEditing} {isWidgetVisible} />
		<NotesList {notes} {isLayoutEditing} {isWidgetVisible} />
		<aside class="stacked-rail">
			<NotesInsights {isLayoutEditing} {isWidgetVisible} />
		</aside>
	</div>
</section>
