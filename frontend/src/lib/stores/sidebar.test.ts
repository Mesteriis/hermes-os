import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import { defaultSidebarSettings } from '$lib/layout';
import {
	cancelSidebarSettingsEditing,
	effectiveSidebarSettings,
	hasSidebarChanges,
	setSidebarSettings,
	sidebarDraft,
	sidebarRootEntries,
	toggleSidebarItemHidden
} from './sidebar';

describe('sidebar store', () => {
	beforeEach(() => {
		setSidebarSettings(defaultSidebarSettings());
		cancelSidebarSettingsEditing();
	});

	it('drives resolved shell entries from persisted and draft sidebar settings', () => {
		const persisted = {
			...defaultSidebarSettings(),
			hiddenItemIds: ['tasks' as const]
		};

		setSidebarSettings(persisted);

		expect(get(effectiveSidebarSettings).hiddenItemIds).toEqual(['tasks']);
		expect(JSON.stringify(get(sidebarRootEntries))).not.toContain('"tasks"');

		toggleSidebarItemHidden('projects');

		expect(get(sidebarDraft)?.hiddenItemIds).toEqual(['tasks', 'projects']);
		expect(get(hasSidebarChanges)).toBe(true);
		expect(JSON.stringify(get(sidebarRootEntries))).not.toContain('"projects"');
	});
});
