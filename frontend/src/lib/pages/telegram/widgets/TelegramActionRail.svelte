<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { TelegramChatGroupFilter } from '$lib/services/telegram';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		groupFilters: TelegramChatGroupFilter[];
		activeGroupFilter: string;
		isTelegramBusy: boolean;
		hasSelectedTelegramChat: boolean;
		isInspectorOpen: boolean;
		onSyncChats: () => void;
		onSyncHistory: () => void;
		onStartRuntime: () => void;
		onSelectGroupFilter: (filter: TelegramChatGroupFilter) => void;
		onToggleInspector: () => void;
	}

	let {
		groupFilters,
		activeGroupFilter,
		isTelegramBusy,
		hasSelectedTelegramChat,
		isInspectorOpen,
		onSyncChats,
		onSyncHistory,
		onStartRuntime,
		onSelectGroupFilter,
		onToggleInspector
	}: Props = $props();
</script>

<section class="telegram-action-rail" aria-label={_('Telegram actions')}>
	<div class="telegram-action-cluster">
		<button type="button" onclick={onSyncChats} disabled={isTelegramBusy || !hasSelectedTelegramChat}>
			<Icon icon="tabler:refresh" width="16" height="16" />{_('Sync Chats')}
		</button>
		<button type="button" onclick={onSyncHistory} disabled={isTelegramBusy || !hasSelectedTelegramChat}>
			<Icon icon="tabler:history" width="16" height="16" />{_('Sync History')}
		</button>
		<button type="button" onclick={onStartRuntime} disabled={isTelegramBusy || !hasSelectedTelegramChat}>
			<Icon icon="tabler:player-play" width="16" height="16" />{_('Start Runtime')}
		</button>
	</div>
	<div class="telegram-group-filter-strip" aria-label={_('Chat Groups')}>
		{#each groupFilters as group}
			{#if group.count > 0 || group.id === 'local:all'}
				<button
					type="button"
					class:active={activeGroupFilter === group.id}
					onclick={() => onSelectGroupFilter(group)}
					title={group.source === 'telegram' ? _('Telegram folder') : _('Local group')}
				>
					<Icon icon={group.icon} width="15" height="15" />
					<span>{_(group.label)}</span>
					<em>{group.count}</em>
					{#if group.source === 'telegram'}<small>TG</small>{/if}
				</button>
			{/if}
		{/each}
	</div>
	<button
		type="button"
		class="telegram-inspector-toggle"
		class:active={isInspectorOpen}
		onclick={onToggleInspector}
	>
		<Icon icon="tabler:layout-sidebar-right" width="16" height="16" />{_('Details')}
	</button>
</section>
