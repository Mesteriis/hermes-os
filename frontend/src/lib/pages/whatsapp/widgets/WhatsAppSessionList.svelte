<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		whatsappSessions: unknown[];
		selectedWhatsappSessionId: string;
		isWhatsappLoading: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onSelectSession: (session: unknown) => void;
		formatDateTime: (date: string | null) => string;
	}

	let {
		whatsappSessions,
		selectedWhatsappSessionId,
		isWhatsappLoading,
		isLayoutEditing,
		isWidgetVisible,
		onSelectSession,
		formatDateTime
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="whatsapp-account-session-metadata" data-widget-hidden={!isWidgetVisible('whatsapp-account-session-metadata')}>
	<WidgetEditChrome widgetId="whatsapp-account-session-metadata" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel conversation-list">
		<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder={_('Search WhatsApp sessions...')} /></label>
		{#if isWhatsappLoading && whatsappSessions.length === 0}
			<div class="empty-panel">{_('Loading WhatsApp Web state...')}</div>
		{:else if whatsappSessions.length === 0}
			<div class="empty-panel">{_('No WhatsApp Web sessions saved yet.')}</div>
		{:else}
			{#each whatsappSessions as session}
				{@const s = session as Record<string, unknown>}
				<button type="button" class:active={selectedWhatsappSessionId === (s.session_id as string)} onclick={() => onSelectSession(session)}>
					<span class="round-icon cyan"><Icon icon="tabler:brand-whatsapp" width="22" height="22" /></span>
					<img src="/assets/hermes-reference-avatar.png" alt="" />
					<span>
						<strong>{s.device_name as string}</strong>
						<small>{s.account_id as string} · {s.companion_runtime as string}</small>
						<em>{s.link_state as string}</em>
					</span>
					<time>{formatDateTime((s.last_sync_at ?? s.updated_at) as string | null)}</time>
				</button>
			{/each}
		{/if}
	</section>
</div>
