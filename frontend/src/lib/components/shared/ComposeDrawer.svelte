<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	interface ComposeForm {
		draft_id: string;
		account_id: string;
		to_text: string;
		cc_text: string;
		subject: string;
		body: string;
	}

	interface Props {
		isOpen: boolean;
		form: ComposeForm;
		onClose: () => void;
		onSaveDraft: () => void;
	}

	let { isOpen, form = $bindable(), onClose, onSaveDraft }: Props = $props();
</script>

{#if isOpen}
	<button type="button" class="drawer-backdrop" onclick={onClose} aria-label="Close compose"></button>
	<aside class="account-drawer" aria-label="Compose email">
		<header>
			<div><p>{_('Compose')}</p><h2>New Message</h2></div>
			<button type="button" class="icon-button" onclick={onClose} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
		</header>
		<form class="setup-form" onsubmit={(event) => { event.preventDefault(); onSaveDraft(); }}>
			<label><span>To</span><input bind:value={form.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
			<label><span>CC</span><input bind:value={form.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
			<label><span>Subject</span><input bind:value={form.subject} placeholder="Email subject" autocomplete="off" /></label>
			<label class="wide"><span>Body</span><textarea bind:value={form.body} rows="8" placeholder="Write your message..."></textarea></label>
			<div class="form-actions wide">
				<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
				<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
			</div>
		</form>
	</aside>
{/if}
