<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		activeViewTitle: string;
		activeViewSubtitle: string;
		calendarViewMode: string;
		calendarSearchQuery: string;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onCalendarViewMode: (mode: string) => void;
		onSearchCalendar: () => void;
		onToggleNewEventForm: () => void;
		onOpenAccountDrawer: () => void;
		onLoadCalendar: () => void;
		onLoadWeeklyBrief: () => void;
	}

	let {
		activeViewTitle, activeViewSubtitle, calendarViewMode, calendarSearchQuery,
		isLayoutEditing, isWidgetVisible,
		onCalendarViewMode, onSearchCalendar, onToggleNewEventForm,
		onOpenAccountDrawer, onLoadCalendar, onLoadWeeklyBrief
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="calendar-toolbar" data-widget-hidden={!isWidgetVisible('calendar-toolbar')}>
	<WidgetEditChrome widgetId="calendar-toolbar" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="view-header">
		<div class="view-title-with-icon">
			<span class="hero-mark small"><Icon icon="tabler:calendar" width="28" height="28" /></span>
			<div><h1>{_(activeViewTitle)}</h1><p>{_(activeViewSubtitle)}</p></div>
		</div>
		<div class="search-bar">
			<input type="text" placeholder={_('Search events...')} bind:value={calendarSearchQuery} oninput={() => onSearchCalendar()} />
		</div>
		<div class="section-tabs pill-tabs">
			<button type="button" class:active={calendarViewMode === 'day'} onclick={() => onCalendarViewMode('day')}>{_('Day')}</button>
			<button type="button" class:active={calendarViewMode === 'week'} onclick={() => onCalendarViewMode('week')}>{_('Week')}</button>
			<button type="button" class:active={calendarViewMode === 'month'} onclick={() => onCalendarViewMode('month')}>{_('Month')}</button>
			<button type="button" class:active={calendarViewMode === 'agenda'} onclick={() => onCalendarViewMode('agenda')}>{_('Agenda')}</button>
		</div>
		<button type="button" class="primary-button" onclick={onToggleNewEventForm}><Icon icon="tabler:plus" width="16" height="16" /> {_('New Event')}</button>
		<button type="button" class="ghost-button" onclick={onOpenAccountDrawer}><Icon icon="tabler:calendar-plus" width="16" height="16" />{_('Add Calendar')}</button>
		<button type="button" class="ghost-button" onclick={() => { onLoadCalendar(); onLoadWeeklyBrief(); }} title={_('Refresh')}><Icon icon="tabler:refresh" width="16" height="16" /></button>
	</div>
</div>
