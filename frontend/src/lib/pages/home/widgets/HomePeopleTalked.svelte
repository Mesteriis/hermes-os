<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		people: Array<{ name: string; meta: string; icon: string }>;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { people, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-people-talked-to" data-widget-hide-if-clipped-content data-widget-hidden={!isWidgetVisible('home-people-talked-to')}>
	<WidgetEditChrome widgetId="home-people-talked-to" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel mini-panel" data-widget-fit-content>
		<header class="panel-title-row"><h2>{_('People You Talked To')}</h2><button type="button" class="link-button" disabled>View all</button></header>
		<div class="person-list">
			{#each people as person}
				<article>
					<img src="/assets/hermes-reference-avatar.png" alt="" />
					<span><strong>{person.name}</strong><small>{person.meta}</small></span>
					<Icon icon={person.icon} width="18" height="18" />
				</article>
			{/each}
		</div>
	</section>
</div>
