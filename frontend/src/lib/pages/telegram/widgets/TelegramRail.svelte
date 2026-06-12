<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { TelegramChat } from '$lib/api';
	import type { TelegramRailTab } from '$lib/services/telegram';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedTelegramChat: TelegramChat | null;
		activeRailTab: TelegramRailTab;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onActiveRailTabChange: (tab: TelegramRailTab) => void;
		onClose: () => void;
	}

	let {
		selectedTelegramChat,
		activeRailTab,
		isLayoutEditing,
		isWidgetVisible,
		onActiveRailTabChange,
		onClose
	}: Props = $props();
</script>

<aside class="stacked-rail telegram-rail">
	<div
		class="widget-frame stacked-rail"
		class:editing={isLayoutEditing}
		data-widget-id="telegram-context-inspector"
		data-widget-hidden={!isWidgetVisible('telegram-context-inspector')}
	>
		<WidgetEditChrome widgetId="telegram-context-inspector" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel telegram-context-panel telegram-empty-inspector">
			<header class="telegram-inspector-head">
				<div>
					<h2>{_('Details')}</h2>
					<p>{selectedTelegramChat?.title ?? _('No chat selected')}</p>
				</div>
				<button type="button" onclick={onClose} title={_('Close')}>
					<Icon icon="tabler:x" width="17" height="17" />
				</button>
			</header>

			<nav class="inspector-tabs telegram-rail-tabs">
				<button type="button" class:active={activeRailTab === 'context'} onclick={() => onActiveRailTabChange('context')}>{_('Context')}</button>
				<button type="button" class:active={activeRailTab === 'members'} onclick={() => onActiveRailTabChange('members')}>{_('Members')}</button>
				<button type="button" class:active={activeRailTab === 'about'} onclick={() => onActiveRailTabChange('about')}>{_('About')}</button>
			</nav>

			<div class="telegram-inspector-placeholder" aria-label={_('Details panel placeholder')}></div>
		</section>
	</div>
</aside>
