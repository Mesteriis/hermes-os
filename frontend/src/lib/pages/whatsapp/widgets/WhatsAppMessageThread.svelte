<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		selectedWhatsappSession: unknown | null;
		selectedWhatsappMessages: unknown[];
		aiAnalysisResult: unknown | null;
		selectedCommunication: unknown | null;
		isWhatsappLoading: boolean;
		isWhatsappActionSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		whatsappMessageTime: (message: unknown) => string;
		loadWhatsappWebWorkspace: () => Promise<void>;
		ingestWhatsappWebMessageFixture: () => Promise<void>;
		whatsappMessageForm: { provider_message_id: string; sender_display_name: string; text: string };
	}

	let {
		selectedWhatsappSession,
		selectedWhatsappMessages,
		aiAnalysisResult,
		selectedCommunication,
		isWhatsappLoading,
		isWhatsappActionSubmitting,
		isLayoutEditing,
		isWidgetVisible,
		whatsappMessageTime,
		loadWhatsappWebWorkspace,
		ingestWhatsappWebMessageFixture,
		whatsappMessageForm
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="whatsapp-chat-message-surface" data-widget-hidden={!isWidgetVisible('whatsapp-chat-message-surface')}>
	<WidgetEditChrome widgetId="whatsapp-chat-message-surface" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel chat-pane whatsapp-chat-pane">
		{#if selectedWhatsappSession}
			{@const session = selectedWhatsappSession as Record<string, unknown>}
			<header>
				<span class="round-icon cyan"><Icon icon="tabler:brand-whatsapp" width="24" height="24" /></span>
				<div><h2>{session.device_name as string}</h2><p>{session.account_id as string} · {session.link_state as string}</p></div>
				<div class="chat-actions">
					<button type="button" disabled title="Live WhatsApp Web runtime is blocked in WhatsApp foundation"><Icon icon="tabler:world" width="17" height="17" /></button>
					<button type="button" disabled title="Outbound WhatsApp sends require a future policy and runtime contract"><Icon icon="tabler:send-off" width="17" height="17" /></button>
					<button type="button" onclick={() => void loadWhatsappWebWorkspace()} disabled={isWhatsappLoading}><Icon icon="tabler:refresh" width="17" height="17" /></button>
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
				{#if selectedWhatsappMessages.length === 0}
					<div class="empty-panel fill">No WhatsApp Web messages for this session.</div>
				{:else}
					{#each selectedWhatsappMessages.slice().reverse() as message}
						{@const msg = message as Record<string, unknown>}
						<article class="bubble" class:outbound={msg.delivery_state === 'sent' || msg.delivery_state === 'send_dry_run'} class:inbound={msg.delivery_state !== 'sent' && msg.delivery_state !== 'send_dry_run'}>
							<strong>{(msg.sender_display_name ?? msg.sender) as string}</strong><br />
							{msg.text as string}
							<time>{whatsappMessageTime(message)}</time>
						</article>
					{/each}
				{/if}
			</div>
			<form class="telegram-inline-form" onsubmit={(event) => { event.preventDefault(); void ingestWhatsappWebMessageFixture(); }}>
				<input bind:value={whatsappMessageForm.provider_message_id} placeholder="Provider message ID" autocomplete="off" />
				<input bind:value={whatsappMessageForm.sender_display_name} placeholder="Sender" autocomplete="off" />
				<input bind:value={whatsappMessageForm.text} placeholder="Fixture message text" autocomplete="off" />
				<button type="submit" disabled={isWhatsappActionSubmitting || !whatsappMessageForm.text.trim()}><Icon icon="tabler:send" width="17" height="17" />Ingest</button>
			</form>
		{:else}
			<div class="empty-panel fill">Create a WhatsApp Web fixture account before ingesting messages.</div>
		{/if}
	</section>
</div>
