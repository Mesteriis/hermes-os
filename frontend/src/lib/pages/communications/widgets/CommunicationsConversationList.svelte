<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		communicationMessages: unknown[];
		isCommunicationsLoading: boolean;
		communicationsError: string;
		selectedConversationIndex: number;
		selectedCommunication: unknown | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		selectCommunication: (index: number) => void;
		communicationChannelIcon: (kind: string) => string;
		senderLabel: (sender: string) => string;
		messageTime: (msg: unknown) => string;
	}

	let {
		communicationMessages,
		isCommunicationsLoading,
		communicationsError,
		selectedConversationIndex,
		selectedCommunication,
		isLayoutEditing,
		isWidgetVisible,
		selectCommunication,
		communicationChannelIcon,
		senderLabel,
		messageTime
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-conversation-list" data-widget-hidden={!isWidgetVisible('communications-conversation-list')}>
	<WidgetEditChrome widgetId="communications-conversation-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel conversation-list">
		<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder={_('Search conversations...')} /></label>
		{#if isCommunicationsLoading}
			<div class="empty-panel">{_('Loading messages...')}</div>
		{:else if communicationsError}
			<div class="empty-panel error">{communicationsError}</div>
		{:else if communicationMessages.length === 0}
			<div class="empty-panel">{_('No local messages yet.')}</div>
		{:else}
			{#each communicationMessages as message, index}
				<button type="button" class:active={selectedConversationIndex === index} onclick={() => selectCommunication(index)}>
					<span class="round-icon cyan">
						<Icon icon={communicationChannelIcon((message as Record<string, unknown>).channel_kind as string)} width="22" height="22" />
					</span>
					<img src="/assets/hermes-reference-avatar.png" alt="" />
					<span>
						<strong>{senderLabel((message as Record<string, unknown>).sender as string)}</strong>
						<small>{(message as Record<string, unknown>).subject as string}</small>
						<em>{(message as Record<string, unknown>).body_text_preview as string}</em>
					</span>
					{#if (message as unknown as { workflow_state?: string }).workflow_state}
						<span class="state-badge {(message as unknown as { workflow_state?: string }).workflow_state}">{(message as unknown as { workflow_state?: string }).workflow_state?.replace('_', ' ')}</span>
					{/if}
					<time>{messageTime(message)}</time>
					{#if ((message as Record<string, unknown>).attachment_count as number) > 0}<b>{(message as Record<string, unknown>).attachment_count as number}</b>{/if}
				</button>
			{/each}
		{/if}
	</section>
</div>
