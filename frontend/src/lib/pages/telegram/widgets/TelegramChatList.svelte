<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { TelegramChat, TelegramMessage } from '$lib/api';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import {
		telegramChatIsPinned,
		telegramChatPreview,
		telegramChatUnreadCount,
		telegramMessageAttachmentHints
	} from '$lib/services/telegram';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		telegramChats: TelegramChat[];
		telegramMessages: TelegramMessage[];
		selectedTelegramChatId: string;
		isTelegramLoading: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onSelectChat: (chat: TelegramChat) => void;
		formatDateTime: (date: string | null) => string;
	}

	let {
		telegramChats,
		telegramMessages,
		selectedTelegramChatId,
		isTelegramLoading,
		isLayoutEditing,
		isWidgetVisible,
		onSelectChat,
		formatDateTime
	}: Props = $props();

	const visibleRange = $derived(telegramChats.length ? `1-${telegramChats.length}` : '0');

	function chatMessages(chat: TelegramChat): TelegramMessage[] {
		return telegramMessages.filter((message) => message.provider_chat_id === chat.provider_chat_id);
	}

	function chatTime(chat: TelegramChat): string {
		return formatDateTime(chat.last_message_at ?? chat.updated_at);
	}

	function chatIcon(chat: TelegramChat): string {
		if (chat.chat_kind === 'bot' || chat.title.toLowerCase().includes('bot')) return 'tabler:robot';
		if (chat.chat_kind === 'channel') return 'tabler:speakerphone';
		if (chat.chat_kind === 'private') return 'tabler:user';
		return 'tabler:users-group';
	}

	function chatInitials(chat: TelegramChat): string {
		const parts = chat.title.split(/\s+/).filter(Boolean);
		if (!parts.length) return 'TG';
		return parts.slice(0, 2).map((part) => part[0]?.toUpperCase()).join('');
	}

	function isMuted(chat: TelegramChat): boolean {
		return Boolean(chat.metadata.muted ?? chat.metadata.is_muted);
	}

	function hasAttachment(chat: TelegramChat): boolean {
		return chatMessages(chat).some((message) => telegramMessageAttachmentHints(message).length > 0);
	}
</script>

<div
	class="widget-frame"
	class:editing={isLayoutEditing}
	data-widget-id="telegram-chat-list"
	data-widget-hidden={!isWidgetVisible('telegram-chat-list')}
>
	<WidgetEditChrome widgetId="telegram-chat-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel conversation-list telegram-conversation-list">
		<header class="telegram-panel-header">
			<h2>{_('Conversations')}</h2>
			<button type="button" title={_('Filters')} disabled={isTelegramLoading}>
				<Icon icon="tabler:adjustments-horizontal" width="17" height="17" />
			</button>
		</header>

		<div class="telegram-chat-scroll">
			{#if isTelegramLoading && telegramChats.length === 0}
				<div class="empty-panel">{_('Loading Telegram state...')}</div>
			{:else if telegramChats.length === 0}
				<div class="empty-panel">{_('No Telegram chats projected yet.')}</div>
			{:else}
				{#each telegramChats as chat}
					{@const unreadCount = telegramChatUnreadCount(chat)}
					{@const preview = telegramChatPreview(chat, telegramMessages)}
					<button
						type="button"
						class="telegram-chat-row"
						class:active={selectedTelegramChatId === chat.provider_chat_id}
						onclick={() => onSelectChat(chat)}
					>
						<span class="telegram-avatar" data-kind={chat.chat_kind}>
							{#if chat.chat_kind === 'group' || chat.chat_kind === 'channel'}
								<Icon icon={chatIcon(chat)} width="19" height="19" />
							{:else}
								{chatInitials(chat)}
							{/if}
						</span>
						<span class="telegram-chat-copy">
							<span class="telegram-chat-title-line">
								<strong>{chat.title}</strong>
								{#if telegramChatIsPinned(chat)}
									<Icon icon="tabler:pin" width="13" height="13" />
								{/if}
								{#if isMuted(chat)}
									<Icon icon="tabler:bell-off" width="13" height="13" />
								{/if}
							</span>
							<small>{preview}</small>
							<span class="telegram-chat-state">
								<em>{chat.sync_state}</em>
								{#if hasAttachment(chat)}
									<Icon icon="tabler:paperclip" width="13" height="13" />
								{/if}
							</span>
						</span>
						<span class="telegram-chat-side">
							<time>{chatTime(chat)}</time>
							{#if unreadCount > 0}
								<b>{unreadCount}</b>
							{:else if chat.metadata.delivery_state === 'sent'}
								<Icon icon="tabler:checks" width="15" height="15" />
							{/if}
						</span>
					</button>
				{/each}
			{/if}
		</div>

		<footer class="telegram-list-footer">
			<span class="telegram-list-range">{visibleRange} {_('of')} {telegramChats.length}</span>
			<div>
				<button type="button" disabled title={_('Previous')}><Icon icon="tabler:chevron-left" width="17" height="17" /></button>
				<button type="button" disabled title={_('Next')}><Icon icon="tabler:chevron-right" width="17" height="17" /></button>
			</div>
		</footer>
	</section>
</div>
