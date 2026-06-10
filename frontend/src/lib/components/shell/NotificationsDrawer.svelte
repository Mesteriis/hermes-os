<script lang="ts">
	import './notifications.css';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		isNotificationsDrawerOpen,
		closeNotificationsDrawer,
		dismissNotification,
		toggleNotificationExpanded,
		isNotificationExpanded,
		expandedNotificationIds
	} from '$lib/stores/notifications';

	const _ = (key: string) => t($currentLocale, key);

	type NotificationItem = {
		id: string;
		source: string;
		icon: string;
		title: string;
		body: string;
		meta: string;
		time: string | null;
		messageId?: string;
		accountId?: string;
		providerChatId?: string | null;
		targetSection?: string;
		isDemo?: boolean;
	};

	interface Props {
		notificationItems: NotificationItem[];
		onOpenTarget: (notification: Record<string, unknown>) => void | Promise<void>;
		formatDateTime: (value: string | null) => string;
	}

	let { notificationItems, onOpenTarget, formatDateTime }: Props = $props();

	function notificationNeedsExpansion(notification: { body: string }) {
		return notification.body.length > 120;
	}
</script>

{#if $isNotificationsDrawerOpen}
	<button
		type="button"
		class="notifications-backdrop"
		aria-label={_('Close notifications')}
		onclick={closeNotificationsDrawer}
	></button>
	<aside id="notifications-drawer" class="notifications-drawer" aria-label={_('Notifications')}>
		<header>
			<div>
				<h2>{_('Notifications')}</h2>
				<p>{notificationItems.length} {_('active')}</p>
			</div>
			<button type="button" class="icon-button" aria-label={_('Close notifications')} onclick={closeNotificationsDrawer}>
				<Icon icon="tabler:x" width="18" height="18" />
			</button>
		</header>
		{#if notificationItems.length === 0}
			<div class="notifications-empty">
				<Icon icon="tabler:bell-check" width="28" height="28" />
				<p>{_('No active notifications.')}</p>
			</div>
		{:else}
			<div class="notifications-list">
				{#each notificationItems as notification}
					<article>
						<button
							type="button"
							class="notification-target"
							onclick={() => void onOpenTarget(notification)}
						>
							<span class="round-icon cyan">
								<Icon icon={notification.icon} width="18" height="18" />
							</span>
							<span>
								<strong>{notification.title}</strong>
								<small>{notification.meta} · {formatDateTime(notification.time)}</small>
								<em class:expanded={isNotificationExpanded(notification.id, $expandedNotificationIds)}>{notification.body}</em>
							</span>
						</button>
						{#if notificationNeedsExpansion(notification)}
							<button
								type="button"
								class="notification-expand"
								aria-expanded={isNotificationExpanded(notification.id, $expandedNotificationIds)}
								onclick={() => toggleNotificationExpanded(notification.id)}
							>
								{isNotificationExpanded(notification.id, $expandedNotificationIds) ? _('Show less') : _('Show full text')}
							</button>
						{/if}
						<button
							type="button"
							class="notification-dismiss"
							aria-label={_('Dismiss notification')}
							onclick={(event) => {
								event.stopPropagation();
								dismissNotification(notification.id);
							}}
						>
							<Icon icon="tabler:trash" width="16" height="16" />
						</button>
					</article>
				{/each}
			</div>
		{/if}
	</aside>
{/if}
