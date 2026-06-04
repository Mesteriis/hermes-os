<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchV1Status, type V1Status } from '$lib/api';

	const apiBaseUrl = import.meta.env.VITE_HERMES_API_BASE_URL ?? 'http://127.0.0.1:8080';
	const apiToken = import.meta.env.VITE_HERMES_LOCAL_API_TOKEN ?? 'change-me-local-api-token';
	const actorId = import.meta.env.VITE_HERMES_ACTOR_ID ?? 'desktop-shell';

	let status = $state<V1Status | null>(null);
	let errorMessage = $state('');
	let isLoading = $state(true);

	const surfaces: Array<{
		key: keyof V1Status['surfaces'];
		label: string;
		description: string;
	}> = [
		{ key: 'messages', label: 'Messages', description: 'Canonical communication timeline' },
		{ key: 'contacts', label: 'Contacts', description: 'Projected contact identities' },
		{ key: 'search', label: 'Search', description: 'Derived full-text index' },
		{ key: 'documents', label: 'Documents', description: 'Imported local document records' }
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
</style>
