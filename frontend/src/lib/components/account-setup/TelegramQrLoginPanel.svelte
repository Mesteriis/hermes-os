<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { TelegramQrLoginStatusResponse } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		telegramQrLogin: TelegramQrLoginStatusResponse | null;
		telegramQrIsPreparing: boolean;
		accountId: string;
		telegramQrStatusLabel: (status: string) => string;
	}

	let { telegramQrLogin, telegramQrIsPreparing, accountId, telegramQrStatusLabel }: Props = $props();
</script>

<div class="qr-login-panel telegram-qr-panel wide" class:large={Boolean(telegramQrLogin?.qr_svg)}>
	{#if telegramQrLogin?.qr_svg}
		<div class="qr-svg" aria-label={_('Telegram QR code')}>{@html telegramQrLogin.qr_svg}</div>
	{:else if telegramQrIsPreparing}
		<div class="qr-skeleton" aria-label={_('Loading Telegram QR code')}>
			<span></span>
			<span></span>
			<span></span>
		</div>
	{:else}
		<Icon icon="tabler:qrcode" width="58" height="58" />
	{/if}
	<div class="qr-login-copy">
		<strong>{telegramQrLogin ? telegramQrStatusLabel(telegramQrLogin.status) : telegramQrIsPreparing ? _('Preparing QR login') : _('QR login')}</strong>
		<p>{telegramQrLogin?.message ?? (telegramQrIsPreparing ? _('Requesting a fresh Telegram QR code...') : _('Telegram QR login starts automatically for this account.'))}</p>
		<p class="qr-account-hint">{_('Account ID')}: {accountId}</p>
		{#if telegramQrLogin?.qr_link}
			<a href={telegramQrLogin.qr_link}>{_('Open Telegram login link')}</a>
		{/if}
	</div>
</div>
