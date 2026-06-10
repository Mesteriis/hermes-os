import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ResolvedWidget } from '$lib/layout';
import { currentView } from './navigation';
import {
	addableWidgetsForCurrentView,
	activeLayout,
	cancelLayoutEditing,
	hideWidget,
	isLayoutEditing,
	layoutDraft,
	layoutSettings,
	setLayoutSettings,
	showWidget,
	startLayoutEditing,
	syncWidgetGridClasses,
	visibleWidgetIds,
	viewportHiddenWidgetTitles
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
		expect(get(addableWidgetsForCurrentView).map((widget) => widget.id)).toEqual(['home-priorities']);

		startLayoutEditing();

		expect(get(isLayoutEditing)).toBe(true);
		expect(get(layoutDraft)).toEqual(get(layoutSettings));

		cancelLayoutEditing();

		expect(get(isLayoutEditing)).toBe(false);
		expect(get(layoutDraft)).toBeNull();
	});

	it('updates visible widget ids and addable widgets from layout draft changes', () => {
		startLayoutEditing();

		expect(get(visibleWidgetIds).has('home-metrics')).toBe(true);

		hideWidget('home-metrics');

		expect(get(visibleWidgetIds).has('home-metrics')).toBe(false);
		expect(get(addableWidgetsForCurrentView).map((widget) => widget.id)).toEqual(['home-metrics']);

		showWidget('home-metrics');

		expect(get(visibleWidgetIds).has('home-metrics')).toBe(true);
		expect(get(addableWidgetsForCurrentView)).toEqual([]);
	});

	it('does not auto-hide widget frames while constructor mode is active', () => {
		const widgetElement = createFakeWidgetElement('home-metrics', ['widget-fit-hidden']);
		const workspaceElement = {
			clientWidth: 1024,
			getBoundingClientRect: () => ({
				bottom: 720,
				height: 720,
				left: 0,
				right: 1024,
				top: 0,
				width: 1024
			})
		};
		const requestAnimationFrame = vi.fn();

		vi.stubGlobal('document', {
			body: workspaceElement,
			querySelector: (selector: string) => {
				if (selector === '.workspace') return workspaceElement;
				return null;
			},
			querySelectorAll: (selector: string) => {
				if (selector === '.widget-frame[data-widget-id]') return [widgetElement];
				return [];
			}
		});
		vi.stubGlobal('window', {
			innerHeight: 720,
			innerWidth: 1024,
			scrollTo: vi.fn(),
			scrollX: 0,
			scrollY: 0
		});
		vi.stubGlobal('requestAnimationFrame', requestAnimationFrame);

		startLayoutEditing();
		viewportHiddenWidgetTitles.set(['Metrics']);

		syncWidgetGridClasses(
			new Map([
				[
					'home-metrics',
					{
						columns: 12,
						definition: { title: 'Metrics' },
						minColumns: 1,
						minRows: 2,
						rows: 3,
						scrollMode: 'none',
						widgetId: 'home-metrics'
					} as ResolvedWidget
				]
			])
		);

		expect(widgetElement.classList.contains('widget-fit-hidden')).toBe(false);
		expect(widgetElement.classList.contains('widget-cols-12')).toBe(true);
		expect(get(viewportHiddenWidgetTitles)).toEqual([]);
		expect(requestAnimationFrame).not.toHaveBeenCalled();
	});
});

afterEach(() => {
	vi.unstubAllGlobals();
});

function createFakeWidgetElement(widgetId: string, initialClasses: string[]) {
	const classList = new FakeClassList(initialClasses);
	return {
		classList,
		dataset: { widgetId },
		getBoundingClientRect: () => ({
			bottom: 120,
			height: 120,
			left: 0,
			right: 1024,
			top: 0,
			width: 1024
		}),
		querySelectorAll: () => []
	};
}

class FakeClassList {
	private readonly classes: Set<string>;

	constructor(initialClasses: string[]) {
		this.classes = new Set(initialClasses);
	}

	add(...classNames: string[]): void {
		for (const className of classNames) {
			this.classes.add(className);
		}
	}

	contains(className: string): boolean {
		return this.classes.has(className);
	}

	remove(...classNames: string[]): void {
		for (const className of classNames) {
			this.classes.delete(className);
		}
	}
}
