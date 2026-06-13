<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { TelegramChat } from '$lib/api';
	import type { TelegramChatFilter, TelegramChatFilterCount } from '$lib/services/telegram';

	const _ = (key: string) => t($currentLocale, key);

	interface TelegramFilterTab {
		id: TelegramChatFilter;
		label: string;
	}

	interface Props {
		runtimeLabel: string;
		searchQuery: string;
		filterTabs: TelegramFilterTab[];
		filterCounts: TelegramChatFilterCount[];
		activeFilter: TelegramChatFilter;
		isFiltersMenuOpen: boolean;
		isNewMenuOpen: boolean;
		isTelegramBusy: boolean;
		selectedTelegramChat: TelegramChat | null;
		onToggleFiltersMenu: () => void;
		onToggleNewMenu: () => void;
		onSelectFilter: (filter: TelegramChatFilter) => void;
		onSyncChats: () => void;
		onAddAccount: () => void;
		onNewMessage: () => void;
		onQuickAction: (action: 'create_note' | 'create_task' | 'create_contact' | 'create_document') => void;
	}

	let {
		runtimeLabel,
		searchQuery = $bindable(),
		filterTabs,
		filterCounts,
		activeFilter,
		isFiltersMenuOpen,
		isNewMenuOpen,
		isTelegramBusy,
		selectedTelegramChat,
		onToggleFiltersMenu,
		onToggleNewMenu,
		onSelectFilter,
		onSyncChats,
		onAddAccount,
		onNewMessage,
		onQuickAction
	}: Props = $props();

	function filterCount(filter: TelegramChatFilter) {
		return filterCounts.find((item) => item.filter === filter)?.count ?? 0;
	}
</script>

<header class="communications-command-header telegram-command-header">
	<div class="command-title telegram-command-title">
		<h1>{_('Communications')} <span>/</span> {_('Telegram')}</h1>
		<p>{runtimeLabel}</p>
	</div>
	<label class="command-search">
		<Icon icon="tabler:search" width="18" height="18" />
		<input
			bind:value={searchQuery}
			placeholder={_('Search conversations...')}
			autocomplete="off"
		/>
	</label>
	<div class="command-menu">
		<button type="button" class:active={isFiltersMenuOpen || activeFilter !== 'all'} onclick={onToggleFiltersMenu}>
			<Icon icon="tabler:filter" width="17" height="17" />{_('Filters')}
		</button>
		{#if isFiltersMenuOpen}
			<div class="command-popover filter-command-popover">
				{#each filterTabs as tab}
					<button type="button" class:active={activeFilter === tab.id} onclick={() => onSelectFilter(tab.id)}>
						<span>{_(tab.label)}</span><em>{filterCount(tab.id)}</em>
					</button>
				{/each}
				<button type="button" onclick={onSyncChats} disabled={isTelegramBusy || !selectedTelegramChat}>
					<span><Icon icon="tabler:refresh" width="15" height="15" />{_('Sync Chats')}</span>
				</button>
			</div>
		{/if}
	</div>
	<div class="command-menu telegram-add-account">
		<button type="button" onclick={onAddAccount}>
			<Icon icon="tabler:user-plus" width="17" height="17" />{_('Add Account')}
		</button>
	</div>
	<div class="command-menu new-command">
		<button type="button" class="primary-button" onclick={onToggleNewMenu}>{_('New')}<Icon icon="tabler:plus" width="17" height="17" /></button>
		{#if isNewMenuOpen}
			<div class="command-popover new-command-popover">
				<button type="button" onclick={onNewMessage}><Icon icon="tabler:send" width="16" height="16" />{_('New Message')}</button>
				<button type="button" onclick={() => onQuickAction('create_note')}><Icon icon="tabler:notes" width="16" height="16" />{_('New Note')}</button>
				<button type="button" onclick={() => onQuickAction('create_task')}><Icon icon="tabler:square-check" width="16" height="16" />{_('New Task')}</button>
				<button type="button" onclick={() => onQuickAction('create_contact')}><Icon icon="tabler:user-plus" width="16" height="16" />{_('New Contact')}</button>
				<button type="button" onclick={() => onQuickAction('create_document')}><Icon icon="tabler:file-plus" width="16" height="16" />{_('New Document')}</button>
			</div>
		{/if}
	</div>
</header>
