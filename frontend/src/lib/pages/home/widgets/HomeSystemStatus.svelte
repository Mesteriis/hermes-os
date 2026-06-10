<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		statusError: string;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { statusError, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-system-status" data-widget-hide-if-clipped-content data-widget-hidden={!isWidgetVisible('home-system-status')}>
	<WidgetEditChrome widgetId="home-system-status" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel mini-panel" data-widget-fit-content>
		<header class="panel-title-row"><h2>{_('System Status')}</h2></header>
		<ul class="status-list">
			<li>All systems operational</li>
			<li>AI Agents online <span>5/5</span></li>
			<li>Data synchronized <span>2m ago</span></li>
			<li>Local AI models <span>Ready</span></li>
		</ul>
		{#if statusError}<p class="inline-error">{statusError}</p>{/if}
	</section>
</div>
