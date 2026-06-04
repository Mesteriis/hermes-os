<script lang="ts">
	import { onMount } from 'svelte';
	import {
		completeGmailOAuthSetup,
		fetchV1Status,
		setupImapAccount,
		startGmailOAuthSetup,
		type GmailOAuthStartResponse,
		type V1Status
	} from '$lib/api';

	const apiBaseUrl = import.meta.env.VITE_HERMES_API_BASE_URL ?? 'http://127.0.0.1:8080';
	const apiToken = import.meta.env.VITE_HERMES_LOCAL_API_TOKEN ?? 'change-me-local-api-token';
	const actorId = import.meta.env.VITE_HERMES_ACTOR_ID ?? 'desktop-shell';

	let status = $state<V1Status | null>(null);
	let errorMessage = $state('');
	let isLoading = $state(true);
	let selectedProvider = $state<'gmail' | 'icloud' | 'imap'>('gmail');
	let setupMessage = $state('');
	let setupError = $state('');
	let isSetupSubmitting = $state(false);
	let gmailPending = $state<GmailOAuthStartResponse | null>(null);
	let gmailAuthorizationCode = $state('');
	let gmailForm = $state({
		account_id: 'gmail-primary',
		display_name: 'Primary Gmail',
		external_account_id: '',
		client_id: '',
		client_secret: '',
		redirect_uri: `${apiBaseUrl.replace(/\/+$/, '')}/api/v1/email-accounts/gmail/oauth/callback`
	});
	let imapForm = $state({
		account_id: 'icloud-primary',
		display_name: 'Primary iCloud',
		external_account_id: '',
		host: 'imap.mail.me.com',
		port: 993,
		tls: true,
		mailbox: 'INBOX',
		username: '',
		password: '',
		secret_kind: 'app_password' as 'app_password' | 'password'
	});

	const surfaces: Array<{
		key: keyof V1Status['surfaces'];
		label: string;
		description: string;
	}> = [
		{ key: 'messages', label: 'Messages', description: 'Canonical communication timeline' },
		{ key: 'contacts', label: 'Contacts', description: 'Projected contact identities' },
		{ key: 'search', label: 'Search', description: 'Derived full-text index' },
		{ key: 'documents', label: 'Documents', description: 'Imported local document records' },
		{ key: 'account_setup', label: 'Accounts', description: 'Provider setup and secret vault' }
	];

	onMount(async () => {
		try {
			status = await fetchV1Status(apiBaseUrl, apiToken, actorId);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Unknown status error';
		} finally {
			isLoading = false;
		}
	});

	function selectProvider(provider: 'gmail' | 'icloud' | 'imap') {
		selectedProvider = provider;
		setupMessage = '';
		setupError = '';

		if (provider === 'icloud') {
			imapForm = {
				...imapForm,
				account_id: imapForm.account_id || 'icloud-primary',
				display_name: imapForm.display_name || 'Primary iCloud',
				host: 'imap.mail.me.com',
				port: 993,
				tls: true,
				mailbox: imapForm.mailbox || 'INBOX',
				secret_kind: 'app_password'
			};
		}
		if (provider === 'imap') {
			imapForm = {
				...imapForm,
				account_id: imapForm.account_id === 'icloud-primary' ? 'imap-primary' : imapForm.account_id,
				display_name:
					imapForm.display_name === 'Primary iCloud' ? 'Primary IMAP' : imapForm.display_name,
				host: imapForm.host === 'imap.mail.me.com' ? '' : imapForm.host,
				secret_kind: 'password'
			};
		}
	}

	async function startGmailSetup() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			gmailPending = await startGmailOAuthSetup(apiBaseUrl, apiToken, actorId, {
				account_id: gmailForm.account_id,
				display_name: gmailForm.display_name,
				external_account_id: gmailForm.external_account_id,
				client_id: gmailForm.client_id,
				client_secret: gmailForm.client_secret || undefined,
				redirect_uri: gmailForm.redirect_uri
			});
			setupMessage = 'Gmail OAuth grant started';
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Gmail setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function completeGmailSetup() {
		if (!gmailPending) {
			setupError = 'Gmail OAuth grant has not been started';
			return;
		}

		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const result = await completeGmailOAuthSetup(apiBaseUrl, apiToken, actorId, {
				setup_id: gmailPending.setup_id,
				state: gmailPending.state,
				authorization_code: gmailAuthorizationCode
			});
			setupMessage = `Gmail account ${result.account_id} saved`;
			gmailAuthorizationCode = '';
			gmailPending = null;
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Gmail setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function saveImapAccount() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const result = await setupImapAccount(apiBaseUrl, apiToken, actorId, {
				account_id: imapForm.account_id,
				provider_kind: selectedProvider === 'icloud' ? 'icloud' : 'imap',
				display_name: imapForm.display_name,
				external_account_id: imapForm.external_account_id,
				host: imapForm.host,
				port: Number(imapForm.port),
				tls: imapForm.tls,
				mailbox: imapForm.mailbox,
				username: imapForm.username,
				password: imapForm.password,
				secret_kind: imapForm.secret_kind
			});
			setupMessage = `Mail account ${result.account_id} saved`;
			imapForm = { ...imapForm, password: '' };
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Mail account setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}
</script>

<svelte:head>
	<title>Hermes Hub</title>
	<meta
		name="description"
		content="Hermes Hub desktop shell for local backend status."
	/>
</svelte:head>

<main class="shell">
	<aside class="sidebar" aria-label="Hermes Hub navigation">
		<div class="brand">
			<p class="brand-label">Hermes Hub</p>
			<p class="brand-subtitle">Desktop shell</p>
		</div>

		<nav aria-label="Primary">
			<a class="active" href="/">Status</a>
			<a href="/">Messages</a>
			<a href="/">Contacts</a>
			<a href="/">Search</a>
			<a href="/">Documents</a>
		</nav>

		<div class="connection">
			<span class:error={errorMessage} class:ready={status}>Backend</span>
			<strong>{status ? 'connected' : errorMessage ? 'unavailable' : 'checking'}</strong>
		</div>
	</aside>

	<section class="content" aria-labelledby="status-heading">
		<header class="page-header">
			<div>
				<p class="section-label">Local V1 API</p>
				<h1 id="status-heading">Local Memory Core</h1>
			</div>
			<div class="endpoint">
				<span>Endpoint</span>
				<strong>{apiBaseUrl}</strong>
			</div>
		</header>

		{#if status}
			<section class="status-panel" aria-label="Backend status">
				<div class="version-card">
					<span>Version</span>
					<strong>{status.version}</strong>
				</div>

				<div class="surface-grid">
					{#each surfaces as surface}
						<article class="surface-card">
							<div>
								<h2>{surface.label}</h2>
								<p>{surface.description}</p>
							</div>
							<span class:enabled={status.surfaces[surface.key]} class="state">
								{status.surfaces[surface.key] ? 'enabled' : 'disabled'}
							</span>
						</article>
					{/each}
				</div>
			</section>

			<section class="account-panel" aria-labelledby="account-setup-heading">
				<header class="panel-header">
					<div>
						<p class="section-label">Provider Accounts</p>
						<h2 id="account-setup-heading">Add Account</h2>
					</div>
					<div class="provider-tabs" aria-label="Account provider">
						<button
							type="button"
							class:active={selectedProvider === 'gmail'}
							onclick={() => selectProvider('gmail')}>Gmail</button
						>
						<button
							type="button"
							class:active={selectedProvider === 'icloud'}
							onclick={() => selectProvider('icloud')}>iCloud</button
						>
						<button
							type="button"
							class:active={selectedProvider === 'imap'}
							onclick={() => selectProvider('imap')}>IMAP</button
						>
					</div>
				</header>

				{#if selectedProvider === 'gmail'}
					<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
						<label>
							<span>Account ID</span>
							<input bind:value={gmailForm.account_id} autocomplete="off" />
						</label>
						<label>
							<span>Display name</span>
							<input bind:value={gmailForm.display_name} autocomplete="off" />
						</label>
						<label>
							<span>Gmail address</span>
							<input bind:value={gmailForm.external_account_id} autocomplete="email" />
						</label>
						<label>
							<span>OAuth client ID</span>
							<input bind:value={gmailForm.client_id} autocomplete="off" />
						</label>
						<label>
							<span>OAuth client secret</span>
							<input bind:value={gmailForm.client_secret} type="password" autocomplete="off" />
						</label>
						<label class="wide">
							<span>Redirect URI</span>
							<input bind:value={gmailForm.redirect_uri} autocomplete="off" />
						</label>
						<div class="form-actions wide">
							<button type="button" onclick={startGmailSetup} disabled={isSetupSubmitting}>
								Start OAuth
							</button>
						</div>
					</form>

					{#if gmailPending}
						<div class="oauth-box">
							<a href={gmailPending.authorization_url} target="_blank" rel="noreferrer">
								Open Google consent
							</a>
							<label>
								<span>Authorization code</span>
								<input bind:value={gmailAuthorizationCode} autocomplete="off" />
							</label>
							<button type="button" onclick={completeGmailSetup} disabled={isSetupSubmitting}>
								Complete Gmail
							</button>
						</div>
					{/if}
				{:else}
					<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
						<label>
							<span>Account ID</span>
							<input bind:value={imapForm.account_id} autocomplete="off" />
						</label>
						<label>
							<span>Display name</span>
							<input bind:value={imapForm.display_name} autocomplete="off" />
						</label>
						<label>
							<span>Email address</span>
							<input bind:value={imapForm.external_account_id} autocomplete="email" />
						</label>
						<label>
							<span>Username</span>
							<input bind:value={imapForm.username} autocomplete="username" />
						</label>
						<label>
							<span>Host</span>
							<input bind:value={imapForm.host} autocomplete="off" />
						</label>
						<label>
							<span>Port</span>
							<input bind:value={imapForm.port} type="number" min="1" max="65535" />
						</label>
						<label>
							<span>Mailbox</span>
							<input bind:value={imapForm.mailbox} autocomplete="off" />
						</label>
						<label>
							<span>Password</span>
							<input bind:value={imapForm.password} type="password" autocomplete="current-password" />
						</label>
						<label class="checkbox-row">
							<input bind:checked={imapForm.tls} type="checkbox" />
							<span>TLS</span>
						</label>
						<div class="form-actions">
							<button type="button" onclick={saveImapAccount} disabled={isSetupSubmitting}>
								Save Account
							</button>
						</div>
					</form>
				{/if}

				{#if setupMessage}
					<p class="setup-state success">{setupMessage}</p>
				{/if}
				{#if setupError}
					<p class="setup-state error">{setupError}</p>
				{/if}
			</section>
		{:else if errorMessage}
			<section class="notice error-notice" aria-label="Backend status error">
				<h2>Backend status unavailable</h2>
				<p>{errorMessage}</p>
			</section>
		{:else if isLoading}
			<section class="notice" aria-label="Backend status loading">
				<h2>Loading backend status</h2>
				<p>Checking the local V1 API with the desktop shell actor.</p>
			</section>
		{/if}
	</section>
</main>

<style>
	:global(body) {
		margin: 0;
		font-family:
			Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
		color: #182033;
		background: #f5f6f8;
	}

	:global(*) {
		box-sizing: border-box;
	}

	.shell {
		display: grid;
		grid-template-columns: 260px minmax(760px, 1fr);
		min-height: 100vh;
	}

	.sidebar {
		display: grid;
		grid-template-rows: auto 1fr auto;
		border-right: 1px solid #d9dee7;
		background: #ffffff;
		padding: 24px 18px;
	}

	.brand {
		margin-bottom: 28px;
	}

	.brand-label {
		margin: 0;
		font-size: 20px;
		font-weight: 700;
		letter-spacing: 0;
	}

	.brand-subtitle {
		margin: 4px 0 0;
		color: #64748b;
		font-size: 13px;
	}

	nav {
		display: grid;
		align-content: start;
		gap: 6px;
	}

	nav a {
		border-radius: 6px;
		color: #334155;
		font-size: 14px;
		font-weight: 600;
		padding: 9px 10px;
		text-decoration: none;
	}

	nav a:hover,
	nav a.active {
		background: #eef2f7;
		color: #0f172a;
	}

	.connection {
		border-top: 1px solid #e2e8f0;
		display: grid;
		gap: 4px;
		padding-top: 16px;
	}

	.connection span {
		color: #64748b;
		font-size: 12px;
		text-transform: uppercase;
	}

	.connection span.ready {
		color: #157f3b;
	}

	.connection span.error {
		color: #b42318;
	}

	.connection strong {
		font-size: 14px;
	}

	.content {
		padding: 32px;
	}

	.page-header {
		align-items: start;
		display: flex;
		justify-content: space-between;
		gap: 32px;
		margin-bottom: 24px;
	}

	.section-label,
	.endpoint span,
	.version-card span {
		color: #64748b;
		font-size: 12px;
		font-weight: 700;
		margin: 0 0 8px;
		text-transform: uppercase;
	}

	h1 {
		font-size: 28px;
		line-height: 1.2;
		margin: 0;
	}

	.endpoint {
		border: 1px solid #d9dee7;
		border-radius: 8px;
		background: #ffffff;
		min-width: 280px;
		padding: 12px 14px;
	}

	.endpoint strong {
		display: block;
		font-size: 14px;
	}

	.status-panel {
		display: grid;
		gap: 16px;
		max-width: 980px;
	}

	.version-card,
	.surface-card,
	.notice {
		border: 1px solid #d9dee7;
		border-radius: 8px;
		background: #ffffff;
	}

	.version-card {
		padding: 18px;
	}

	.version-card strong {
		display: block;
		font-size: 22px;
	}

	.surface-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(280px, 1fr));
		gap: 12px;
	}

	.surface-card {
		align-items: center;
		display: flex;
		justify-content: space-between;
		gap: 18px;
		min-height: 112px;
		padding: 16px;
	}

	.surface-card h2,
	.notice h2 {
		font-size: 16px;
		line-height: 1.25;
		margin: 0;
	}

	.surface-card p,
	.notice p {
		color: #64748b;
		font-size: 13px;
		line-height: 1.45;
		margin: 6px 0 0;
	}

	.state {
		border: 1px solid #d6dbe4;
		border-radius: 999px;
		color: #475569;
		flex: 0 0 auto;
		font-size: 12px;
		font-weight: 700;
		padding: 5px 9px;
		text-transform: uppercase;
	}

	.state.enabled {
		background: #ecfdf3;
		border-color: #abefc6;
		color: #067647;
	}

	.notice {
		max-width: 720px;
		padding: 20px;
	}

	.error-notice {
		border-color: #fecdca;
	}

	.error-notice h2,
	.error-notice p {
		color: #b42318;
	}

	.account-panel {
		background: #ffffff;
		border: 1px solid #d9dee7;
		border-radius: 8px;
		display: grid;
		gap: 18px;
		margin-top: 22px;
		max-width: 980px;
		padding: 18px;
	}

	.panel-header {
		align-items: center;
		display: flex;
		gap: 20px;
		justify-content: space-between;
	}

	.panel-header h2 {
		font-size: 18px;
		line-height: 1.25;
		margin: 0;
	}

	.provider-tabs {
		background: #eef2f7;
		border-radius: 8px;
		display: inline-grid;
		gap: 2px;
		grid-template-columns: repeat(3, minmax(72px, 1fr));
		padding: 3px;
	}

	.provider-tabs button,
	.form-actions button,
	.oauth-box button {
		border: 0;
		border-radius: 6px;
		cursor: pointer;
		font: inherit;
		font-size: 13px;
		font-weight: 700;
	}

	.provider-tabs button {
		background: transparent;
		color: #475569;
		padding: 7px 10px;
	}

	.provider-tabs button.active {
		background: #ffffff;
		color: #0f172a;
	}

	.setup-form {
		display: grid;
		gap: 12px;
		grid-template-columns: repeat(2, minmax(240px, 1fr));
	}

	.setup-form label,
	.oauth-box label {
		display: grid;
		gap: 6px;
	}

	.setup-form label span,
	.oauth-box label span {
		color: #475569;
		font-size: 12px;
		font-weight: 700;
	}

	.setup-form input,
	.oauth-box input {
		background: #ffffff;
		border: 1px solid #cbd5e1;
		border-radius: 6px;
		color: #0f172a;
		font: inherit;
		font-size: 14px;
		min-width: 0;
		padding: 9px 10px;
	}

	.setup-form input:focus,
	.oauth-box input:focus {
		border-color: #2563eb;
		outline: 2px solid #dbeafe;
	}

	.setup-form .wide,
	.oauth-box {
		grid-column: 1 / -1;
	}

	.checkbox-row {
		align-content: end;
		display: flex !important;
		gap: 8px !important;
		min-height: 64px;
	}

	.checkbox-row input {
		height: 18px;
		width: 18px;
	}

	.form-actions {
		align-content: end;
		display: grid;
	}

	.form-actions button,
	.oauth-box button {
		background: #1d4ed8;
		color: #ffffff;
		min-height: 38px;
		padding: 9px 13px;
	}

	.form-actions button:disabled,
	.oauth-box button:disabled {
		background: #94a3b8;
		cursor: not-allowed;
	}

	.oauth-box {
		background: #f8fafc;
		border: 1px solid #d9dee7;
		border-radius: 8px;
		display: grid;
		gap: 12px;
		padding: 14px;
	}

	.oauth-box a {
		color: #1d4ed8;
		font-size: 14px;
		font-weight: 700;
	}

	.setup-state {
		border-radius: 6px;
		font-size: 13px;
		font-weight: 700;
		margin: 0;
		padding: 10px 12px;
	}

	.setup-state.success {
		background: #ecfdf3;
		color: #067647;
	}

	.setup-state.error {
		background: #fef3f2;
		color: #b42318;
	}
</style>
