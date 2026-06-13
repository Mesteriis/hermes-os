<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import './addWidgetDrawer.css';

	const _ = (key: string) => t($currentLocale, key);

	interface AddableWidget {
		id: string;
		title: string;
		defaultZone: string;
	}

	interface Props {
		isOpen: boolean;
		widgets: AddableWidget[];
		onClose: () => void;
		onShowWidget: (widgetId: string) => void;
	}

	let { isOpen, widgets, onClose, onShowWidget }: Props = $props();
</script>

{#if isOpen}
	<div class="widget-drawer" role="dialog" aria-label={_('Add widget')}>
		<header>
			<h2>{_('Add widget')}</h2>
			<button
				type="button"
				class="icon-button"
				onclick={onClose}
				title={_('Close add widget drawer')}
				aria-label={_('Close add widget drawer')}
			>
				<Icon icon="tabler:x" width="16" height="16" />
			</button>
		</header>
		<div class="widget-drawer-list">
			{#each widgets as widget}
				<button type="button" onclick={() => onShowWidget(widget.id)}>
					<strong>{widget.title}</strong>
					<span>{widget.defaultZone}</span>
				</button>
			{:else}
				<p>{_('No widgets available.')}</p>
			{/each}
		</div>
	</div>
{/if}
