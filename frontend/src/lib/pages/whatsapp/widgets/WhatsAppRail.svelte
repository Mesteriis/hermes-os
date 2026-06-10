<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		whatsappClosureCapabilities: unknown[];
		whatsappBlockedCapabilities: unknown[];
		whatsappCapabilities: unknown | null;
		whatsappProviderAccounts: unknown[];
		isWhatsappActionSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		capabilityLabel: (capability: string) => string;
		openAccountDrawer: (target: string) => void;
		ingestWhatsappWebMessageFixture: () => Promise<void>;
		whatsappMessageForm: { account_id: string; provider_chat_id: string; chat_title: string; sender_id: string; sender_display_name: string; text: string };
	}

	let {
		whatsappClosureCapabilities,
		whatsappBlockedCapabilities,
		whatsappCapabilities,
		whatsappProviderAccounts,
		isWhatsappActionSubmitting,
		isLayoutEditing,
		isWidgetVisible,
		capabilityLabel,
		openAccountDrawer,
		ingestWhatsappWebMessageFixture,
		whatsappMessageForm
	}: Props = $props();
</script>

<aside class="stacked-rail whatsapp-rail">
	<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="whatsapp-sync-controls" data-widget-hidden={!isWidgetVisible('whatsapp-sync-controls')}>
		<WidgetEditChrome widgetId="whatsapp-sync-controls" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Account Setup')}</h2>
			<div class="setup-summary-card">
				<span class="round-icon green"><Icon icon="tabler:brand-whatsapp" width="22" height="22" /></span>
				<div>
					<strong>{whatsappProviderAccounts.length} {_('WhatsApp accounts')}</strong>
					<p>{whatsappProviderAccounts.length ? _('Companion session records are available for fixture ingestion.') : _('No WhatsApp Web account record is configured yet.')}</p>
				</div>
			</div>
			<div class="form-actions wide">
				<button type="button" onclick={() => openAccountDrawer('whatsapp')} disabled={isWhatsappActionSubmitting}>{_('Open Wizard')}</button>
			</div>
		</section>

		<section class="panel info-card">
			<h2>{_('Runtime Guardrails')}</h2>
			<div class="health-row"><span>{_('Mode')}</span><strong>{(whatsappCapabilities as Record<string, unknown>)?.runtime_mode ?? _('unknown') as string}</strong></div>
			{#if whatsappClosureCapabilities.length}
				<ul class="detail-list">
					{#each whatsappClosureCapabilities as capability}
						<li>{capabilityLabel((capability as Record<string, unknown>).capability as string)}<em>{(capability as Record<string, unknown>).status as string}</em></li>
					{/each}
				</ul>
			{:else}
				<p>{_('Capability contract is not loaded yet.')}</p>
			{/if}
			{#if whatsappBlockedCapabilities.length}
				<div class="evidence-row">
					<strong>{_('Live Scope')}</strong>
					<p>{whatsappBlockedCapabilities.map((capability) => capabilityLabel((capability as Record<string, unknown>).capability as string)).join(', ')}</p>
				</div>
			{/if}
			{#if (whatsappCapabilities as Record<string, unknown>)?.unsupported_features && ((whatsappCapabilities as Record<string, unknown>).unsupported_features as unknown[]).length}
				<div class="evidence-row">
					<strong>{_('Unsupported')}</strong>
					<p>{((whatsappCapabilities as Record<string, unknown>).unsupported_features as unknown[]).map(capabilityLabel as unknown as (f: unknown) => string).join(', ')}</p>
				</div>
			{/if}
		</section>

		<section class="panel info-card">
			<h2>{_('Fixture Message')}</h2>
			<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void ingestWhatsappWebMessageFixture(); }}>
				<label><span>{_('Account ID')}</span><input bind:value={whatsappMessageForm.account_id} autocomplete="off" /></label>
				<label><span>{_('Chat ID')}</span><input bind:value={whatsappMessageForm.provider_chat_id} autocomplete="off" /></label>
				<label><span>{_('Chat title')}</span><input bind:value={whatsappMessageForm.chat_title} autocomplete="off" /></label>
				<label><span>{_('Sender ID')}</span><input bind:value={whatsappMessageForm.sender_id} autocomplete="off" /></label>
				<label><span>{_('Sender')}</span><input bind:value={whatsappMessageForm.sender_display_name} autocomplete="off" /></label>
				<label class="wide"><span>{_('Text')}</span><textarea bind:value={whatsappMessageForm.text} rows="3"></textarea></label>
				<div class="form-actions wide"><button type="submit" disabled={isWhatsappActionSubmitting || !whatsappMessageForm.text.trim()}>{_('Ingest Fixture')}</button></div>
			</form>
		</section>
	</div>
</aside>
