export type PrimaryNavId =
	| 'home'
	| 'communications'
	| 'timeline'
	| 'persons'
	| 'projects'
	| 'tasks'
	| 'calendar'
	| 'documents'
	| 'notes'
	| 'knowledge'
	| 'agents';

export type CommunicationSectionId =
	| 'unified'
	| 'inbox'
	| 'waiting'
	| 'needs_reply'
	| 'mentions'
	| 'mail'
	| 'telegram'
	| 'whatsapp'
	| 'calls'
	| 'meetings';

export type SidebarViewId = PrimaryNavId | 'telegram' | 'whatsapp' | 'settings' | 'organizations';
export type SidebarPrimaryItemId = Exclude<PrimaryNavId, 'communications'>;
export type CommunicationSidebarItemId = `communications.${CommunicationSectionId}`;
export type SidebarItemId = SidebarPrimaryItemId | CommunicationSidebarItemId;
export type SidebarRootItemId = SidebarPrimaryItemId | `group:${string}`;

export type PrimaryWorkspaceNavItem = {
	id: PrimaryNavId;
	label: string;
	icon: string;
};

export const SIDEBAR_SETTINGS_SCHEMA_VERSION = 2;

export type SidebarNavGroup = {
	id: string;
	label: string;
	icon: string;
	itemIds: SidebarItemId[];
	separatorBeforeItemIds: SidebarItemId[];
};

export type SidebarSettings = {
	schemaVersion: typeof SIDEBAR_SETTINGS_SCHEMA_VERSION;
	rootItemIds: SidebarRootItemId[];
	groups: SidebarNavGroup[];
	hiddenItemIds: SidebarItemId[];
};

export type ResolvedSidebarItem<TPrimaryItem extends { id: PrimaryNavId }> =
	| {
			kind: 'primary';
			itemId: SidebarPrimaryItemId;
			primary: TPrimaryItem;
	  }
	| {
			kind: 'communication';
			itemId: CommunicationSidebarItemId;
			section: CommunicationSection;
	  };

export type ResolvedSidebarRootEntry<TPrimaryItem extends { id: PrimaryNavId }> =
	| {
			kind: 'item';
			rootId: SidebarPrimaryItemId;
			item: ResolvedSidebarItem<TPrimaryItem>;
	  }
	| {
			kind: 'group';
			rootId: `group:${string}`;
			group: SidebarNavGroup & { items: Array<ResolvedSidebarItem<TPrimaryItem>> };
	  };

export type CommunicationNavGroup = 'overview' | 'workflow' | 'sources';

export type CommunicationSection = {
	id: CommunicationSectionId;
	label: string;
	icon: string;
	group: CommunicationNavGroup;
};

export const primaryWorkspaceNav: PrimaryWorkspaceNavItem[] = [
	{ id: 'home', label: 'Home', icon: 'tabler:home' },
	{ id: 'communications', label: 'Communications', icon: 'tabler:messages' },
	{ id: 'timeline', label: 'Timeline', icon: 'tabler:timeline-event' },
	{ id: 'persons', label: 'Persons', icon: 'tabler:user' },
	{ id: 'projects', label: 'Projects', icon: 'tabler:briefcase' },
	{ id: 'tasks', label: 'Tasks', icon: 'tabler:checkbox' },
	{ id: 'calendar', label: 'Calendar', icon: 'tabler:calendar' },
	{ id: 'documents', label: 'Documents', icon: 'tabler:file-text' },
	{ id: 'notes', label: 'Notes', icon: 'tabler:notes' },
	{ id: 'knowledge', label: 'Knowledge Graph', icon: 'tabler:share' },
	{ id: 'agents', label: 'AI Agents', icon: 'tabler:sparkles' }
];

export const communicationSections: CommunicationSection[] = [
	{ id: 'unified', label: 'Unified', icon: 'tabler:sparkles', group: 'overview' },
	{ id: 'inbox', label: 'Inbox', icon: 'tabler:mail', group: 'workflow' },
	{ id: 'waiting', label: 'Waiting', icon: 'tabler:clock-hour-4', group: 'workflow' },
	{ id: 'needs_reply', label: 'Needs Reply', icon: 'tabler:message-reply', group: 'workflow' },
	{ id: 'mentions', label: 'Mentions', icon: 'tabler:at', group: 'workflow' },
	{ id: 'mail', label: 'Mail', icon: 'tabler:mail', group: 'sources' },
	{ id: 'telegram', label: 'Telegram', icon: 'tabler:brand-telegram', group: 'sources' },
	{ id: 'whatsapp', label: 'WhatsApp', icon: 'tabler:brand-whatsapp', group: 'sources' },
	{ id: 'calls', label: 'Calls', icon: 'tabler:phone', group: 'sources' },
	{ id: 'meetings', label: 'Meetings', icon: 'tabler:calendar-event', group: 'sources' }
];

const sidebarPrimaryItemIds = primaryWorkspaceNav
	.map((item) => item.id)
	.filter((itemId): itemId is SidebarPrimaryItemId => itemId !== 'communications');
const sidebarPrimaryItemIdSet = new Set<SidebarPrimaryItemId>(sidebarPrimaryItemIds);
const communicationSectionIdSet = new Set<CommunicationSectionId>(
	communicationSections.map((section) => section.id)
);
const sidebarItemIdSet = new Set<SidebarItemId>([
	...sidebarPrimaryItemIds,
	...communicationSections.map((section) => communicationSidebarItemId(section.id))
]);

const defaultCommunicationsGroup: SidebarNavGroup = {
	id: 'communications',
	label: 'Communications',
	icon: 'tabler:messages',
	itemIds: communicationSections.map((section) => communicationSidebarItemId(section.id)),
	separatorBeforeItemIds: [communicationSidebarItemId('mail')]
};

const defaultRootItemIds: SidebarRootItemId[] = primaryWorkspaceNav.flatMap((item) =>
	item.id === 'communications' ? [sidebarGroupRootId(defaultCommunicationsGroup.id)] : [item.id as SidebarPrimaryItemId]
);

export function communicationSectionViewId(sectionId: CommunicationSectionId): SidebarViewId {
	if (sectionId === 'telegram' || sectionId === 'whatsapp') {
		return sectionId;
	}

	return 'communications';
}

export function communicationSidebarItemId(sectionId: CommunicationSectionId): CommunicationSidebarItemId {
	return `communications.${sectionId}`;
}

export function parseCommunicationSidebarItemId(itemId: SidebarItemId): CommunicationSectionId | null {
	if (!itemId.startsWith('communications.')) {
		return null;
	}

	const sectionId = itemId.slice('communications.'.length);
	return communicationSectionIdSet.has(sectionId as CommunicationSectionId)
		? (sectionId as CommunicationSectionId)
		: null;
}

export function sidebarGroupRootId(groupId: string): `group:${string}` {
	return `group:${normalizeGroupId(groupId)}`;
}

export function sidebarGroupIdFromRootId(rootId: SidebarRootItemId): string | null {
	return rootId.startsWith('group:') ? rootId.slice('group:'.length) : null;
}

export function defaultSidebarSettings(): SidebarSettings {
	return {
		schemaVersion: SIDEBAR_SETTINGS_SCHEMA_VERSION,
		rootItemIds: [...defaultRootItemIds],
		groups: [cloneSidebarGroup(defaultCommunicationsGroup)],
		hiddenItemIds: []
	};
}

export function parseSidebarSettings(value: unknown): SidebarSettings {
	if (!isRecord(value) || value.schemaVersion !== SIDEBAR_SETTINGS_SCHEMA_VERSION) {
		return defaultSidebarSettings();
	}

	const groups = parseSidebarGroups(value.groups);
	const rootItemIds = parseSidebarRootItemIds(value.rootItemIds);
	if (groups.length === 0 || rootItemIds.length === 0) {
		return defaultSidebarSettings();
	}

	return normalizeSidebarSettings({
		schemaVersion: SIDEBAR_SETTINGS_SCHEMA_VERSION,
		rootItemIds,
		groups,
		hiddenItemIds: parseSidebarItemIdArray(value.hiddenItemIds)
	});
}

export function resolveSidebarRootEntries<TPrimaryItem extends { id: PrimaryNavId }>(
	primaryItems: TPrimaryItem[],
	settings: SidebarSettings
): Array<ResolvedSidebarRootEntry<TPrimaryItem>> {
	const primaryItemById = new Map(primaryItems.map((item) => [item.id, item]));
	const communicationSectionById = new Map(communicationSections.map((section) => [section.id, section]));
	const groupById = new Map(settings.groups.map((group) => [group.id, group]));
	const hiddenIds = new Set(settings.hiddenItemIds);

	return settings.rootItemIds
		.map((rootId): ResolvedSidebarRootEntry<TPrimaryItem> | null => {
			const groupId = sidebarGroupIdFromRootId(rootId);
			if (groupId) {
				const group = groupById.get(groupId);
				if (!group) {
					return null;
				}

				const items = group.itemIds
					.filter((itemId) => !hiddenIds.has(itemId))
					.map((itemId) => resolveSidebarItem(itemId, primaryItemById, communicationSectionById))
					.filter((item): item is ResolvedSidebarItem<TPrimaryItem> => item !== null);

				return {
					kind: 'group',
					rootId: sidebarGroupRootId(group.id),
					group: {
						...group,
						items
					}
				};
			}

			const primaryRootId = rootId as SidebarPrimaryItemId;
			if (hiddenIds.has(primaryRootId)) {
				return null;
			}

			const item = resolveSidebarItem(primaryRootId, primaryItemById, communicationSectionById);
			return item
				? {
						kind: 'item',
						rootId: primaryRootId,
						item
					}
				: null;
		})
		.filter((entry): entry is ResolvedSidebarRootEntry<TPrimaryItem> => entry !== null);
}

export function visibleSidebarItemIds(settings: SidebarSettings): SidebarItemId[] {
	const hiddenIds = new Set(settings.hiddenItemIds);
	const rootItemIds = settings.rootItemIds.filter(
		(rootId): rootId is SidebarPrimaryItemId =>
			!rootId.startsWith('group:') && !hiddenIds.has(rootId as SidebarPrimaryItemId)
	);
	const groupItemIds = settings.groups.flatMap((group) =>
		group.itemIds.filter((itemId) => !hiddenIds.has(itemId))
	);
	return [...rootItemIds, ...groupItemIds];
}

function resolveSidebarItem<TPrimaryItem extends { id: PrimaryNavId }>(
	itemId: SidebarItemId,
	primaryItemById: Map<PrimaryNavId, TPrimaryItem>,
	communicationSectionById: Map<CommunicationSectionId, CommunicationSection>
): ResolvedSidebarItem<TPrimaryItem> | null {
	const communicationSectionId = parseCommunicationSidebarItemId(itemId);
	if (communicationSectionId) {
		const section = communicationSectionById.get(communicationSectionId);
		return section ? { kind: 'communication', itemId: itemId as CommunicationSidebarItemId, section } : null;
	}

	const primary = primaryItemById.get(itemId as SidebarPrimaryItemId);
	return primary ? { kind: 'primary', itemId: itemId as SidebarPrimaryItemId, primary } : null;
}

function parseSidebarGroups(value: unknown): SidebarNavGroup[] {
	if (!Array.isArray(value)) {
		return [];
	}

	return value
		.map((item, index) => parseSidebarGroup(item, index))
		.filter((item): item is SidebarNavGroup => item !== null);
}

function parseSidebarGroup(value: unknown, index: number): SidebarNavGroup | null {
	if (!isRecord(value)) {
		return null;
	}

	const rawId = typeof value.id === 'string' ? value.id.trim() : '';
	const itemIds = parseSidebarItemIdArray(value.itemIds);
	const label = typeof value.label === 'string' ? value.label.trim().slice(0, 32) : '';
	const icon = typeof value.icon === 'string' && value.icon.trim() ? value.icon.trim().slice(0, 48) : 'tabler:folder';
	const groupId = normalizeGroupId(rawId || label || `group-${index + 1}`);
	const separatorBeforeItemIds = Object.hasOwn(value, 'separatorBeforeItemIds')
		? parseSidebarItemIdArray(value.separatorBeforeItemIds)
		: groupId === 'communications'
			? [communicationSidebarItemId('mail')]
			: [];

	if (rawId.length === 0 && label.length === 0 && itemIds.length === 0) {
		return null;
	}

	return {
		id: groupId,
		label: label || `Group ${index + 1}`,
		icon,
		itemIds,
		separatorBeforeItemIds
	};
}

function parseSidebarRootItemIds(value: unknown): SidebarRootItemId[] {
	if (!Array.isArray(value)) {
		return [];
	}

	const result: SidebarRootItemId[] = [];
	for (const item of value) {
		if (typeof item !== 'string') {
			continue;
		}
		const normalizedGroupId = sidebarGroupIdFromRootId(item as SidebarRootItemId);
		if (normalizedGroupId) {
			const rootId = sidebarGroupRootId(normalizedGroupId);
			if (!result.includes(rootId)) {
				result.push(rootId);
			}
			continue;
		}
		if (sidebarPrimaryItemIdSet.has(item as SidebarPrimaryItemId)) {
			const itemId = item as SidebarPrimaryItemId;
			if (!result.includes(itemId)) {
				result.push(itemId);
			}
		}
	}
	return result;
}

function normalizeSidebarSettings(settings: SidebarSettings): SidebarSettings {
	const seenGroupIds = new Set<string>();
	const seenItemIds = new Set<SidebarItemId>();
	let groups = settings.groups.map((group, index) => {
		const id = uniqueGroupId(group.id, index, seenGroupIds);
		const itemIds = group.itemIds.filter((itemId) => {
			if (seenItemIds.has(itemId)) {
				return false;
			}
			seenItemIds.add(itemId);
			return true;
		});

		return {
			id,
			label: group.label,
			icon: group.icon,
			itemIds,
			separatorBeforeItemIds: group.separatorBeforeItemIds.filter(
				(itemId) => itemIds.includes(itemId) && itemIds.indexOf(itemId) > 0
			)
		};
	});

	const groupIds = new Set(groups.map((group) => group.id));
	let rootItemIds = settings.rootItemIds.filter((rootId) => {
		const groupId = sidebarGroupIdFromRootId(rootId);
		if (groupId) {
			return groupIds.has(groupId);
		}
		const primaryRootId = rootId as SidebarPrimaryItemId;
		if (seenItemIds.has(primaryRootId)) {
			return false;
		}
		seenItemIds.add(primaryRootId);
		return true;
	});

	for (const group of groups) {
		const rootId = sidebarGroupRootId(group.id);
		if (!rootItemIds.includes(rootId)) {
			rootItemIds.push(rootId);
		}
	}

	for (const itemId of sidebarPrimaryItemIds) {
		if (!seenItemIds.has(itemId)) {
			rootItemIds.push(itemId);
			seenItemIds.add(itemId);
		}
	}

	const communicationsGroup =
		groups.find((group) => group.id === defaultCommunicationsGroup.id) ??
		cloneSidebarGroup(defaultCommunicationsGroup);
	const existingCommunicationIds = new Set(
		groups.flatMap((group) =>
			group.itemIds.filter((itemId) => itemId.startsWith('communications.'))
		)
	);
	const missingCommunicationIds = defaultCommunicationsGroup.itemIds.filter(
		(itemId) => !existingCommunicationIds.has(itemId)
	);

	if (missingCommunicationIds.length > 0) {
		communicationsGroup.itemIds = [...communicationsGroup.itemIds, ...missingCommunicationIds];
		if (!groups.some((group) => group.id === communicationsGroup.id)) {
			groups = [communicationsGroup, ...groups];
		}
		const rootId = sidebarGroupRootId(communicationsGroup.id);
		if (!rootItemIds.includes(rootId)) {
			rootItemIds = [
				...rootItemIds.slice(0, 1),
				rootId,
				...rootItemIds.slice(1)
			];
		}
	}

	return {
		schemaVersion: SIDEBAR_SETTINGS_SCHEMA_VERSION,
		rootItemIds,
		groups,
		hiddenItemIds: settings.hiddenItemIds.filter((itemId) => sidebarItemIdSet.has(itemId))
	};
}

function parseSidebarItemIdArray(value: unknown): SidebarItemId[] {
	if (!Array.isArray(value)) {
		return [];
	}

	const result: SidebarItemId[] = [];
	for (const item of value) {
		if (typeof item !== 'string' || !sidebarItemIdSet.has(item as SidebarItemId)) {
			continue;
		}
		const itemId = item as SidebarItemId;
		if (!result.includes(itemId)) {
			result.push(itemId);
		}
	}
	return result;
}

function uniqueGroupId(rawId: string, index: number, seenGroupIds: Set<string>): string {
	const normalizedId = normalizeGroupId(rawId || `group-${index + 1}`);
	if (!seenGroupIds.has(normalizedId)) {
		seenGroupIds.add(normalizedId);
		return normalizedId;
	}

	let suffix = 2;
	while (seenGroupIds.has(`${normalizedId}-${suffix}`)) {
		suffix += 1;
	}
	const uniqueId = `${normalizedId}-${suffix}`;
	seenGroupIds.add(uniqueId);
	return uniqueId;
}

function normalizeGroupId(value: string): string {
	const normalized = value
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9_-]+/g, '-')
		.replace(/^-+|-+$/g, '');
	return normalized || 'group';
}

function cloneSidebarGroup(group: SidebarNavGroup): SidebarNavGroup {
	return {
		id: group.id,
		label: group.label,
		icon: group.icon,
		itemIds: [...group.itemIds],
		separatorBeforeItemIds: [...group.separatorBeforeItemIds]
	};
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}
