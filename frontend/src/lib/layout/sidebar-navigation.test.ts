import { describe, expect, it } from 'vitest';

import {
	communicationSidebarItemId,
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
			'agents'
		]);

		expect(primaryWorkspaceNav.map((item) => item.label)).not.toContain('Telegram');
		expect(primaryWorkspaceNav.map((item) => item.label)).not.toContain('WhatsApp');
		expect(primaryWorkspaceNav.map((item) => item.label)).not.toContain('Settings');
	});

	it('defines the Communications second-level navigation in the intended order', () => {
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
			'timeline',
			'persons',
			'projects',
			'tasks',
			'calendar',
			'documents',
			'notes',
			'knowledge',
			'agents'
		]);
		expect(settings.groups).toEqual([
			{
				id: 'communications',
				label: 'Communications',
				icon: 'tabler:messages',
				itemIds: communicationSections.map((section) => communicationSidebarItemId(section.id)),
				separatorBeforeItemIds: ['communications.mail']
			}
		]);
		expect(settings.hiddenItemIds).toEqual([]);
		expect(visibleSidebarItemIds(settings)).toEqual([
			'home',
			'timeline',
			'persons',
			'projects',
			'tasks',
			'calendar',
			'documents',
			'notes',
			'knowledge',
			'agents',
			...communicationSections.map((section) => communicationSidebarItemId(section.id))
		]);
	});

	it('parses custom parent groups, hidden items, and appends missing entries safely', () => {
		const settings = parseSidebarSettings({
			schemaVersion: 2,
			rootItemIds: ['home', 'group:communications', 'group:focus', 'projects', 'unknown'],
			groups: [
				{
					id: 'communications',
					label: 'Comms',
					icon: 'tabler:messages',
					itemIds: ['communications.inbox', 'communications.telegram', 'communications.inbox', 'unknown'],
					separatorBeforeItemIds: ['communications.telegram', 'communications.unified']
				},
				{
					id: 'focus',
					label: 'Focus',
					icon: 'tabler:folder',
					itemIds: ['tasks', 'calendar'],
					separatorBeforeItemIds: ['calendar']
				}
			],
			hiddenItemIds: ['tasks', 'communications.telegram', 'unknown', 'tasks']
		});

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
				'communications.inbox',
				'communications.telegram',
				'communications.unified',
				'communications.waiting',
				'communications.needs_reply',
				'communications.mentions',
				'communications.mail',
				'communications.whatsapp',
				'communications.calls',
				'communications.meetings'
			],
			separatorBeforeItemIds: ['communications.telegram']
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
					itemIds: ['communications.unified', 'communications.inbox'],
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
			hiddenItemIds: ['tasks', 'communications.inbox']
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
					'communications.unified',
					'communications.waiting',
					'communications.needs_reply',
					'communications.mentions',
					'communications.mail',
					'communications.telegram',
					'communications.whatsapp',
					'communications.calls',
					'communications.meetings'
				]
			},
			{ kind: 'group', label: 'Focus', ids: ['calendar'] },
			{ kind: 'item', id: 'projects' },
			{ kind: 'item', id: 'timeline' },
			{ kind: 'item', id: 'persons' },
			{ kind: 'item', id: 'documents' },
			{ kind: 'item', id: 'notes' },
			{ kind: 'item', id: 'knowledge' },
			{ kind: 'item', id: 'agents' }
		]);
	});
});
