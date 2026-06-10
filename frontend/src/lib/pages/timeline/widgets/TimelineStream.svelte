<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		messages: Array<{ sender_display_name?: string; sender?: string; subject?: string; body_text_preview?: string; occurred_at?: string; projected_at?: string }>;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { messages, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="timeline-stream" data-widget-hidden={!isWidgetVisible('timeline-stream')}>
	<WidgetEditChrome widgetId="timeline-stream" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel feed-panel large-timeline">
		<header class="panel-title-row"><h2>{_('Today')}</h2><button type="button" class="ghost-button" disabled>{_('All Events')}</button></header>
		{#each messages.slice(0, 20) as msg, index}
			<article class="timeline-event-row">
				<span class="rail-dot"></span>
				<span class="round-icon blue"><Icon icon="tabler:message" width="20" height="20" /></span>
				<div>
					<strong>{msg.sender_display_name || msg.sender || _('Unknown')}</strong>
					<p>{msg.subject || msg.body_text_preview}</p>
					<time>{msg.occurred_at || msg.projected_at}</time>
				</div>
			</article>
		{/each}
	</section>
</div>
