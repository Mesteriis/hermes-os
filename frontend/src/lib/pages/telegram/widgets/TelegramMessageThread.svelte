<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		selectedTelegramChat: unknown | null;
		selectedTelegramMessages: unknown[];
		aiAnalysisResult: unknown | null;
		selectedCommunication: unknown | null;
		isTelegramLoading: boolean;
		isTelegramActionSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		telegramMessageTime: (message: unknown) => string;
		loadTelegramWorkspace: () => Promise<void>;
		ingestTelegramMessageFixture: () => Promise<void>;
		telegramMessageForm: { provider_message_id: string; sender_display_name: string; text: string };
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
		telegramMessageTime,
		loadTelegramWorkspace,
		ingestTelegramMessageFixture,
		telegramMessageForm
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="telegram-message-thread" data-widget-hidden={!isWidgetVisible('telegram-message-thread')}>
	<WidgetEditChrome widgetId="telegram-message-thread" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel chat-pane telegram-chat-pane">
		{#if selectedTelegramChat}
			{@const chat = selectedTelegramChat as Record<string, unknown>}
			<header>
				<span class="round-icon cyan"><Icon icon="tabler:brand-telegram" width="24" height="24" /></span>
				<div><h2>{chat.title as string}</h2><p>{chat.account_id as string} · {chat.provider_chat_id as string}</p></div>
				<div class="chat-actions">
					<button type="button" disabled title="1:1 audio call controls are backend-foundation only in this Telegram foundation"><Icon icon="tabler:phone" width="17" height="17" /></button>
					<button type="button" disabled title="Video calls are outside this Telegram foundation"><Icon icon="tabler:video" width="17" height="17" /></button>
					<button type="button" onclick={() => void loadTelegramWorkspace()} disabled={isTelegramLoading}><Icon icon="tabler:refresh" width="17" height="17" /></button>
				</div>
			</header>
			<div class="chat-body">
				{#if aiAnalysisResult && (aiAnalysisResult as Record<string, unknown>).message_id === (selectedCommunication as { message_id?: string })?.message_id}
					<article class="ai-analysis-card">
						<strong><Icon icon="tabler:sparkles" width="16" height="16" />AI Analysis</strong>
						{#if (aiAnalysisResult as Record<string, unknown>).category}<p><em>Category:</em> {(aiAnalysisResult as Record<string, unknown>).category as string}</p>{/if}
						{#if (aiAnalysisResult as Record<string, unknown>).summary}<p><em>Summary:</em> {(aiAnalysisResult as Record<string, unknown>).summary as string}</p>{/if}
						{#if (aiAnalysisResult as Record<string, unknown>).importance_score != null}<p><em>Importance:</em> {(aiAnalysisResult as Record<string, unknown>).importance_score as number}/100</p>{/if}
						<p><em>State:</em> <span class="state-badge {(aiAnalysisResult as Record<string, unknown>).workflow_state as string}">{(aiAnalysisResult as Record<string, unknown>).workflow_state as string}</span></p>
					</article>
				{/if}
				{#if selectedTelegramMessages.length === 0}
					<div class="empty-panel fill">No messages for this chat.</div>
				{:else}
					{#each selectedTelegramMessages.slice().reverse() as message}
						{@const msg = message as Record<string, unknown>}
						<article class="bubble" class:outbound={msg.delivery_state === 'sent' || msg.delivery_state === 'send_dry_run'} class:inbound={msg.delivery_state !== 'sent' && msg.delivery_state !== 'send_dry_run'}>
							<strong>{(msg.sender_display_name ?? msg.sender) as string}</strong><br />
							{msg.text as string}
							<time>{telegramMessageTime(message)}</time>
						</article>
					{/each}
				{/if}
			</div>
			<form class="telegram-inline-form" onsubmit={(event) => { event.preventDefault(); void ingestTelegramMessageFixture(); }}>
				<input bind:value={telegramMessageForm.provider_message_id} placeholder="Provider message ID" autocomplete="off" />
				<input bind:value={telegramMessageForm.sender_display_name} placeholder="Sender" autocomplete="off" />
				<input bind:value={telegramMessageForm.text} placeholder="Fixture message text" autocomplete="off" />
				<button type="submit" disabled={isTelegramActionSubmitting || !telegramMessageForm.text.trim()}><Icon icon="tabler:send" width="17" height="17" />Ingest</button>
			</form>
		{:else}
			<div class="empty-panel fill">Create a Telegram fixture account and ingest a message.</div>
		{/if}
	</section>
</div>
