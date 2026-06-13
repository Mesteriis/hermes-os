<script lang="ts">
	import { setupWhatsappWebFixtureAccount } from '$lib/api';
	import { providerKindLabel } from '$lib/services/accounts';
	import WhatsappAccountWizard from './WhatsappAccountWizard.svelte';

	interface Props {
		onAccountSaved?: () => Promise<void>;
	}

	let { onAccountSaved }: Props = $props();

	let isWhatsappActionSubmitting = $state(false);
	let setupMessage = $state('');
	let setupError = $state('');
	let whatsappAccountForm = $state({
		account_id: 'whatsapp-primary',
		display_name: 'Primary WhatsApp Web',
		external_account_id: 'whatsapp-fixture-device',
		device_name: 'Hermes Desktop Fixture',
		local_state_path: 'docker/data/whatsapp/whatsapp-primary'
	});

	async function reloadAfterSave() {
		if (onAccountSaved) {
			await onAccountSaved();
		}
	}

	async function setupWhatsappWebFixture() {
		if (isWhatsappActionSubmitting) {
			return;
		}
		isWhatsappActionSubmitting = true;
		setupMessage = '';
		setupError = '';
		try {
			const result = await setupWhatsappWebFixtureAccount({
				account_id: whatsappAccountForm.account_id,
				provider_kind: 'whatsapp_web',
				display_name: whatsappAccountForm.display_name,
				external_account_id: whatsappAccountForm.external_account_id,
				device_name: whatsappAccountForm.device_name,
				local_state_path: whatsappAccountForm.local_state_path
			});
			setupMessage = `${providerKindLabel(result.provider_kind)} account ${result.account_id} saved`;
			await reloadAfterSave();
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'WhatsApp Web fixture setup failed';
		} finally {
			isWhatsappActionSubmitting = false;
		}
	}
</script>

<WhatsappAccountWizard
	bind:whatsappAccountForm
	{isWhatsappActionSubmitting}
	{setupWhatsappWebFixture}
/>
{#if setupMessage}<p class="setup-state success">{setupMessage}</p>{/if}
{#if setupError}<p class="setup-state error">{setupError}</p>{/if}
