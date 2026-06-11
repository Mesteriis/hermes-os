<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { conversationPreview, senderEmail } from '$lib/services/communications';
	import type { CommunicationMessageSummary, CommunicationMessageSummaryV2 } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type ConversationMessage = CommunicationMessageSummary | CommunicationMessageSummaryV2;
	type NavigatorMode = 'threads' | 'contacts';
	type ContactGroup = {
		key: string;
		name: string;
		email: string;
		messages: ConversationMessage[];
		latestMessage: ConversationMessage;
	};

	interface Props {
		communicationMessages: ConversationMessage[];
		isCommunicationsLoading: boolean;
		communicationsError: string;
		selectedConversationIndex: number;
		selectedCommunication: ConversationMessage | null;
		navigatorMode: NavigatorMode;
		expandedContactKey: string | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		selectCommunication: (index: number) => void;
		onNavigatorModeChange: (mode: NavigatorMode) => void;
		onExpandedContactKeyChange: (key: string | null) => void;
		communicationChannelIcon: (kind: string) => string;
		senderLabel: (sender: string) => string;
		messageTime: (msg: ConversationMessage) => string;
	}

	let {
		communicationMessages,
		isCommunicationsLoading,
		communicationsError,
		selectedConversationIndex,
		selectedCommunication,
		navigatorMode,
		expandedContactKey,
		isLayoutEditing,
		isWidgetVisible,
		selectCommunication,
		onNavigatorModeChange,
		onExpandedContactKeyChange,
		communicationChannelIcon,
		senderLabel,
		messageTime
	}: Props = $props();

	const selectedMessageId = $derived(selectedCommunication?.message_id ?? null);
	const selectedContactKey = $derived(selectedCommunication ? contactKey(selectedCommunication) : null);
	const contactGroups = $derived.by(() => buildContactGroups(communicationMessages));

	$effect(() => {
		if (navigatorMode === 'contacts' && selectedContactKey && expandedContactKey === null) {
			onExpandedContactKeyChange(selectedContactKey);
		}
	});

	function selectMessage(message: ConversationMessage): void {
		const index = communicationMessages.findIndex((item) => item.message_id === message.message_id);
		if (index >= 0) {
			selectCommunication(index);
		}
	}

	function selectContactGroup(group: ContactGroup): void {
		onExpandedContactKeyChange(expandedContactKey === group.key ? null : group.key);
		selectMessage(group.latestMessage);
	}

	function buildContactGroups(messages: ConversationMessage[]): ContactGroup[] {
		const groups = new Map<string, ContactGroup>();
		for (const message of messages) {
			const key = contactKey(message);
			const existing = groups.get(key);
			if (existing) {
				existing.messages.push(message);
				if (messageTimestamp(message) > messageTimestamp(existing.latestMessage)) {
					existing.latestMessage = message;
				}
				continue;
			}
			groups.set(key, {
				key,
				name: senderLabel(message.sender),
				email: senderEmail(message.sender),
				messages: [message],
				latestMessage: message
			});
		}
		return Array.from(groups.values())
			.map((group) => ({
				...group,
				messages: group.messages.sort((a, b) => messageTimestamp(b) - messageTimestamp(a))
			}))
			.sort((a, b) => messageTimestamp(b.latestMessage) - messageTimestamp(a.latestMessage));
	}

	function contactKey(message: ConversationMessage): string {
		return senderEmail(message.sender).toLowerCase();
	}

	function messageTimestamp(message: ConversationMessage): number {
		const timestamp = Date.parse(message.occurred_at ?? message.projected_at);
		return Number.isFinite(timestamp) ? timestamp : 0;
	}

	function conversationTitle(message: ConversationMessage): string {
		const subject = message.subject.trim();
		return subject || conversationPreview(message);
	}
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-conversation-list" data-widget-hidden={!isWidgetVisible('communications-conversation-list')}>
	<WidgetEditChrome widgetId="communications-conversation-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel conversation-list">
		<header class="conversation-list-head">
			<strong>{navigatorMode === 'threads' ? _('Threads') : _('Contacts')}</strong>
			<div class="navigator-mode-toggle" aria-label={_('Navigator mode')}>
				<button type="button" class:active={navigatorMode === 'threads'} title={_('Threads')} onclick={() => onNavigatorModeChange('threads')}>
					<Icon icon="tabler:messages" width="15" height="15" />
				</button>
				<button type="button" class:active={navigatorMode === 'contacts'} title={_('Contacts')} onclick={() => onNavigatorModeChange('contacts')}>
					<Icon icon="tabler:address-book" width="15" height="15" />
				</button>
			</div>
		</header>
		{#if isCommunicationsLoading}
			<div class="empty-panel">{_('Loading messages...')}</div>
		{:else if communicationsError}
			<div class="empty-panel error">{communicationsError}</div>
		{:else if communicationMessages.length === 0}
			<div class="empty-panel">{_('No local messages yet.')}</div>
		{:else}
			{#if navigatorMode === 'threads'}
				{#each communicationMessages as message, index}
					<button type="button" class:active={selectedConversationIndex === index} onclick={() => selectCommunication(index)}>
						<span class="round-icon cyan"><Icon icon={communicationChannelIcon(message.channel_kind)} width="22" height="22" /></span>
						<span class="conversation-copy">
							<strong class="conversation-sender">{senderLabel(message.sender)}</strong>
							<small class="conversation-subject">{conversationTitle(message)}</small>
						</span>
						{#if 'workflow_state' in message && message.workflow_state}
							<span class="state-badge {message.workflow_state}">{message.workflow_state.replace('_', ' ')}</span>
						{/if}
						<time class="conversation-time">{messageTime(message)}</time>
						{#if message.attachment_count > 0}<b>{message.attachment_count}</b>{/if}
					</button>
				{/each}
			{:else}
				<div class="contact-tree">
					{#each contactGroups as group}
						<div class="contact-tree-group" class:expanded={expandedContactKey === group.key}>
							<button type="button" class="contact-tree-parent" class:active={selectedContactKey === group.key} onclick={() => selectContactGroup(group)}>
								<span class="round-icon cyan"><Icon icon="tabler:user-circle" width="22" height="22" /></span>
								<span class="conversation-copy">
									<strong class="conversation-sender">{group.name}</strong>
									<small class="conversation-subject">{group.email}</small>
								</span>
								<time class="conversation-time">{messageTime(group.latestMessage)}</time>
								<b>{group.messages.length}</b>
							</button>
							{#if expandedContactKey === group.key}
								<div class="contact-tree-children">
									{#each group.messages as message}
										<button type="button" class="contact-tree-child" class:active={selectedMessageId === message.message_id} onclick={() => selectMessage(message)}>
											<span class="conversation-copy">
												<strong class="conversation-sender">{conversationTitle(message)}</strong>
												<small class="conversation-subject">{messageTime(message)}</small>
											</span>
											{#if message.attachment_count > 0}<b>{message.attachment_count}</b>{/if}
										</button>
									{/each}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</section>
</div>
