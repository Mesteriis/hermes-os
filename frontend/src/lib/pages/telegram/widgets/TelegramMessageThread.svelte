<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { MessageAnalyzeResponse, TelegramChat, TelegramMessage, TelegramRuntimeStatus } from '$lib/api';
	import { formatBytes } from '$lib/services/formatting';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import {
		type TelegramAttachmentHint,
		type TelegramThreadTab,
		telegramAttachmentHintsForMessages,
		telegramLinkHintsForMessages,
		telegramMessageAttachmentHints,
		telegramMessagesChronological,
		telegramPinnedMessages
	} from '$lib/services/telegram';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedTelegramChat: TelegramChat | null;
		selectedTelegramMessages: TelegramMessage[];
		aiAnalysisResult: MessageAnalyzeResponse | null;
		selectedCommunication: { message_id?: string } | null;
		isTelegramLoading: boolean;
		isTelegramActionSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		activeThreadTab: TelegramThreadTab;
		onActiveThreadTabChange: (tab: TelegramThreadTab) => void;
		onRailTabChange: (tab: 'context' | 'members' | 'about') => void;
		telegramMessageTime: (message: TelegramMessage) => string;
		loadTelegramWorkspace: () => Promise<void>;
		syncSelectedTelegramHistory: () => Promise<void>;
		syncOlderTelegramHistory: () => Promise<void>;
		sendTelegramManualMessage: () => Promise<void>;
		downloadTelegramMedia: (attachment: TelegramAttachmentHint, message?: TelegramMessage) => Promise<void>;
		telegramManualSendForm: { text: string };
		selectedTelegramRuntimeStatus: TelegramRuntimeStatus | null;
	}

	let {
		selectedTelegramChat,
		selectedTelegramMessages,
		aiAnalysisResult,
		selectedCommunication,
		isTelegramLoading,
		isTelegramActionSubmitting,
		isLayoutEditing,
		isWidgetVisible,
		activeThreadTab,
		onActiveThreadTabChange,
		onRailTabChange,
		telegramMessageTime,
		loadTelegramWorkspace,
		syncSelectedTelegramHistory,
		syncOlderTelegramHistory,
		sendTelegramManualMessage,
		downloadTelegramMedia,
		telegramManualSendForm,
		selectedTelegramRuntimeStatus
	}: Props = $props();

	let threadSearchQuery = $state('');
	let isSearchOpen = $state(false);
	let isEmojiTrayOpen = $state(false);
	let isSendMenuOpen = $state(false);

	const chronologicalMessages = $derived(telegramMessagesChronological(selectedTelegramMessages));
	const fileHints = $derived(telegramAttachmentHintsForMessages(chronologicalMessages));
	const linkHints = $derived(telegramLinkHintsForMessages(chronologicalMessages));
	const pinnedMessages = $derived(telegramPinnedMessages(chronologicalMessages));
	const filteredMessages = $derived(
		chronologicalMessages.filter((message) => {
			const query = threadSearchQuery.trim().toLowerCase();
			if (!query) return true;
			return [
				message.text,
				message.sender,
				message.sender_display_name ?? '',
				message.provider_message_id
			]
				.join(' ')
				.toLowerCase()
				.includes(query);
		})
	);
	const tabs = $derived([
		{ id: 'messages' as const, label: _('Messages'), count: chronologicalMessages.length },
		{ id: 'files' as const, label: _('Files'), count: fileHints.length },
		{ id: 'links' as const, label: _('Links'), count: linkHints.length },
		{ id: 'topics' as const, label: _('Topics'), count: topicCount(selectedTelegramChat) },
		{ id: 'pinned' as const, label: _('Pinned'), count: pinnedMessages.length },
		{ id: 'timeline' as const, label: _('Timeline'), count: chronologicalMessages.length }
	]);

	function appendEmoji(value: string) {
		telegramManualSendForm.text = `${telegramManualSendForm.text}${value}`;
		isEmojiTrayOpen = false;
	}

	function submitManualSend() {
		isSendMenuOpen = false;
		void sendTelegramManualMessage();
	}

	function senderName(message: TelegramMessage): string {
		return message.sender_display_name ?? message.sender;
	}

	function senderInitials(message: TelegramMessage): string {
		return senderName(message)
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase())
			.join('') || 'TG';
	}

	function isOutbound(message: TelegramMessage): boolean {
		return message.delivery_state === 'sent' || message.delivery_state === 'send_dry_run';
	}

	function topicCount(chat: TelegramChat | null): number {
		const value = chat?.metadata.topics_count ?? chat?.metadata.topic_count;
		return typeof value === 'number' ? value : 0;
	}

	function memberSummary(chat: TelegramChat): string {
		const memberCount = chat.metadata.member_count ?? chat.metadata.members_count;
		const onlineCount = chat.metadata.online_count ?? chat.metadata.online_members_count;
		if (typeof memberCount === 'number' && typeof onlineCount === 'number') {
			return `${memberCount.toLocaleString('en-US')} ${_('members')}, ${onlineCount.toLocaleString('en-US')} ${_('online')}`;
		}
		if (typeof memberCount === 'number') return `${memberCount.toLocaleString('en-US')} ${_('members')}`;
		return `${chat.account_id} · ${chat.provider_chat_id}`;
	}

	function formatDate(value: string): string {
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return '';
		return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric' }).format(date);
	}

	function handleThreadScroll(event: Event) {
		if (activeThreadTab !== 'messages' || isTelegramActionSubmitting) return;
		const target = event.currentTarget as HTMLElement | null;
		if (!target || target.scrollTop > 48) return;
		void syncOlderTelegramHistory();
	}
</script>

<div
	class="widget-frame"
	class:editing={isLayoutEditing}
	data-widget-id="telegram-message-thread"
	data-widget-hidden={!isWidgetVisible('telegram-message-thread')}
>
	<WidgetEditChrome widgetId="telegram-message-thread" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel chat-pane telegram-chat-pane">
		{#if selectedTelegramChat}
			<header class="telegram-thread-header">
				<div class="telegram-thread-title">
					<span class="telegram-avatar large" data-kind={selectedTelegramChat.chat_kind}>
						<Icon icon="tabler:brand-telegram" width="24" height="24" />
					</span>
					<div>
						<h2>{selectedTelegramChat.title}</h2>
						<p>{memberSummary(selectedTelegramChat)}</p>
					</div>
				</div>
				{#if selectedTelegramRuntimeStatus}
					<span class="state-badge {selectedTelegramRuntimeStatus.status}">
						{selectedTelegramRuntimeStatus.status}
					</span>
				{/if}
				<div class="telegram-thread-actions">
					<button type="button" class:active={isSearchOpen} onclick={() => (isSearchOpen = !isSearchOpen)} title={_('Search')}>
						<Icon icon="tabler:search" width="18" height="18" />
					</button>
					<button type="button" onclick={() => onActiveThreadTabChange('pinned')} title={_('Pinned')}>
						<Icon icon="tabler:pin" width="18" height="18" />
					</button>
					<button type="button" onclick={() => onRailTabChange('members')} title={_('Members')}>
						<Icon icon="tabler:users" width="18" height="18" />
					</button>
					<button type="button" onclick={() => void syncSelectedTelegramHistory()} disabled={isTelegramActionSubmitting} title={_('Sync History')}>
						<Icon icon="tabler:history" width="18" height="18" />
					</button>
					<button type="button" onclick={() => void loadTelegramWorkspace()} disabled={isTelegramLoading} title={_('Refresh')}>
						<Icon icon="tabler:refresh" width="18" height="18" />
					</button>
				</div>
			</header>

			{#if isSearchOpen}
				<label class="telegram-thread-search">
					<Icon icon="tabler:search" width="16" height="16" />
					<input bind:value={threadSearchQuery} placeholder={_('Search in this chat...')} autocomplete="off" />
				</label>
			{/if}

			<nav class="message-context-tabs telegram-thread-tabs">
				{#each tabs as tab}
					<button
						type="button"
						class:active={activeThreadTab === tab.id}
						onclick={() => onActiveThreadTabChange(tab.id)}
					>
						{tab.label}
						{#if tab.count > 0}
							<em>{tab.count}</em>
						{/if}
					</button>
				{/each}
			</nav>

			<div class="chat-body telegram-thread-body" onscroll={handleThreadScroll}>
				{#if aiAnalysisResult && aiAnalysisResult.message_id === selectedCommunication?.message_id}
					<article class="ai-analysis-card telegram-ai-card">
						<strong><Icon icon="tabler:sparkles" width="16" height="16" />{_('AI Analysis')}</strong>
						{#if aiAnalysisResult.category}<p><em>{_('Category:')}</em> {aiAnalysisResult.category}</p>{/if}
						{#if aiAnalysisResult.summary}<p><em>{_('Summary:')}</em> {aiAnalysisResult.summary}</p>{/if}
						{#if aiAnalysisResult.importance_score != null}<p><em>{_('Importance:')}</em> {aiAnalysisResult.importance_score}/100</p>{/if}
					</article>
				{/if}

				{#if activeThreadTab === 'messages'}
					{#if selectedTelegramChat.chat_kind !== 'private'}
						<div class="telegram-history-actions">
							<button type="button" onclick={() => void syncOlderTelegramHistory()} disabled={isTelegramActionSubmitting}>
								<Icon icon="tabler:arrow-up" width="16" height="16" />
								{_('Load older')}
							</button>
						</div>
					{/if}
					{#if filteredMessages.length === 0}
						<div class="empty-panel fill">
							{threadSearchQuery
								? _('No Telegram messages match this search.')
								: isTelegramActionSubmitting
									? _('Syncing selected Telegram history...')
									: _('No messages for this chat.')}
						</div>
					{:else}
						<div class="telegram-date-chip">{_('Today')}</div>
						{#each filteredMessages as message}
							{@const attachments = telegramMessageAttachmentHints(message)}
							<article class="telegram-message-row" class:outbound={isOutbound(message)}>
								<span class="telegram-message-avatar">{senderInitials(message)}</span>
								<div class="bubble telegram-bubble" class:outbound={isOutbound(message)} class:inbound={!isOutbound(message)}>
									<strong>{senderName(message)}</strong>
									<p>{message.text}</p>
									{#if attachments.length}
										<div class="telegram-bubble-files">
											{#each attachments as attachment}
												<div class="telegram-file-card compact">
													<span><Icon icon="tabler:file" width="18" height="18" /></span>
													<div>
														<strong>{attachment.fileName}</strong>
														<small>{attachment.sizeBytes == null ? attachment.downloadState : `${formatBytes(attachment.sizeBytes)} · ${attachment.downloadState}`}</small>
													</div>
													<button
														type="button"
														disabled={isTelegramActionSubmitting || attachment.tdlibFileId === null}
														title={attachment.tdlibFileId === null ? _('Download requires TDLib file metadata') : _('Download media')}
														onclick={() => void downloadTelegramMedia(attachment, message)}
													>
														<Icon icon="tabler:download" width="16" height="16" />
													</button>
												</div>
											{/each}
										</div>
									{/if}
									<time>
										{telegramMessageTime(message)}
										<span>{message.delivery_state}</span>
									</time>
								</div>
							</article>
						{/each}
					{/if}
				{:else if activeThreadTab === 'files'}
					{#if fileHints.length === 0}
						<div class="empty-panel fill">{_('No files in selected Telegram history.')}</div>
					{:else}
						<div class="telegram-file-list">
							{#each fileHints as file}
								<div class="telegram-file-card">
									<span><Icon icon={file.kind === 'photo' ? 'tabler:photo' : file.kind === 'video' ? 'tabler:video' : 'tabler:file-description'} width="20" height="20" /></span>
									<div>
										<strong>{file.fileName}</strong>
										<small>{file.mimeType ?? file.kind} · {file.sizeBytes == null ? file.downloadState : formatBytes(file.sizeBytes)}</small>
									</div>
									<button
										type="button"
										disabled={isTelegramActionSubmitting || file.tdlibFileId === null}
										title={file.tdlibFileId === null ? _('Download requires TDLib file metadata') : _('Download media')}
										onclick={() => void downloadTelegramMedia(file)}
									>
										<Icon icon="tabler:download" width="17" height="17" />
									</button>
								</div>
							{/each}
						</div>
					{/if}
				{:else if activeThreadTab === 'links'}
					{#if linkHints.length === 0}
						<div class="empty-panel fill">{_('No links in selected Telegram history.')}</div>
					{:else}
						<div class="telegram-link-list">
							{#each linkHints as link}
								<a href={link.url} target="_blank" rel="noreferrer">
									<Icon icon="tabler:link" width="17" height="17" />
									<span>{link.label}</span>
									<em>{link.occurredAt ? formatDate(link.occurredAt) : ''}</em>
								</a>
							{/each}
						</div>
					{/if}
				{:else if activeThreadTab === 'pinned'}
					{#if pinnedMessages.length === 0}
						<div class="empty-panel fill">{_('No pinned messages in selected Telegram history.')}</div>
					{:else}
						{#each pinnedMessages as message}
							<article class="telegram-timeline-row">
								<Icon icon="tabler:pin" width="16" height="16" />
								<div><strong>{senderName(message)}</strong><p>{message.text}</p></div>
								<time>{telegramMessageTime(message)}</time>
							</article>
						{/each}
					{/if}
				{:else if activeThreadTab === 'timeline'}
					{#if chronologicalMessages.length === 0}
						<div class="empty-panel fill">{_('No timeline events in selected Telegram history.')}</div>
					{:else}
						{#each chronologicalMessages as message}
							<article class="telegram-timeline-row">
								<Icon icon={isOutbound(message) ? 'tabler:send' : 'tabler:message'} width="16" height="16" />
								<div><strong>{senderName(message)}</strong><p>{message.text}</p></div>
								<time>{telegramMessageTime(message)}</time>
							</article>
						{/each}
					{/if}
				{:else}
					<div class="empty-panel fill">{_('Telegram topics are available after TDLib forum topic sync is implemented.')}</div>
				{/if}
			</div>

			<form class="telegram-compose-bar" onsubmit={(event) => { event.preventDefault(); submitManualSend(); }}>
				<button type="button" disabled title={_('Attachment upload is not available in this slice')}>
					<Icon icon="tabler:paperclip" width="18" height="18" />
				</button>
				<textarea
					value={telegramManualSendForm.text}
					oninput={(event) => (telegramManualSendForm.text = event.currentTarget.value)}
					rows="1"
					placeholder={_('Write a message...')}
					autocomplete="off"
				></textarea>
				<div class="telegram-compose-menu">
					<button type="button" onclick={() => (isEmojiTrayOpen = !isEmojiTrayOpen)} title={_('Emoji')}>
						<Icon icon="tabler:mood-smile" width="18" height="18" />
					</button>
					{#if isEmojiTrayOpen}
						<div class="telegram-emoji-popover">
							{#each ['👍', '🔥', '🎉', '✅', '🙏'] as emoji}
								<button type="button" onclick={() => appendEmoji(emoji)}>{emoji}</button>
							{/each}
						</div>
					{/if}
				</div>
				<button type="button" disabled title={_('Voice messages require media runtime')}>
					<Icon icon="tabler:microphone" width="18" height="18" />
				</button>
				<button
					type="submit"
					class="send"
					disabled={isTelegramActionSubmitting || !telegramManualSendForm.text.trim()}
					title={_('Send')}
				>
					<Icon icon="tabler:send" width="18" height="18" />
				</button>
				<div class="telegram-compose-menu">
					<button type="button" class="send-more" onclick={() => (isSendMenuOpen = !isSendMenuOpen)} title={_('More')}>
						<Icon icon="tabler:chevron-down" width="17" height="17" />
					</button>
					{#if isSendMenuOpen}
						<div class="command-popover telegram-send-popover">
							<button type="button" onclick={submitManualSend} disabled={isTelegramActionSubmitting || !telegramManualSendForm.text.trim()}>
								<Icon icon="tabler:send" width="15" height="15" />{_('Send now')}
							</button>
							<button type="button" onclick={() => { isSendMenuOpen = false; void syncSelectedTelegramHistory(); }} disabled={isTelegramActionSubmitting}>
								<Icon icon="tabler:history" width="15" height="15" />{_('Sync History')}
							</button>
						</div>
					{/if}
				</div>
			</form>
		{:else}
			<div class="empty-panel fill">{_('Select a Telegram chat to inspect messages and compose replies.')}</div>
		{/if}
	</section>
</div>
