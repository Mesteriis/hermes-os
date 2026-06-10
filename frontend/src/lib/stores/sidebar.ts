import { derived, get, writable } from 'svelte/store';
import {
	communicationSections,
	defaultSidebarSettings,
	parseCommunicationSidebarItemId,
	parseSidebarSettings,
	primaryWorkspaceNav,
	resolveSidebarRootEntries,
	sidebarGroupIdFromRootId,
	sidebarGroupRootId,
	type PrimaryNavId,
	type ResolvedSidebarItem,
	type SidebarItemId,
	type SidebarNavGroup,
	type SidebarRootItemId,
	type SidebarSettings
} from '$lib/layout';

export type ShellNavItem = {
	id: PrimaryNavId;
	label: string;
	icon: string;
	badge?: string;
	enabled: boolean;
};

export const sidebarSettings = writable<SidebarSettings>(defaultSidebarSettings());
export const sidebarDraft = writable<SidebarSettings | null>(null);
export const sidebarError = writable('');
export const newSidebarGroupLabel = writable('');
export const isSidebarSettingsSaving = writable(false);

export const primaryNavItems: ShellNavItem[] = primaryWorkspaceNav.map((item) => ({
	...item,
	enabled: true
}));

export const effectiveSidebarSettings = derived(
	[sidebarSettings, sidebarDraft],
	([$sidebarSettings, $sidebarDraft]) => $sidebarDraft ?? $sidebarSettings
);

export const sidebarRootEntries = derived(effectiveSidebarSettings, ($effectiveSidebarSettings) =>
	resolveSidebarRootEntries(primaryNavItems, $effectiveSidebarSettings)
);

export const sidebarHiddenNavItems = derived(effectiveSidebarSettings, ($effectiveSidebarSettings) =>
	$effectiveSidebarSettings.hiddenItemIds
		.map((itemId) => sidebarConfigItem(itemId))
		.filter((item): item is { id: SidebarItemId; label: string; icon: string } => item !== null)
);

export const hasSidebarChanges = derived(
	[sidebarSettings, sidebarDraft],
	([$sidebarSettings, $sidebarDraft]) =>
		$sidebarDraft !== null && JSON.stringify($sidebarDraft) !== JSON.stringify($sidebarSettings)
);

export function setSidebarSettings(settings: SidebarSettings): void {
	sidebarSettings.set(parseSidebarSettings(settings));
	sidebarDraft.set(null);
	sidebarError.set('');
}

export function updateSidebarDraft(update: (draft: SidebarSettings) => SidebarSettings): void {
	const draft = get(sidebarDraft) ?? cloneSidebarSettings(get(sidebarSettings));
	sidebarDraft.set(parseSidebarSettings(update(draft)));
	sidebarError.set('');
}

export function resetSidebarSettingsToDefault(): void {
	sidebarDraft.set(defaultSidebarSettings());
	sidebarError.set('');
}

export function cancelSidebarSettingsEditing(): void {
	sidebarDraft.set(null);
	sidebarError.set('');
	newSidebarGroupLabel.set('');
}

export function sidebarGroupLabel(group: SidebarNavGroup, index: number): string {
	return group.label || (group.id === 'communications' ? 'Communications' : `Group ${index + 1}`);
}

export function sidebarRootIndexForGroup(groupId: string): number {
	return get(effectiveSidebarSettings).rootItemIds.indexOf(sidebarGroupRootId(groupId));
}

export function sidebarConfigItem(itemId: SidebarItemId): { id: SidebarItemId; label: string; icon: string } | null {
	const communicationSectionId = parseCommunicationSidebarItemId(itemId);
	if (communicationSectionId) {
		const section = communicationSections.find((item) => item.id === communicationSectionId);
		return section ? { id: itemId, label: section.label, icon: section.icon } : null;
	}

	const item = primaryNavItems.find((navItem) => navItem.id === itemId);
	return item ? { id: itemId, label: item.label, icon: item.icon } : null;
}

export function sidebarItemLabel(item: ResolvedSidebarItem<ShellNavItem>): string {
	return item.kind === 'primary' ? item.primary.label : item.section.label;
}

export function sidebarGroupHasSeparatorBefore(group: SidebarNavGroup, itemId: SidebarItemId): boolean {
	return group.itemIds.indexOf(itemId) > 0 && group.separatorBeforeItemIds.includes(itemId);
}

export function sidebarGroupIdFromLabel(label: string): string {
	const settings = get(effectiveSidebarSettings);
	const base =
		label
			.trim()
			.toLowerCase()
			.replace(/[^a-z0-9_-]+/g, '-')
			.replace(/^-+|-+$/g, '') || `group-${settings.groups.length + 1}`;
	const existingIds = new Set(settings.groups.map((group) => group.id));
	if (!existingIds.has(base)) {
		return base;
	}

	let suffix = 2;
	while (existingIds.has(`${base}-${suffix}`)) {
		suffix += 1;
	}
	return `${base}-${suffix}`;
}

export function addSidebarGroup(): void {
	const settings = get(effectiveSidebarSettings);
	const label = get(newSidebarGroupLabel).trim().slice(0, 32);
	const groupLabel = label || `Group ${settings.groups.length + 1}`;
	const groupId = sidebarGroupIdFromLabel(groupLabel);
	updateSidebarDraft((draft) => ({
		...draft,
		rootItemIds: [...draft.rootItemIds, sidebarGroupRootId(groupId)],
		groups: [
			...draft.groups,
			{
				id: groupId,
				label: groupLabel,
				icon: 'tabler:folder',
				itemIds: [],
				separatorBeforeItemIds: []
			}
		]
	}));
	newSidebarGroupLabel.set('');
}

export function removeSidebarGroup(groupId: string): void {
	if (groupId === 'communications') {
		sidebarError.set('The Communications group can be renamed or reordered, but not removed.');
		return;
	}

	updateSidebarDraft((draft) => {
		if (draft.groups.length <= 1) {
			return draft;
		}

		const groupIndex = draft.groups.findIndex((group) => group.id === groupId);
		if (groupIndex < 0) {
			return draft;
		}

		const groups = draft.groups.map((group) => ({
			...group,
			itemIds: [...group.itemIds],
			separatorBeforeItemIds: [...group.separatorBeforeItemIds]
		}));
		const [removedGroup] = groups.splice(groupIndex, 1);
		if (!removedGroup) {
			return draft;
		}

		let rootItemIds = draft.rootItemIds.filter((rootId) => sidebarGroupIdFromRootId(rootId) !== groupId);
		const communicationsGroupIndex = groups.findIndex((group) => group.id === 'communications');
		for (const itemId of removedGroup.itemIds) {
			const communicationSectionId = parseCommunicationSidebarItemId(itemId);
			if (communicationSectionId && communicationsGroupIndex >= 0) {
				groups[communicationsGroupIndex] = {
					...groups[communicationsGroupIndex],
					itemIds: [...groups[communicationsGroupIndex].itemIds, itemId]
				};
			} else if (!communicationSectionId) {
				rootItemIds = [...rootItemIds, itemId as SidebarRootItemId];
			}
		}

		return { ...draft, rootItemIds, groups };
	});
}

export function moveSidebarGroup(groupId: string, direction: -1 | 1): void {
	updateSidebarDraft((draft) => {
		const rootId = sidebarGroupRootId(groupId);
		const rootIndex = draft.rootItemIds.indexOf(rootId);
		const nextIndex = rootIndex + direction;
		if (rootIndex < 0 || nextIndex < 0 || nextIndex >= draft.rootItemIds.length) {
			return draft;
		}

		const rootItemIds = [...draft.rootItemIds];
		[rootItemIds[rootIndex], rootItemIds[nextIndex]] = [rootItemIds[nextIndex], rootItemIds[rootIndex]];
		return { ...draft, rootItemIds };
	});
}

export function moveSidebarRootItem(rootId: SidebarRootItemId, direction: -1 | 1): void {
	updateSidebarDraft((draft) => {
		const rootIndex = draft.rootItemIds.indexOf(rootId);
		const nextIndex = rootIndex + direction;
		if (rootIndex < 0 || nextIndex < 0 || nextIndex >= draft.rootItemIds.length) {
			return draft;
		}
		const rootItemIds = [...draft.rootItemIds];
		[rootItemIds[rootIndex], rootItemIds[nextIndex]] = [rootItemIds[nextIndex], rootItemIds[rootIndex]];
		return { ...draft, rootItemIds };
	});
}

export function moveSidebarItem(itemId: SidebarItemId, direction: -1 | 1): void {
	updateSidebarDraft((draft) => {
		const groups = draft.groups.map((group) => ({
			...group,
			itemIds: [...group.itemIds],
			separatorBeforeItemIds: [...group.separatorBeforeItemIds]
		}));
		const groupIndex = groups.findIndex((group) => group.itemIds.includes(itemId));
		if (groupIndex < 0) {
			return draft;
		}

		const itemIndex = groups[groupIndex].itemIds.indexOf(itemId);
		const nextItemIndex = itemIndex + direction;
		if (nextItemIndex >= 0 && nextItemIndex < groups[groupIndex].itemIds.length) {
			[groups[groupIndex].itemIds[itemIndex], groups[groupIndex].itemIds[nextItemIndex]] = [
				groups[groupIndex].itemIds[nextItemIndex],
				groups[groupIndex].itemIds[itemIndex]
			];
			return { ...draft, groups };
		}

		const nextGroupIndex = groupIndex + direction;
		if (nextGroupIndex < 0 || nextGroupIndex >= groups.length) {
			return draft;
		}

		groups[groupIndex].itemIds = groups[groupIndex].itemIds.filter((id) => id !== itemId);
		groups[groupIndex].separatorBeforeItemIds = groups[groupIndex].separatorBeforeItemIds.filter(
			(id) => id !== itemId
		);
		if (direction < 0) {
			groups[nextGroupIndex].itemIds = [...groups[nextGroupIndex].itemIds, itemId];
		} else {
			groups[nextGroupIndex].itemIds = [itemId, ...groups[nextGroupIndex].itemIds];
		}
		return { ...draft, groups };
	});
}

export function moveSidebarItemToGroup(itemId: SidebarItemId, targetGroupId: string): void {
	updateSidebarDraft((draft) => {
		if (targetGroupId !== 'root' && !draft.groups.some((group) => group.id === targetGroupId)) {
			return draft;
		}

		const groups = draft.groups.map((group) => ({
			...group,
			itemIds: group.itemIds.filter((id) => id !== itemId),
			separatorBeforeItemIds: group.separatorBeforeItemIds.filter((id) => id !== itemId)
		}));
		const rootItemIds = draft.rootItemIds.filter((id) => id !== itemId);

		if (targetGroupId === 'root') {
			if (parseCommunicationSidebarItemId(itemId)) {
				return draft;
			}
			return {
				...draft,
				rootItemIds: [...rootItemIds, itemId as SidebarRootItemId],
				groups
			};
		}

		return {
			...draft,
			rootItemIds,
			groups: groups.map((group) =>
				group.id === targetGroupId ? { ...group, itemIds: [...group.itemIds, itemId] } : group
			)
		};
	});
}

export function toggleSidebarGroupSeparator(groupId: string, itemId: SidebarItemId): void {
	updateSidebarDraft((draft) => ({
		...draft,
		groups: draft.groups.map((group) => {
			if (group.id !== groupId || group.itemIds.indexOf(itemId) <= 0) {
				return group;
			}

			const hasSeparator = group.separatorBeforeItemIds.includes(itemId);
			return {
				...group,
				separatorBeforeItemIds: hasSeparator
					? group.separatorBeforeItemIds.filter((id) => id !== itemId)
					: [...group.separatorBeforeItemIds, itemId]
			};
		})
	}));
}

export function toggleSidebarItemHidden(itemId: SidebarItemId): void {
	updateSidebarDraft((draft) => ({
		...draft,
		hiddenItemIds: draft.hiddenItemIds.includes(itemId)
			? draft.hiddenItemIds.filter((id) => id !== itemId)
			: [...draft.hiddenItemIds, itemId]
	}));
}

export function updateSidebarGroupLabel(groupId: string, label: string): void {
	updateSidebarDraft((draft) => ({
		...draft,
		groups: draft.groups.map((group) =>
			group.id === groupId ? { ...group, label: label.slice(0, 32) } : group
		)
	}));
}

function cloneSidebarSettings(settings: SidebarSettings): SidebarSettings {
	return structuredClone(settings);
}
