import { describe, expect, it } from 'vitest';

import {
	communicationSections,
	communicationSectionViewId,
	defaultSidebarSettings,
	parseSidebarSettings,
	primaryWorkspaceNav,
	resolveSidebarRootEntries,
	visibleSidebarItemIds,
	type CommunicationSectionId
} from './sidebar-navigation';

describe('sidebar navigation architecture', () => {
	it('keeps only workspace domains in the root navigation', () => {
		expect(primaryWorkspaceNav.map((item) => item.id)).toEqual([
			'home',
			'communications',
			'timeline',
			'persons',
			'projects',
			'tasks',
			'calendar',
			'documents',
			'notes',
			'knowledge',
			'review',
			'agents'
		]);

		expect(primaryWorkspaceNav.map((item) => item.label)).not.toContain('Telegram');
		expect(primaryWorkspaceNav.map((item) => item.label)).not.toContain('WhatsApp');
		expect(primaryWorkspaceNav.map((item) => item.label)).not.toContain('Settings');
	});

	it('defines the internal Communications filters and sources in the intended order', () => {
		expect(communicationSections.map((item) => item.id)).toEqual([
			'unified',
			'inbox',
			'waiting',
			'needs_reply',
			'mentions',
			'mail',
			'telegram',
			'whatsapp',
			'calls',
			'meetings'
		]);
	});

	it.each([
		['telegram', 'telegram'],
		['whatsapp', 'whatsapp'],
		['unified', 'communications'],
		['mail', 'communications'],
		['calls', 'communications'],
		['meetings', 'communications']
	] satisfies Array<[CommunicationSectionId, string]>)(
		'maps Communications section %s to internal view %s',
		(sectionId, expectedViewId) => {
			expect(communicationSectionViewId(sectionId)).toBe(expectedViewId);
		}
	);

	it('uses the current root navigation order as the default sidebar settings', () => {
		const settings = defaultSidebarSettings();

		expect(settings.rootItemIds).toEqual([
			'home',
			'group:communications',
			'persons',
			'projects',
			'tasks',
			'calendar',
			'documents',
			'notes',
			'knowledge',
			'review',
			'agents'
		]);
		expect(settings.groups).toEqual([
			{
				id: 'communications',
				label: 'Communications',
				icon: 'tabler:messages',
				itemIds: [
					'communications.mail',
					'communications.telegram',
					'communications.whatsapp',
					'communications.calls',
					'communications.meetings',
					'timeline'
				],
				separatorBeforeItemIds: []
			}
		]);
		expect(settings.hiddenItemIds).toEqual([]);
		expect(visibleSidebarItemIds(settings)).toEqual([
			'home',
			'persons',
			'projects',
			'tasks',
			'calendar',
			'documents',
			'notes',
			'knowledge',
			'review',
			'agents',
			'communications.mail',
			'communications.telegram',
			'communications.whatsapp',
			'communications.calls',
			'communications.meetings',
			'timeline'
		]);
	});

	it('migrates v2 custom parent groups and removes mail filters from sidebar items', () => {
		const settings = parseSidebarSettings({
			schemaVersion: 2,
			rootItemIds: ['home', 'group:communications', 'timeline', 'group:focus', 'projects', 'unknown'],
			groups: [
				{
					id: 'communications',
					label: 'Comms',
					icon: 'tabler:messages',
					itemIds: [
						'communications.inbox',
						'communications.telegram',
						'communications.inbox',
						'communications.needs_reply',
						'unknown'
					],
					separatorBeforeItemIds: ['communications.telegram', 'communications.unified']
				},
				{
					id: 'focus',
					label: 'Focus',
					icon: 'tabler:folder',
					itemIds: ['tasks', 'calendar', 'timeline'],
					separatorBeforeItemIds: ['calendar']
				}
			],
			hiddenItemIds: ['tasks', 'communications.telegram', 'communications.inbox', 'unknown', 'tasks']
		});

		expect(settings.schemaVersion).toBe(3);
		expect(settings.rootItemIds.slice(0, 4)).toEqual([
			'home',
			'group:communications',
			'group:focus',
			'projects'
		]);
		expect(settings.groups[0]).toEqual({
			id: 'communications',
			label: 'Comms',
			icon: 'tabler:messages',
			itemIds: [
				'communications.telegram',
				'communications.mail',
				'communications.whatsapp',
				'communications.calls',
				'communications.meetings',
				'timeline'
			],
			separatorBeforeItemIds: []
		});
		expect(settings.groups[1]).toEqual({
			id: 'focus',
			label: 'Focus',
			icon: 'tabler:folder',
			itemIds: ['tasks', 'calendar'],
			separatorBeforeItemIds: ['calendar']
		});
		expect(settings.hiddenItemIds).toEqual(['tasks', 'communications.telegram']);
	});

	it('resolves root entries and group children without hidden items', () => {
		const settings = parseSidebarSettings({
			schemaVersion: 2,
			rootItemIds: ['home', 'group:communications', 'group:focus', 'projects'],
			groups: [
				{
					id: 'communications',
					label: 'Communications',
					icon: 'tabler:messages',
					itemIds: ['communications.mail', 'communications.telegram'],
					separatorBeforeItemIds: []
				},
				{
					id: 'focus',
					label: 'Focus',
					icon: 'tabler:folder',
					itemIds: ['tasks', 'calendar'],
					separatorBeforeItemIds: ['calendar']
				}
			],
			hiddenItemIds: ['tasks', 'communications.telegram']
		});

		const entries = resolveSidebarRootEntries(primaryWorkspaceNav, settings);

		expect(entries.map((entry) => {
			if (entry.kind === 'group') {
				return {
					kind: entry.kind,
					label: entry.group.label,
					ids: entry.group.items.map((item) => item.itemId)
				};
			}
			return {
				kind: entry.kind,
				id: entry.item.itemId
			};
		})).toEqual([
			{ kind: 'item', id: 'home' },
			{
				kind: 'group',
				label: 'Communications',
				ids: [
					'communications.mail',
					'communications.whatsapp',
					'communications.calls',
					'communications.meetings',
					'timeline'
				]
			},
			{ kind: 'group', label: 'Focus', ids: ['calendar'] },
			{ kind: 'item', id: 'projects' },
			{ kind: 'item', id: 'persons' },
			{ kind: 'item', id: 'documents' },
			{ kind: 'item', id: 'notes' },
			{ kind: 'item', id: 'knowledge' },
			{ kind: 'item', id: 'review' },
			{ kind: 'item', id: 'agents' }
		]);
	});
});
