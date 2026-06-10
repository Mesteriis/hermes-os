<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { CalendarAccount, CalendarEvent } from '$lib/api';

	interface Props {
		weekColumns: string[];
		calendarSearchResults: CalendarEvent[];
		filteredEvents: CalendarEvent[];
		isCalendarLoading: boolean;
		calendarAccounts: CalendarAccount[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onPrepareEvent: (evt: CalendarEvent) => void;
	}

	let { weekColumns, calendarSearchResults, filteredEvents, isCalendarLoading, calendarAccounts, isLayoutEditing, isWidgetVisible, onPrepareEvent }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="calendar-week-grid" data-widget-hidden={!isWidgetVisible('calendar-week-grid')}>
	<WidgetEditChrome widgetId="calendar-week-grid" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel week-board">
		<div class="week-header">
			{#each weekColumns as day}
				<strong>{day}</strong>
			{/each}
		</div>
		<div class="event-list">
			{#if isCalendarLoading}
				<div class="loading-state">Loading events...</div>
			{:else if (calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents).length === 0}
				<div class="empty-state">No events</div>
			{:else}
				{#each (calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents) as evt (evt.event_id)}
					{@const tone = evt.event_type === 'meeting' ? 'blue' : evt.event_type === 'deadline' ? 'red' : evt.event_type === 'focus' ? 'green' : 'neutral'}
					{@const dayLabel = new Date(evt.start_at).toLocaleDateString('en-US', {weekday:'short', day:'numeric'})}
					<div class="event-row {tone}" onclick={() => onPrepareEvent(evt)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && onPrepareEvent(evt)}>
						<span class="event-day">{dayLabel}</span>
						<span class="event-time">{new Date(evt.start_at).toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'})} - {new Date(evt.end_at).toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'})}</span>
						<strong>{evt.title}</strong>
						<span class="event-type-chip">{evt.event_type || 'event'}</span>
						{#if evt.importance_score && evt.importance_score > 0.5}<em class="importance-dot high"></em>{/if}
						{#if evt.readiness_score != null && evt.readiness_score < 0.5}<em class="importance-dot warn"></em>{/if}
					</div>
				{/each}
			{/if}
		</div>
		<footer class="source-footer">
			{#each calendarAccounts as acct}
				<span class="source-badge">{acct.account_name}</span>
			{/each}
		</footer>
	</section>
</div>
