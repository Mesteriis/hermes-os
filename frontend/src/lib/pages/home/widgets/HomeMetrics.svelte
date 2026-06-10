<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		stats: Array<{ label: string; value: string; delta: string; icon: string; tone?: string }>;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { stats, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-metrics" data-widget-hide-if-clipped-content data-widget-hidden={!isWidgetVisible('home-metrics')}>
	<WidgetEditChrome widgetId="home-metrics" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="metric-grid home-metrics" data-widget-fit-content>
		{#each stats as metric}
			<article class="metric-card">
				<span>{metric.label}</span>
				<div><strong>{metric.value}</strong><Icon icon={metric.icon} width="26" height="26" /></div>
				<small>↑ {metric.delta}</small>
			</article>
		{/each}
		<article class="metric-card focus-card">
			<span>{_('Focus Score')}</span>
			<div class="score-ring"><strong>78</strong></div>
			<small>Good ↑ 5</small>
		</article>
	</div>
</div>
