<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="timeline-filters" data-widget-hidden={!isWidgetVisible('timeline-filters')}>
	<WidgetEditChrome widgetId="timeline-filters" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card"><h2>{_('Timeline Filters')}</h2>{#each ['Messages','Documents','Tasks','Calendar','Notes','Decisions'] as item}<label class="mini-check"><input type="checkbox" checked />{_(item)}</label>{/each}</section>
</div>
