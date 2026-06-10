<script lang="ts">
	import './vault.css';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { VaultStatus } from '$lib/api';
	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		wizardStep: 'intro' | 'entropy' | 'biometric' | 'recovery' | 'done';
		status: VaultStatus | null;
		statusError: string;
		entropyEventsCount: number;
		wizardError: string;
		wizardMessage: string;
		recovery: { path?: string; recovery_phrase?: string } | null;
		isActionSubmitting: boolean;
		onStartWizard: () => void;
		onCreateVault: () => Promise<void>;
		onUnlockVault: () => Promise<void>;
		onExportRecovery: () => Promise<void>;
		onContinue: () => void;
		onEntropyMove: (event: MouseEvent) => void;
	}

	let {
		wizardStep,
		status,
		statusError,
		entropyEventsCount,
		wizardError,
		wizardMessage,
		recovery,
		isActionSubmitting,
		onStartWizard,
		onCreateVault,
		onUnlockVault,
		onExportRecovery,
		onContinue,
		onEntropyMove
	}: Props = $props();
</script>

<section class="vault-onboarding" aria-label={_('Secure vault onboarding')} onmousemove={onEntropyMove}>
	<div class="vault-panel">
		<div class="vault-panel__header">
			<div class="vault-emblem"><Icon icon="tabler:shield-lock" width="30" height="30" /></div>
			<div>
				<p class="vault-kicker">{_('Hermes Secure Vault')}</p>
				<h1>{status?.state === 'locked' ? _('Unlock Secure Vault') : _('Create Your Personal Secure Vault')}</h1>
			</div>
		</div>

		{#if status?.state === 'locked'}
			<div class="vault-step">
				<p>{_('Hermes Hub needs the secure vault unlocked before it can save provider credentials on this device.')}</p>
				<div class="vault-actions">
					<button type="button" onclick={onUnlockVault} disabled={isActionSubmitting}>{_('Unlock Existing Vault')}</button>
				</div>
			</div>
		{:else if wizardStep === 'intro'}
			<div class="vault-step">
				<p>{_('Hermes Hub encrypts credentials stored on this device. Secrets live in a dedicated host vault; PostgreSQL keeps only non-secret bindings.')}</p>
				<p class="vault-warning">{_('If you lose the recovery phrase or file, access to encrypted secrets may become impossible.')}</p>
				<div class="vault-actions">
					<button type="button" onclick={onStartWizard}>{_('Start Entropy Collection')}</button>
				</div>
			</div>
		{:else if wizardStep === 'entropy'}
			<div class="vault-step">
				<p>{_('Move your mouse around the screen. Hermes combines OS randomness, timing entropy and mouse movement before creating the master key.')}</p>
				<div class="vault-entropy-canvas">
					<div class="vault-entropy-meter">
						<span>{_('Entropy')}</span>
						<strong>{status?.entropy_progress ?? 0}%</strong>
					</div>
					<progress class="vault-progress" value={status?.entropy_progress ?? 0} max="100"></progress>
					<p>{Math.min(entropyEventsCount, 2000)} / 2000 events</p>
				</div>
				<div class="vault-actions">
					<button type="button" onclick={onCreateVault} disabled={(status?.entropy_progress ?? 0) < 100 || isActionSubmitting}>{_('Create Vault')}</button>
				</div>
			</div>
		{:else if wizardStep === 'biometric'}
			<div class="vault-step">
				<p>{_('Vault material is ready. In release runtime Hermes will use macOS Keychain as source-of-truth for the master key. Docker dev uses the configured dev key path.')}</p>
				<div class="vault-actions">
					<button type="button" onclick={onCreateVault} disabled={isActionSubmitting}>{_('Create Vault')}</button>
					<button type="button" onclick={onUnlockVault} disabled={isActionSubmitting}>{_('Unlock Existing Vault')}</button>
				</div>
			</div>
		{:else if wizardStep === 'recovery'}
			<div class="vault-step">
				<p>{_('Export recovery material before continuing. Store the phrase and file safely outside Hermes.')}</p>
				<p class="vault-warning">{_('Without the recovery phrase or file, restoration after reinstall or Keychain access loss is impossible.')}</p>
				<div class="vault-actions">
					<button type="button" onclick={onExportRecovery} disabled={isActionSubmitting}>{_('Export Recovery')}</button>
				</div>
			</div>
		{:else}
			<div class="vault-step">
				<p>{_('Vault is ready.')} {_('Recovery file')}: <strong>{recovery?.path ?? '~/.hermes/vault/hermes-recovery.key'}</strong></p>
				{#if recovery?.recovery_phrase}
					<div class="vault-recovery-phrase">{recovery.recovery_phrase}</div>
				{/if}
				<div class="vault-actions">
					<button type="button" onclick={onContinue}>{_('Continue')}</button>
				</div>
			</div>
		{/if}

		{#if wizardMessage}<p class="vault-state success">{wizardMessage}</p>{/if}
		{#if wizardError}<p class="vault-state error">{wizardError}</p>{/if}
		{#if statusError}<p class="vault-state error">{statusError}</p>{/if}
	</div>
</section>
