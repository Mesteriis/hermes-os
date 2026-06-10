<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		items: Array<{ icon: string; title: string; meta: string; time: string; tag?: string; tone?: string }>;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { items, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-whats-new" data-widget-hide-if-clipped-content data-widget-hidden={!isWidgetVisible('home-whats-new')}>
	<WidgetEditChrome widgetId="home-whats-new" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel feed-panel" data-widget-fit-content>
		<header class="panel-title-row">
			<div><h2>{_("What's New")}</h2><p>{_('Key changes and important updates')}</p></div>
			<button type="button" class="ghost-button" disabled>All Types</button>
		</header>
		<div class="feed-list">
			{#each items as item}
				<article class="feed-row">
					<span class="round-icon {item.tone}"><Icon icon={item.icon} width="22" height="22" /></span>
					<div>
						<strong>{item.title}</strong>
						<p>{item.meta}</p>
						{#if item.tag}<em>{item.tag}</em>{/if}
					</div>
					<time>{item.time}</time>
				</article>
			{/each}
		</div>
		<button type="button" class="link-row" disabled>View all events <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
	</section>
</div>
