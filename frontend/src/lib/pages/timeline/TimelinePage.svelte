<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import TimelineStream from './widgets/TimelineStream.svelte';
	import TimelineFilters from './widgets/TimelineFilters.svelte';
	import { communicationMessages } from '$lib/stores/communications';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let {
		isLayoutEditing,
		isWidgetVisible
	}: Props = $props();
</script>

<section class="timeline-page">
	<div class="view-header"><div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:timeline-event" width="28" height="28" /></span><div><h1>Timeline</h1><p>Chronological activity across connected sources.</p></div></div></div>
	<div class="timeline-layout">
			<TimelineStream messages={$communicationMessages as any} {isLayoutEditing} {isWidgetVisible} />
		<aside class="stacked-rail">
			<TimelineFilters {isLayoutEditing} {isWidgetVisible} />
		</aside>
	</div>

</section>
