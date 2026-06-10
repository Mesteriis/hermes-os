<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		telegramClosureCapabilities: unknown[];
		telegramBlockedCapabilities: unknown[];
		telegramCapabilities: unknown | null;
		automationTemplates: unknown[];
		telegramCalls: unknown[];
		selectedTelegramCall: unknown | null;
		selectedTelegramCallId: string | null;
		callTranscript: unknown | null;
		telegramSendDryRunResult: unknown | null;
		telegramProviderAccounts: unknown[];
		isTelegramActionSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		capabilityLabel: (capability: string) => string;
		openAccountDrawer: (target: string) => void;
		selectTelegramCall: (call: unknown) => void;
		saveTelegramAutomationTemplate: () => Promise<void>;
		saveTelegramAutomationPolicy: () => Promise<void>;
		runTelegramAutomationDryRun: () => Promise<void>;
		saveTelegramCallFixture: () => Promise<void>;
		saveCallTranscriptFixtureFromUi: () => Promise<void>;
		automationTemplateForm: { template_id: string; name: string; body_template: string; required_variables_text: string };
		automationPolicyForm: { policy_id: string; template_id: string; name: string; account_id: string; allowed_chat_ids_text: string; trigger_kind: string; max_sends_per_hour: number; quiet_hours_text: string; conditions_text: string; enabled: boolean };
		telegramSendForm: { policy_id: string; provider_chat_id: string; variables_text: string; source_context_text: string };
		telegramCallForm: { call_id: string; provider_call_id: string; account_id: string; provider_chat_id: string; direction: string; call_state: string; metadata_text: string };
		transcriptForm: { transcript_id: string; source_audio_ref: string; language_code: string; always_on_policy: boolean };
	}

	let {
		telegramClosureCapabilities,
		telegramBlockedCapabilities,
		telegramCapabilities,
		automationTemplates,
		telegramCalls,
		selectedTelegramCall,
		selectedTelegramCallId,
		callTranscript,
		telegramSendDryRunResult,
		telegramProviderAccounts,
		isTelegramActionSubmitting,
		isLayoutEditing,
		isWidgetVisible,
		capabilityLabel,
		openAccountDrawer,
		selectTelegramCall,
		saveTelegramAutomationTemplate,
		saveTelegramAutomationPolicy,
		runTelegramAutomationDryRun,
		saveTelegramCallFixture,
		saveCallTranscriptFixtureFromUi,
		automationTemplateForm,
		automationPolicyForm,
		telegramSendForm,
		telegramCallForm,
		transcriptForm
	}: Props = $props();
</script>

<aside class="stacked-rail telegram-rail">
	<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="telegram-sync-controls" data-widget-hidden={!isWidgetVisible('telegram-sync-controls')}>
		<WidgetEditChrome widgetId="telegram-sync-controls" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
		<h2>Account Setup</h2>
		<div class="setup-summary-card">
			<span class="round-icon purple"><Icon icon="tabler:brand-telegram" width="22" height="22" /></span>
			<div>
				<strong>{telegramProviderAccounts.length} Telegram accounts</strong>
				<p>{telegramProviderAccounts.length ? 'User and bot records are available for ingestion and policies.' : 'No Telegram account record is configured yet.'}</p>
			</div>
		</div>
		<div class="form-actions wide">
			<button type="button" onclick={() => openAccountDrawer('telegram')} disabled={isTelegramActionSubmitting}>Open Wizard</button>
		</div>
		</section>

		<section class="panel info-card">
		<h2>Runtime Guardrails</h2>
		<div class="health-row"><span>Mode</span><strong>{(telegramCapabilities as Record<string, unknown>)?.runtime_mode ?? 'unknown' as string}</strong></div>
		{#if telegramClosureCapabilities.length}
			<ul class="detail-list">
				{#each telegramClosureCapabilities as capability}
					<li>{capabilityLabel((capability as Record<string, unknown>).capability as string)}<em>{(capability as Record<string, unknown>).status as string}</em></li>
				{/each}
			</ul>
		{:else}
			<p>Capability contract is not loaded yet.</p>
		{/if}
		{#if telegramBlockedCapabilities.length}
			<div class="evidence-row">
				<strong>Blocked Live Runtime</strong>
				<p>{telegramBlockedCapabilities.map((capability) => capabilityLabel((capability as Record<string, unknown>).capability as string)).join(', ')}</p>
			</div>
		{/if}
		{#if (telegramCapabilities as Record<string, unknown>)?.unsupported_features && ((telegramCapabilities as Record<string, unknown>).unsupported_features as unknown[]).length}
			<div class="evidence-row">
				<strong>Telegram Scope</strong>
				<p>{((telegramCapabilities as Record<string, unknown>).unsupported_features as unknown[]).map(capabilityLabel as unknown as (f: unknown) => string).join(', ')}</p>
			</div>
		{/if}
		</section>

		<section class="panel info-card">
		<h2>Template</h2>
		<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveTelegramAutomationTemplate(); }}>
			<label><span>Template ID</span><input bind:value={automationTemplateForm.template_id} autocomplete="off" /></label>
			<label><span>Name</span><input bind:value={automationTemplateForm.name} autocomplete="off" /></label>
			<label class="wide"><span>Body</span><textarea bind:value={automationTemplateForm.body_template} rows="3"></textarea></label>
			<label class="wide"><span>Required variables</span><input bind:value={automationTemplateForm.required_variables_text} autocomplete="off" /></label>
			<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting}>Save Template</button></div>
		</form>
		{#if automationTemplates.length}
			<ul class="detail-list">
				{#each (automationTemplates as unknown[]).slice(0, 3) as template}
					<li>{(template as Record<string, unknown>).name as string}<em>{(template as Record<string, unknown>).template_id as string}</em></li>
				{/each}
			</ul>
		{/if}
		</section>

		<section class="panel info-card">
		<h2>Policy</h2>
		<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveTelegramAutomationPolicy(); }}>
			<label><span>Policy ID</span><input bind:value={automationPolicyForm.policy_id} autocomplete="off" /></label>
			<label><span>Template ID</span><input bind:value={automationPolicyForm.template_id} autocomplete="off" /></label>
			<label><span>Name</span><input bind:value={automationPolicyForm.name} autocomplete="off" /></label>
			<label><span>Account ID</span><input bind:value={automationPolicyForm.account_id} autocomplete="off" /></label>
			<label class="wide"><span>Allowed chat IDs</span><input bind:value={automationPolicyForm.allowed_chat_ids_text} autocomplete="off" /></label>
			<label><span>Trigger</span><input bind:value={automationPolicyForm.trigger_kind} autocomplete="off" /></label>
			<label><span>Max/hour</span><input bind:value={automationPolicyForm.max_sends_per_hour} type="number" min="1" max="100" /></label>
			<label class="wide"><span>Quiet hours JSON</span><textarea bind:value={automationPolicyForm.quiet_hours_text} rows="2"></textarea></label>
			<label class="wide"><span>Conditions JSON</span><textarea bind:value={automationPolicyForm.conditions_text} rows="2"></textarea></label>
			<label class="checkbox-row"><input bind:checked={automationPolicyForm.enabled} type="checkbox" /><span>Enabled</span></label>
			<div class="form-actions"><button type="submit" disabled={isTelegramActionSubmitting}>Save Policy</button></div>
		</form>
		</section>

		<section class="panel info-card">
		<h2>Dry Run</h2>
		<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void runTelegramAutomationDryRun(); }}>
			<label><span>Policy ID</span><input bind:value={telegramSendForm.policy_id} autocomplete="off" /></label>
			<label><span>Chat ID</span><input bind:value={telegramSendForm.provider_chat_id} autocomplete="off" /></label>
			<label class="wide"><span>Variables JSON</span><textarea bind:value={telegramSendForm.variables_text} rows="3"></textarea></label>
			<label class="wide"><span>Source context JSON</span><textarea bind:value={telegramSendForm.source_context_text} rows="2"></textarea></label>
			<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting}>Run Dry-run</button></div>
		</form>
		{#if telegramSendDryRunResult}
			<div class="evidence-row">
				<strong>{(telegramSendDryRunResult as Record<string, unknown>).status as string}</strong>
				<p>{(telegramSendDryRunResult as Record<string, unknown>).rendered_text as string}</p>
				<small>{(telegramSendDryRunResult as Record<string, unknown>).rendered_preview_hash as string}</small>
			</div>
		{/if}
		</section>
	</div>

	<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="telegram-selected-chat-metadata" data-widget-hidden={!isWidgetVisible('telegram-selected-chat-metadata')}>
		<WidgetEditChrome widgetId="telegram-selected-chat-metadata" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
		<h2>Calls</h2>
		{#if telegramCalls.length}
			{#each (telegramCalls as unknown[]).slice(0, 4) as call}
				<button type="button" class="collection-row as-button" class:active={(selectedTelegramCall as Record<string, unknown>)?.call_id === (call as Record<string, unknown>).call_id} onclick={() => selectTelegramCall(call)}>
					<span>{(call as Record<string, unknown>).provider_chat_id as string}</span>
					<em>{(call as Record<string, unknown>).call_state as string}</em>
				</button>
			{/each}
		{:else}
			<p>No calls saved.</p>
		{/if}
		<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveTelegramCallFixture(); }}>
			<label><span>Call ID</span><input bind:value={telegramCallForm.call_id} autocomplete="off" /></label>
			<label><span>Provider call ID</span><input bind:value={telegramCallForm.provider_call_id} autocomplete="off" /></label>
			<label><span>Account ID</span><input bind:value={telegramCallForm.account_id} autocomplete="off" /></label>
			<label><span>Chat ID</span><input bind:value={telegramCallForm.provider_chat_id} autocomplete="off" /></label>
			<label><span>Direction</span><select bind:value={telegramCallForm.direction}><option value="incoming">Incoming</option><option value="outgoing">Outgoing</option></select></label>
			<label><span>State</span><select bind:value={telegramCallForm.call_state}><option value="ringing">Ringing</option><option value="active">Active</option><option value="ended">Ended</option><option value="missed">Missed</option><option value="declined">Declined</option><option value="failed">Failed</option></select></label>
			<label class="wide"><span>Metadata JSON</span><textarea bind:value={telegramCallForm.metadata_text} rows="2"></textarea></label>
			<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting}>Save Call</button></div>
		</form>
		</section>

		<section class="panel info-card">
		<h2>Transcript</h2>
		{#if selectedTelegramCall}
			<div class="health-row"><span>Selected call</span><strong>{(selectedTelegramCall as Record<string, unknown>).call_id as string}</strong></div>
		{/if}
		{#if callTranscript}
			<div class="evidence-row">
				<strong>{(callTranscript as Record<string, unknown>).transcript_status as string} · {(callTranscript as Record<string, unknown>).stt_provider as string}</strong>
				<p>{(callTranscript as Record<string, unknown>).transcript_text as string}</p>
			</div>
		{:else}
			<p>No transcript for selected call.</p>
		{/if}
		<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveCallTranscriptFixtureFromUi(); }}>
			<label><span>Transcript ID</span><input bind:value={transcriptForm.transcript_id} autocomplete="off" /></label>
			<label><span>Audio ref</span><input bind:value={transcriptForm.source_audio_ref} autocomplete="off" /></label>
			<label><span>Language</span><input bind:value={transcriptForm.language_code} autocomplete="off" /></label>
			<label class="checkbox-row"><input bind:checked={transcriptForm.always_on_policy} type="checkbox" /><span>Always-on policy</span></label>
			<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting || !selectedTelegramCallId}>Save Transcript</button></div>
		</form>
		</section>
	</div>
</aside>
