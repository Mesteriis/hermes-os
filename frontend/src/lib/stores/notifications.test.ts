import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import {
	consumePendingNotificationTarget,
	dismissNotification,
	notificationCount,
	notificationItems,
	openNotificationTarget,
	pendingNotificationTarget,
	setNotificationItems,
	type NotificationItem
} from './notifications';

const items: NotificationItem[] = [
	{
		id: 'n1',
		source: 'communications',
		icon: 'tabler:mail',
		title: 'First',
		body: 'First body',
		meta: 'Inbox',
		time: '2026-06-09T10:00:00Z',
		targetSection: 'unified',
		messageId: 'message-1'
	},
	{
		id: 'n2',
		source: 'telegram',
		icon: 'tabler:brand-telegram',
		title: 'Second',
		body: 'Second body',
		meta: 'Telegram',
		time: '2026-06-09T11:00:00Z',
		targetSection: 'telegram'
	}
];

describe('notifications store', () => {
	beforeEach(() => {
		setNotificationItems([]);
		pendingNotificationTarget.set(null);
	});

	it('derives count from non-dismissed notifications and stores selected targets', () => {
		setNotificationItems(items);

		expect(get(notificationItems).map((item) => item.id)).toEqual(['n2', 'n1']);
		expect(get(notificationCount)).toBe(2);

		dismissNotification('n2');

		expect(get(notificationItems).map((item) => item.id)).toEqual(['n1']);
		expect(get(notificationCount)).toBe(1);

		openNotificationTarget(items[0]);

		expect(get(pendingNotificationTarget)?.messageId).toBe('message-1');
		expect(consumePendingNotificationTarget()?.id).toBe('n1');
		expect(get(pendingNotificationTarget)).toBeNull();
	});
});
