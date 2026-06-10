<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { MailAccountOption } from '$lib/services/communications';

	const _ = (key: string) => t($currentLocale, key);

	interface ComposeForm {
		draft_id: string;
		account_id: string;
		to_text: string;
		cc_text: string;
		bcc_text: string;
		subject: string;
		body: string;
		mode: 'compose' | 'reply' | 'forward';
		in_reply_to: string | null;
		references: string[];
	}

	interface Props {
		isOpen: boolean;
		form: ComposeForm;
		accountOptions: MailAccountOption[];
		isSending: boolean;
		sendError: string;
		statusMessage: string;
		isSendReviewOpen: boolean;
		onClose: () => void;
		onSaveDraft: () => void;
		onOpenSendReview: () => void;
		onCloseSendReview: () => void;
		onConfirmSend: () => void;
	}

	let {
		isOpen,
		form = $bindable(),
		accountOptions,
		isSending,
		sendError,
		statusMessage,
		isSendReviewOpen,
		onClose,
		onSaveDraft,
		onOpenSendReview,
		onCloseSendReview,
		onConfirmSend
	}: Props = $props();

	const selectedAccount = $derived(
		accountOptions.find((account) => account.accountId === form.account_id) ?? null
	);
	const canSend = $derived(Boolean(selectedAccount?.canSend));
	const canOpenSendReview = $derived(
		Boolean(form.account_id && form.to_text.trim() && form.subject.trim() && form.body.trim() && canSend && !isSending)
	);
	const composeTitle = $derived(
		form.mode === 'reply' ? _('Reply') : form.mode === 'forward' ? _('Forward') : _('New Message')
	);
</script>

{#if isOpen}
	<button type="button" class="drawer-backdrop" onclick={onClose} aria-label={_('Close compose')}></button>
	<aside class="account-drawer compose-wide" aria-label={_('Compose email')}>
		<header>
			<div><p>{_('Compose')}</p><h2>{composeTitle}</h2></div>
			<button type="button" class="icon-button" onclick={onClose} aria-label={_('Close')}><Icon icon="tabler:x" width="18" height="18" /></button>
		</header>
		<form class="setup-form" onsubmit={(event) => { event.preventDefault(); onSaveDraft(); }}>
			<label class="wide">
				<span>{_('From')}</span>
				<select bind:value={form.account_id} disabled={accountOptions.length === 0}>
					{#if accountOptions.length === 0}
						<option value="">{_('No mail accounts')}</option>
					{:else}
						{#each accountOptions as account}
							<option value={account.accountId}>{account.label} · {account.email}</option>
						{/each}
					{/if}
				</select>
			</label>
			<label><span>{_('To')}</span><input bind:value={form.to_text} placeholder={_('recipient@example.com')} autocomplete="off" /></label>
			<label><span>{_('CC')}</span><input bind:value={form.cc_text} placeholder={_('cc@example.com')} autocomplete="off" /></label>
			<label><span>{_('BCC')}</span><input bind:value={form.bcc_text} placeholder={_('bcc@example.com')} autocomplete="off" /></label>
			<label><span>{_('Subject')}</span><input bind:value={form.subject} placeholder={_('Email subject')} autocomplete="off" /></label>
			<label class="wide"><span>{_('Body')}</span><textarea bind:value={form.body} rows="8" placeholder={_('Write your message...')}></textarea></label>
			{#if selectedAccount && !selectedAccount.canSend}
				<p class="form-status error wide">{selectedAccount.sendUnavailableReason}</p>
			{/if}
			{#if sendError}<p class="form-status error wide">{sendError}</p>{/if}
			{#if statusMessage}<p class="form-status wide">{statusMessage}</p>{/if}
			<div class="form-actions wide">
				<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />{_('Save Draft')}</button>
				<button type="button" onclick={onOpenSendReview} disabled={!canOpenSendReview}><Icon icon="tabler:send" width="16" height="16" />{isSending ? _('Sending') : _('Send')}</button>
			</div>
		</form>
	</aside>
	{#if isSendReviewOpen}
		<button type="button" class="drawer-backdrop modal-backdrop" onclick={onCloseSendReview} aria-label={_('Close send review')}></button>
		<section class="account-modal send-review-modal" aria-label={_('Review send')}>
			<header>
				<div><p>{_('Provider write')}</p><h2>{_('Review send')}</h2></div>
				<button type="button" class="icon-button" onclick={onCloseSendReview} aria-label={_('Close')}><Icon icon="tabler:x" width="18" height="18" /></button>
			</header>
			<div class="send-review-grid">
				<span>{_('From')}</span><strong>{selectedAccount?.label ?? form.account_id}</strong>
				<span>{_('To')}</span><strong>{form.to_text}</strong>
				<span>{_('Subject')}</span><strong>{form.subject}</strong>
			</div>
			<p class="form-status">{_('This will send through the selected provider account.')}</p>
			<div class="form-actions">
				<button type="button" class="secondary-action" onclick={onCloseSendReview}>{_('Cancel')}</button>
				<button type="button" class="primary-button" onclick={onConfirmSend} disabled={isSending}><Icon icon="tabler:send" width="16" height="16" />{isSending ? _('Sending') : _('Confirm Send')}</button>
			</div>
		</section>
	{/if}
{/if}
