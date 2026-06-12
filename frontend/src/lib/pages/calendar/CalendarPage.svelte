<script lang="ts">
	import Icon from '@iconify/svelte';
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as calendarService from '$lib/services/calendar';
	import type { CalendarAccount, CalendarEvent, CalendarSource } from '$lib/api';
	import CalendarToolbar from './widgets/CalendarToolbar.svelte';
	import CalendarWeekGrid from './widgets/CalendarWeekGrid.svelte';
	import CalendarUpcoming from './widgets/CalendarUpcoming.svelte';
	import CalendarSourceStatus from './widgets/CalendarSourceStatus.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let calendarAccounts = $state<CalendarAccount[]>([]);
	let calendarEvents = $state<CalendarEvent[]>([]);
	let calendarSources = $state<CalendarSource[]>([]);
	let calendarError = $state('');
	let isCalendarLoading = $state(false);
	let calendarViewMode = $state<'day' | 'week' | 'month' | 'agenda'>('week');
	let calendarSearchQuery = $state('');
	let calendarSearchResults = $state<CalendarEvent[]>([]);
	let selectedEvent = $state<CalendarEvent | null>(null);
	let weeklyBrief = $state<Record<string, unknown> | null>(null);
	let eventBrief = $state<Record<string, unknown> | null>(null);
	let eventAgenda = $state<Record<string, unknown> | null>(null);
	let showNewEventForm = $state(false);
	let newEventTitle = $state('');
	let newEventStart = $state('');
	let newEventEnd = $state('');
	let newEventType = $state('meeting');

	const weekDays = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'];
	const nowDate = new Date();
	const weekStart = new Date(nowDate);
	weekStart.setDate(nowDate.getDate() - nowDate.getDay() + 1);
	weekStart.setHours(0, 0, 0, 0);
	const weekColumns = weekDays.map((d, i) => {
		const d2 = new Date(weekStart); d2.setDate(weekStart.getDate() + i);
		return `${d} ${d2.getDate()}`;
	});
	const eventTypeOptions = $derived(
		['meeting', 'focus', 'deadline', 'personal', 'travel', 'tax', 'review', 'planning'].map(
			(type) => ({ value: type, label: _(type.charAt(0).toUpperCase() + type.slice(1)) })
		)
	);

	let filteredEvents = $derived(calendarEvents.filter(e => {
		const start = new Date(e.start_at);
		const end = new Date(weekStart); end.setDate(weekStart.getDate() + 7);
		return start >= weekStart && start < end;
	}));

	async function loadCalendar() {
		isCalendarLoading = true;
		const result = await calendarService.loadCalendar();
		calendarAccounts = result.accounts;
		calendarEvents = result.events;
		calendarSources = result.sources;
		calendarError = result.error;
		isCalendarLoading = false;
	}

	async function loadWeeklyBrief() {
		const result = await calendarService.loadWeeklyBrief();
		weeklyBrief = result.brief;
	}

	async function searchCalendar() {
		const result = await calendarService.searchCalendar(calendarSearchQuery);
		calendarSearchResults = result.results;
	}

	async function prepareEvent(evt: CalendarEvent) {
		selectedEvent = evt;
		const result = await calendarService.prepareEvent(evt);
		eventBrief = result.brief;
		eventAgenda = result.agenda;
	}

	async function completeEvent(evt: CalendarEvent) {
		selectedEvent = evt;
		await calendarService.completeEvent(evt);
	}

	async function handleCreateEvent() {
		const result = await calendarService.handleCreateEvent(newEventTitle, newEventStart, newEventEnd, newEventType);
		if (result.error) { calendarError = result.error; return; }
		showNewEventForm = false;
		newEventTitle = '';
		await loadCalendar();
	}

	$effect(() => {
		loadCalendar();
	});
</script>

<section class="calendar-page">
		<CalendarToolbar
			activeViewTitle="Calendar"
			activeViewSubtitle="All your events from connected calendars"
			{calendarViewMode}
		{calendarSearchQuery}
		{isLayoutEditing}
		{isWidgetVisible}
		onCalendarViewMode={(mode) => { calendarViewMode = mode as 'day' | 'week' | 'month' | 'agenda'; loadCalendar(); }}
		onSearchCalendar={searchCalendar}
		onToggleNewEventForm={() => (showNewEventForm = !showNewEventForm)}
		onOpenAccountDrawer={() => {}}
		onLoadCalendar={loadCalendar}
		onLoadWeeklyBrief={loadWeeklyBrief}
	/>

	{#if showNewEventForm}
		<div class="panel new-event-form">
			<h3>{_('New Event')}</h3>
			<div class="form-row">
				<input type="text" placeholder={_('Event title')} bind:value={newEventTitle} />
				<HermesSelect
					value={newEventType}
					options={eventTypeOptions}
					placeholder={_('Event type')}
					searchPlaceholder={_('Search event types...')}
					emptyLabel={_('No options')}
					ariaLabel={_('Event type')}
					searchable={false}
					onChange={(nextValue) => (newEventType = nextValue)}
				/>
			</div>
			<div class="form-row"><input type="datetime-local" bind:value={newEventStart} /><span>→</span><input type="datetime-local" bind:value={newEventEnd} /></div>
			<div class="form-actions"><button type="button" class="primary-button" onclick={handleCreateEvent}>{_('Create')}</button><button type="button" class="ghost-button" onclick={() => (showNewEventForm = false)}>{_('Cancel')}</button></div>
		</div>
	{/if}

	<div class="filter-bar">
		<span>{calendarAccounts.length} {_('accounts')} &middot; {calendarEvents.length} {_('events')}</span>
		{#if calendarError}<span class="error-text">{calendarError}</span>{/if}
		{#if calendarSearchResults.length > 0}<span class="search-hint">{_('Search')}: {calendarSearchResults.length} {_('results for')} "{calendarSearchQuery}"</span>{/if}
	</div>

	<div class="calendar-layout">
		<CalendarWeekGrid
			{weekColumns}
			{calendarSearchResults}
			{filteredEvents}
			{isCalendarLoading}
			{calendarAccounts}
			{isLayoutEditing}
			{isWidgetVisible}
			onPrepareEvent={prepareEvent}
		/>
		<aside class="stacked-rail">
			<div class="panel info-card">
				<h2>{_('Weekly Brief')} <button type="button" class="link-row" onclick={loadWeeklyBrief}><Icon icon="tabler:refresh" width="12" height="12" /></button></h2>
				{#if weeklyBrief}
					<div class="metric-grid tiny">
						<article class="metric-card"><span>{_('Events')}</span><strong>{weeklyBrief.upcoming_events_this_week as number || 0}</strong></article>
						<article class="metric-card"><span>{_('Overdue')}</span><strong>{weeklyBrief.overdue_deadlines as number || 0}</strong></article>
						<article class="metric-card"><span>{_('No Notes')}</span><strong>{weeklyBrief.past_events_without_notes as number || 0}</strong></article>
					</div>
				{:else}<p class="muted">{_('Click refresh to load')}</p>{/if}
			</div>
			<CalendarUpcoming {calendarEvents} {isLayoutEditing} {isWidgetVisible} onPrepareEvent={prepareEvent} />
			{#if selectedEvent}
				<div class="panel info-card event-detail">
					<h2>{selectedEvent.title} <button type="button" class="ghost-button small" onclick={() => { selectedEvent = null; eventBrief = null; eventAgenda = null; }}><Icon icon="tabler:x" width="14" height="14" /></button></h2>
					<div class="event-meta">
						<span><Icon icon="tabler:clock" width="14" height="14" /> {new Date(selectedEvent.start_at).toLocaleString()}</span>
						{#if selectedEvent.location}<span><Icon icon="tabler:map-pin" width="14" height="14" /> {selectedEvent.location}</span>{/if}
						<span class="chip {selectedEvent.status}">{selectedEvent.status}</span>
					</div>
					{#if eventBrief}
						<div class="brief-section"><h4>{_('Brief')}</h4>
							{#if (eventBrief.participants as any[])}<div class="brief-participants">{#each (eventBrief.participants as any[]) as p}<span class="participant-chip">{p.name || p.email}</span>{/each}</div>{/if}
							{#if (eventBrief.context as any)?.summary}<p class="muted">{(eventBrief.context as any).summary}</p>{/if}
						</div>
					{/if}
					{#if eventAgenda}
						<div class="brief-section"><h4>{_('Agenda')}</h4>
							{#if eventAgenda.suggested_agenda}<ul class="agenda-list">{#each (eventAgenda.suggested_agenda as any[]) as item}<li>{item}</li>{/each}</ul>{/if}
						</div>
					{/if}
					<div class="event-actions">
					<button type="button" class="primary-button small" onclick={() => selectedEvent && prepareEvent(selectedEvent)}><Icon icon="tabler:brain" width="14" height="14" /> {_('Prepare')}</button>
					<button type="button" class="ghost-button small" onclick={() => selectedEvent && completeEvent(selectedEvent)}><Icon icon="tabler:check" width="14" height="14" /> {_('Complete')}</button>
					</div>
				</div>
			{/if}
			<CalendarSourceStatus {calendarSources} {calendarAccounts} {isLayoutEditing} {isWidgetVisible} />
		</aside>
	</div>
</section>
