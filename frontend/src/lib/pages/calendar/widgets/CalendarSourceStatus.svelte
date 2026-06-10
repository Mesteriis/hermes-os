<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { CalendarAccount, CalendarSource } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		calendarSources: CalendarSource[];
		calendarAccounts: CalendarAccount[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { calendarSources, calendarAccounts, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="calendar-source-status" data-widget-hidden={!isWidgetVisible('calendar-source-status')}>
	<WidgetEditChrome widgetId="calendar-source-status" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>{_('Calendars')}</h2>
		{#if calendarSources.length === 0}
			{#each calendarAccounts as acct}
				<label class="mini-check"><input type="checkbox" checked disabled />{acct.account_name}<em>{acct.provider}</em></label>
			{/each}
		{:else}
			{#each calendarSources as src}
				<label class="mini-check"><input type="checkbox" checked disabled />{src.name}<em>{src.timezone || ''}</em></label>
			{/each}
		{/if}
	</section>
</div>
