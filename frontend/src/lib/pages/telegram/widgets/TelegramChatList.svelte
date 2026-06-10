<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		telegramChats: unknown[];
		selectedTelegramChatId: string;
		isTelegramLoading: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onSelectChat: (chat: unknown) => void;
		formatDateTime: (date: string | null) => string;
	}

	let {
		telegramChats,
		selectedTelegramChatId,
		isTelegramLoading,
		isLayoutEditing,
		isWidgetVisible,
		onSelectChat,
		formatDateTime
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="telegram-chat-list" data-widget-hidden={!isWidgetVisible('telegram-chat-list')}>
	<WidgetEditChrome widgetId="telegram-chat-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel conversation-list">
		<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder={_('Search Telegram chats...')} /></label>
		{#if isTelegramLoading && telegramChats.length === 0}
			<div class="empty-panel">{_('Loading Telegram state...')}</div>
		{:else if telegramChats.length === 0}
			<div class="empty-panel">{_('No Telegram chats projected yet.')}</div>
		{:else}
			{#each telegramChats as chat}
				{@const c = chat as Record<string, unknown>}
				<button type="button" class:active={selectedTelegramChatId === (c.provider_chat_id as string)} onclick={() => onSelectChat(chat)}>
					<span class="round-icon cyan"><Icon icon="tabler:brand-telegram" width="22" height="22" /></span>
					<img src="/assets/hermes-reference-avatar.png" alt="" />
					<span>
						<strong>{c.title as string}</strong>
						<small>{c.account_id as string} · {c.chat_kind as string}</small>
						<em>{c.sync_state as string}</em>
					</span>
					<time>{formatDateTime((c.last_message_at ?? c.updated_at) as string | null)}</time>
				</button>
			{/each}
		{/if}
	</section>
</div>
