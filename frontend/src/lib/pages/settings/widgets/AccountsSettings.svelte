<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { ProviderAccount, CalendarAccount } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		providerAccounts: ProviderAccount[];
		calendarAccounts: CalendarAccount[];

		emailProviderAccounts: ProviderAccount[];
		telegramProviderAccounts: ProviderAccount[];
		whatsappProviderAccounts: ProviderAccount[];

		onOpenAccountDrawer: (target?: string) => void;
		accountProviderIconFn: (providerKind: string) => string;
		accountProviderLabelFn: (providerKind: string) => string;
		accountUpdatedLabelFn: (account: ProviderAccount) => string;
		formatDateTimeFn: (value: string | null) => string;
	}

	let {
		providerAccounts,
		calendarAccounts,
		emailProviderAccounts,
		telegramProviderAccounts,
		whatsappProviderAccounts,
		onOpenAccountDrawer,
		accountProviderIconFn,
		accountProviderLabelFn,
		accountUpdatedLabelFn,
		formatDateTimeFn
	}: Props = $props();
</script>

<div class="settings-account-layout">
	<section class="panel account-section">
		<header class="panel-title-row">
			<div><h2>Mail Accounts</h2><p>Gmail OAuth, iCloud app-password and generic IMAP records.</p></div>
			<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer('mail')}><Icon icon="tabler:plus" width="16" height="16" />Add Mail</button>
		</header>
		<div class="account-card-grid">
			{#if emailProviderAccounts.length === 0}
				<div class="empty-panel fill">No mail accounts configured.</div>
			{:else}
				{#each emailProviderAccounts as account}
					<article class="account-card">
						<span class="round-icon cyan"><Icon icon={accountProviderIconFn(account.provider_kind)} width="22" height="22" /></span>
						<div>
							<strong>{account.display_name}</strong>
							<p>{account.external_account_id || account.account_id}</p>
							<small>{accountProviderLabelFn(account.provider_kind)} · updated {accountUpdatedLabelFn(account)}</small>
						</div>
						<code>{account.account_id}</code>
					</article>
				{/each}
			{/if}
		</div>
	</section>

	<section class="panel account-section">
		<header class="panel-title-row">
			<div><h2>Calendar Accounts</h2><p>Local and external calendar metadata accounts.</p></div>
			<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer('calendar')}><Icon icon="tabler:calendar-plus" width="16" height="16" />Add Calendar</button>
		</header>
		<div class="account-card-grid">
			{#if calendarAccounts.length === 0}
				<div class="empty-panel fill">No calendar accounts configured.</div>
			{:else}
				{#each calendarAccounts as account}
					<article class="account-card">
						<span class="round-icon green"><Icon icon="tabler:calendar" width="22" height="22" /></span>
						<div>
							<strong>{account.account_name}</strong>
							<p>{account.email || account.account_id}</p>
							<small>{account.provider} · updated {formatDateTimeFn(account.updated_at)}</small>
						</div>
						<code>{account.account_id}</code>
					</article>
				{/each}
			{/if}
		</div>
	</section>

	<section class="panel account-section">
		<header class="panel-title-row">
			<div><h2>Telegram Accounts</h2><p>User and bot accounts used by Telegram ingestion and automation policies.</p></div>
			<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer('telegram')}><Icon icon="tabler:brand-telegram" width="16" height="16" />Add Telegram</button>
		</header>
		<div class="account-card-grid">
			{#if telegramProviderAccounts.length === 0}
				<div class="empty-panel fill">No Telegram accounts configured.</div>
			{:else}
				{#each telegramProviderAccounts as account}
					<article class="account-card">
						<span class="round-icon purple"><Icon icon={accountProviderIconFn(account.provider_kind)} width="22" height="22" /></span>
						<div>
							<strong>{account.display_name}</strong>
							<p>{account.external_account_id || account.account_id}</p>
							<small>{accountProviderLabelFn(account.provider_kind)} · updated {accountUpdatedLabelFn(account)}</small>
						</div>
						<code>{account.account_id}</code>
					</article>
				{/each}
			{/if}
		</div>
	</section>

	<section class="panel account-section">
		<header class="panel-title-row">
			<div><h2>Other Provider Accounts</h2><p>WhatsApp Web and future communication providers.</p></div>
			<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer('whatsapp')}><Icon icon="tabler:brand-whatsapp" width="16" height="16" />Add WhatsApp</button>
		</header>
		<div class="account-card-grid">
			{#if whatsappProviderAccounts.length === 0}
				<div class="empty-panel fill">No WhatsApp Web accounts configured.</div>
			{:else}
				{#each whatsappProviderAccounts as account}
					<article class="account-card">
						<span class="round-icon green"><Icon icon={accountProviderIconFn(account.provider_kind)} width="22" height="22" /></span>
						<div>
							<strong>{account.display_name}</strong>
							<p>{account.external_account_id || account.account_id}</p>
							<small>{accountProviderLabelFn(account.provider_kind)} · updated {accountUpdatedLabelFn(account)}</small>
						</div>
						<code>{account.account_id}</code>
					</article>
				{/each}
			{/if}
		</div>
	</section>
</div>
