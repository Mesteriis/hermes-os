<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-upcoming" data-widget-hide-if-clipped-content data-widget-hidden={!isWidgetVisible('home-upcoming')}>
	<WidgetEditChrome widgetId="home-upcoming" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel schedule-panel" data-widget-fit-content>
		<header class="panel-title-row"><div><h2>{_('Upcoming')}</h2><p>{_('Your schedule')}</p></div></header>
		<div class="schedule-list">
			<article><time>Today, May 12</time><strong>14:00 Call with Acme Corp</strong><span>16:30 Review Q2 Report</span></article>
			<article><time>Tomorrow, May 13</time><strong>10:00 Project Hermes - Planning</strong><span>15:00 Design Review</span></article>
		</div>
		<button type="button" class="link-row" disabled>View full calendar <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
	</section>
</div>
