<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	type WhatsappAccountForm = {
		account_id: string;
		display_name: string;
		external_account_id: string;
		device_name: string;
		local_state_path: string;
	};

	interface Props {
		whatsappAccountForm: WhatsappAccountForm;
		isWhatsappActionSubmitting: boolean;
		setupWhatsappWebFixture: () => Promise<void>;
	}

	let {
		whatsappAccountForm = $bindable(),
		isWhatsappActionSubmitting,
		setupWhatsappWebFixture
	}: Props = $props();
</script>

<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void setupWhatsappWebFixture(); }}>
	<label><span>{_('Account ID')}</span><input bind:value={whatsappAccountForm.account_id} autocomplete="off" /></label>
	<label><span>{_('Display name')}</span><input bind:value={whatsappAccountForm.display_name} autocomplete="off" /></label>
	<label><span>{_('External ID')}</span><input bind:value={whatsappAccountForm.external_account_id} autocomplete="off" /></label>
	<label><span>{_('Device name')}</span><input bind:value={whatsappAccountForm.device_name} autocomplete="off" /></label>
	<label class="wide"><span>{_('Local state path')}</span><input bind:value={whatsappAccountForm.local_state_path} autocomplete="off" /></label>
	<div class="wizard-note wide">{_('WhatsApp Web live runtime remains blocked; this creates a fixture companion-session record.')}</div>
	<div class="form-actions wide"><button type="submit" disabled={isWhatsappActionSubmitting}>{_('Save Fixture')}</button></div>
</form>
