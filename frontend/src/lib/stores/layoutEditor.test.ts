import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import { currentView } from './navigation';
import {
	activeLayout,
	cancelLayoutEditing,
	isLayoutEditing,
	layoutDraft,
	layoutSettings,
	setLayoutSettings,
	startLayoutEditing
} from './layoutEditor';

describe('layout editor store', () => {
	beforeEach(() => {
		currentView.set('home');
		cancelLayoutEditing();
		setLayoutSettings({ schemaVersion: 2, views: {} });
	});

	it('resolves active layout from persisted settings and owns constructor mode state', () => {
		setLayoutSettings({
			schemaVersion: 2,
			views: {
				home: {
					presetId: 'home-default',
					presetVersion: 1,
					hiddenWidgetIds: ['home-priorities'],
					zoneOverrides: {},
					orderOverrides: {},
					gridOverrides: {}
				}
			}
		});

		expect(get(activeLayout)?.hiddenByUser.map((widget) => widget.widgetId)).toEqual([
			'home-priorities'
		]);

		startLayoutEditing();

		expect(get(isLayoutEditing)).toBe(true);
		expect(get(layoutDraft)).toEqual(get(layoutSettings));

		cancelLayoutEditing();

		expect(get(isLayoutEditing)).toBe(false);
		expect(get(layoutDraft)).toBeNull();
	});
});
