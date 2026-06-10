<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { CalendarEvent } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		calendarEvents: CalendarEvent[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onPrepareEvent: (evt: CalendarEvent) => void;
	}

	let { calendarEvents, isLayoutEditing, isWidgetVisible, onPrepareEvent }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="calendar-upcoming" data-widget-hidden={!isWidgetVisible('calendar-upcoming')}>
	<WidgetEditChrome widgetId="calendar-upcoming" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>{_('Upcoming')}</h2>
		{#if calendarEvents.length === 0}
			<p class="muted">No upcoming events</p>
		{:else}
			{#each calendarEvents.filter(e => new Date(e.start_at) >= new Date()).slice(0, 8) as evt}
				<div class="deadline" role="button" tabindex="0" onclick={() => onPrepareEvent(evt)} onkeydown={(e) => e.key === 'Enter' && onPrepareEvent(evt)}>
					<span>{new Date(evt.start_at).toLocaleDateString('en-US', {weekday:'short', month:'short', day:'numeric'})} &middot; {evt.title}</span>
					<time>{new Date(evt.start_at).toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'})}</time>
				</div>
			{/each}
		{/if}
	</section>
</div>
