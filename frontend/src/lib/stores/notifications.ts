import { derived, get, writable } from 'svelte/store';
import type { CommunicationSectionId } from '$lib/layout';

export type NotificationItem = {
	id: string;
	source: 'telegram' | 'communications';
	icon: string;
	title: string;
	body: string;
	meta: string;
	time: string | null;
	messageId?: string;
	accountId?: string;
	providerChatId?: string | null;
	targetSection?: CommunicationSectionId;
	isDemo?: boolean;
};

export const isNotificationsDrawerOpen = writable(false);
export const dismissedNotificationIds = writable<string[]>([]);
export const expandedNotificationIds = writable<string[]>([]);
const rawNotificationItems = writable<NotificationItem[]>([]);
export const pendingNotificationTarget = writable<NotificationItem | null>(null);

export const notificationItems = derived(
	[rawNotificationItems, dismissedNotificationIds],
	([$rawNotificationItems, $dismissedNotificationIds]) => {
		const dismissed = new Set($dismissedNotificationIds);
		return $rawNotificationItems
			.filter((notification) => !dismissed.has(notification.id))
			.sort((left, right) => new Date(right.time ?? 0).getTime() - new Date(left.time ?? 0).getTime())
			.slice(0, 12);
	}
);

export const notificationCount = derived(notificationItems, ($notificationItems) => $notificationItems.length);

export function setNotificationItems(items: NotificationItem[]): void {
	rawNotificationItems.set([...items]);
}

export function toggleNotificationsDrawer(): void {
	isNotificationsDrawerOpen.update((v) => !v);
}

export function closeNotificationsDrawer(): void {
	isNotificationsDrawerOpen.set(false);
}

export function dismissNotification(notificationId: string): void {
	dismissedNotificationIds.update((ids) =>
		ids.includes(notificationId) ? ids : [...ids, notificationId]
	);
}

export function toggleNotificationExpanded(notificationId: string): void {
	expandedNotificationIds.update((ids) =>
		ids.includes(notificationId) ? ids.filter((id) => id !== notificationId) : [...ids, notificationId]
	);
}

export function isNotificationExpanded(notificationId: string, expandedIds: string[]): boolean {
	return expandedIds.includes(notificationId);
}

export function openNotificationTarget(notification: NotificationItem): void {
	pendingNotificationTarget.set(notification);
	closeNotificationsDrawer();
}

export function consumePendingNotificationTarget(): NotificationItem | null {
	const target = get(pendingNotificationTarget);
	pendingNotificationTarget.set(null);
	return target;
}
