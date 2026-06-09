import { describe, expect, it } from 'vitest';
import { findPresetForView, layoutPresets, layoutViewIdForAppView } from './presets';
import { widgetRegistry } from './registry';
import { resolveLayout } from './resolver';
import { defaultLayoutSettings, parseLayoutSettings } from './settings';
import { LAYOUT_GRID_COLUMNS, LAYOUT_GRID_MAX_ROWS, LAYOUT_SCHEMA_VERSION } from './types';
import type { LayoutViewId } from './types';
import type { LayoutPreset, LayoutWidgetInstance, ViewLayoutOverride, WidgetDefinition } from './types';

const expectedViews: LayoutViewId[] = [
	'home',
	'communications',
	'timeline',
	'persons',
	'projects',
	'tasks',
	'calendar',
	'documents',
	'notes',
	'knowledge-graph',
	'telegram',
	'whatsapp',
	'ai-agents',
	'organizations',
	'settings'
];

function testDefinition(
	id: string,
	title: string,
	allowedZones: string[],
	priority: number,
	defaultScrollMode: WidgetDefinition['defaultScrollMode'] = 'none'
): WidgetDefinition {
	return {
		id,
		title,
		viewScope: ['home'],
		defaultZone: 'main',
		allowedZones,
		minColumns: 2,
		minRows: 2,
		defaultScrollMode,
		priority,
		canHide: true,
		canAdd: true,
		dataMode: 'static'
	};
}

function testInstance(widgetId: string, zoneId: string, order: number): LayoutWidgetInstance {
	return {
		widgetId,
		zoneId,
		order,
		columns: 4,
		rows: 4,
		highlight: 'none',
		visible: true
	};
}

const testWidgets: WidgetDefinition[] = [
	testDefinition('home-whats-new', "What's New", ['main', 'rail'], 10, 'vertical'),
	testDefinition('home-priorities', "Today's Priorities", ['main'], 20),
	testDefinition('home-later', 'Later', ['main'], 30),
	testDefinition('home-extra', 'Extra', ['main'], 40)
];

const testPreset: LayoutPreset = {
	id: 'home-default',
	version: 1,
	viewId: 'home',
	archetype: 'operational_board',
	zones: [
		{ id: 'main', title: 'Main', minWidth: 320, minHeight: 240 },
		{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 }
	],
	widgets: [testInstance('home-whats-new', 'main', 2), testInstance('home-priorities', 'main', 1)]
};

const orderedPreset: LayoutPreset = {
	...testPreset,
	widgets: [
		testInstance('home-whats-new', 'main', 3),
		testInstance('home-priorities', 'main', 1),
		testInstance('home-later', 'main', 2),
		testInstance('home-extra', 'main', 4)
	]
};

describe('layout domain exports', () => {
	it('uses schema version 2 for row and column widget layout settings', () => {
		expect(LAYOUT_SCHEMA_VERSION).toBe(2);
		expect(LAYOUT_GRID_COLUMNS).toBe(12);
		expect(LAYOUT_GRID_MAX_ROWS).toBe(24);
	});
});

describe('default widget inventory', () => {
	it('declares one preset for every current view', () => {
		expect(layoutPresets.map((preset) => preset.viewId).sort()).toEqual([...expectedViews].sort());
	});

	it('does not declare duplicate widget definitions', () => {
		const seen = new Set<string>();
		const duplicates = widgetRegistry
			.map((definition) => definition.id)
			.filter((widgetId) => {
				if (seen.has(widgetId)) {
					return true;
				}

				seen.add(widgetId);
				return false;
			});

		expect(duplicates).toEqual([]);
	});

	it('does not declare duplicate widget instances inside each preset', () => {
		const duplicates = layoutPresets.flatMap((preset) => {
			const seen = new Set<string>();
			return preset.widgets
				.map((widget) => widget.widgetId)
				.filter((widgetId) => {
					if (seen.has(widgetId)) {
						return true;
					}

					seen.add(widgetId);
					return false;
				})
				.map((widgetId) => `${preset.viewId}:${widgetId}`);
		});

		expect(duplicates).toEqual([]);
	});

	it('has a widget definition for every preset instance', () => {
		const widgetIds = new Set(widgetRegistry.map((definition) => definition.id));
		const missing = layoutPresets.flatMap((preset) =>
			preset.widgets
				.filter((widget) => !widgetIds.has(widget.widgetId))
				.map((widget) => `${preset.viewId}:${widget.widgetId}`)
		);

		expect(missing).toEqual([]);
	});

	it('keeps every preset instance inside a declared preset zone', () => {
		const invalidZones = layoutPresets.flatMap((preset) => {
			const zoneIds = new Set(preset.zones.map((zone) => zone.id));
			return preset.widgets
				.filter((widget) => !zoneIds.has(widget.zoneId))
				.map((widget) => `${preset.viewId}:${widget.widgetId}:${widget.zoneId}`);
		});

		expect(invalidZones).toEqual([]);
	});

	it('keeps all visible default widgets inside allowed zones', () => {
		const widgetsById = new Map(widgetRegistry.map((definition) => [definition.id, definition]));
		const invalidZones = layoutPresets.flatMap((preset) =>
			preset.widgets
				.filter((widget) => {
					if (!widget.visible) {
						return false;
					}

					const definition = widgetsById.get(widget.widgetId);
					return !definition || !definition.allowedZones.includes(widget.zoneId);
				})
				.map((widget) => `${preset.viewId}:${widget.widgetId}:${widget.zoneId}`)
		);

		expect(invalidZones).toEqual([]);
	});

	it('keeps widget definitions scoped to the presets that use them', () => {
		const widgetsById = new Map(widgetRegistry.map((definition) => [definition.id, definition]));
		const invalidScopes = layoutPresets.flatMap((preset) =>
			preset.widgets
				.filter((widget) => {
					const definition = widgetsById.get(widget.widgetId);
					return !definition || !definition.viewScope.includes(preset.viewId);
				})
				.map((widget) => `${preset.viewId}:${widget.widgetId}`)
		);

		expect(invalidScopes).toEqual([]);
	});
});

describe('app shell layout view aliases', () => {
	it('maps current app shell view ids to layout domain view ids', () => {
		expect(layoutViewIdForAppView('knowledge')).toBe('knowledge-graph');
		expect(layoutViewIdForAppView('agents')).toBe('ai-agents');
		expect(layoutViewIdForAppView('settings')).toBe('settings');
		expect(layoutViewIdForAppView('unknown')).toBeNull();
	});

	it('finds layout presets from app shell aliases', () => {
		expect(findPresetForView('knowledge')?.viewId).toBe('knowledge-graph');
		expect(findPresetForView('agents')?.viewId).toBe('ai-agents');
	});
});

describe('layout settings parser', () => {
	it('returns defaults for missing or invalid values', () => {
		expect(parseLayoutSettings(null)).toEqual(defaultLayoutSettings());
		expect(parseLayoutSettings({ schemaVersion: 99, views: {} })).toEqual(defaultLayoutSettings());
		expect(parseLayoutSettings('bad')).toEqual(defaultLayoutSettings());
	});

	it('keeps valid home view overrides', () => {
		const parsed = parseLayoutSettings({
			schemaVersion: 2,
			views: {
				home: {
					presetId: 'home-default',
					presetVersion: 1,
					hiddenWidgetIds: ['home-system-status'],
					zoneOverrides: { 'home-whats-new': 'rail' },
					orderOverrides: { main: ['home-priorities', 'home-whats-new'] },
					gridOverrides: {
						'home-whats-new': { columns: 6, rows: 8, scrollMode: 'vertical' }
					}
				}
			}
		});

		expect(parsed.views.home?.presetId).toBe('home-default');
		expect(parsed.views.home?.presetVersion).toBe(1);
		expect(parsed.views.home?.hiddenWidgetIds).toEqual(['home-system-status']);
		expect(parsed.views.home?.zoneOverrides).toEqual({ 'home-whats-new': 'rail' });
		expect(parsed.views.home?.orderOverrides).toEqual({
			main: ['home-priorities', 'home-whats-new']
		});
		expect(parsed.views.home?.gridOverrides).toEqual({
			'home-whats-new': { columns: 6, rows: 8, scrollMode: 'vertical' }
		});
	});

	it('migrates legacy schema v1 overrides while dropping grid override state', () => {
		const parsed = parseLayoutSettings({
			schemaVersion: 1,
			views: {
				home: {
					presetId: 'home-default',
					presetVersion: 1,
					hiddenWidgetIds: ['home-system-status'],
					zoneOverrides: { 'home-whats-new': 'rail' },
					orderOverrides: { main: ['home-priorities', 'home-whats-new'] }
				}
			}
		});

		expect(parsed.schemaVersion).toBe(2);
		expect(parsed.views.home?.hiddenWidgetIds).toEqual(['home-system-status']);
		expect(parsed.views.home?.zoneOverrides).toEqual({ 'home-whats-new': 'rail' });
		expect(parsed.views.home?.gridOverrides).toEqual({});
	});

	it('keeps organizations view overrides (regression: parser allow-list dropped them)', () => {
		const parsed = parseLayoutSettings({
			schemaVersion: 2,
			views: {
				organizations: {
					presetId: 'organizations-default',
					presetVersion: 1,
					hiddenWidgetIds: ['organizations-health'],
					zoneOverrides: {},
					orderOverrides: {},
					gridOverrides: {}
				}
			}
		});

		expect(parsed.views.organizations?.presetId).toBe('organizations-default');
		expect(parsed.views.organizations?.hiddenWidgetIds).toEqual(['organizations-health']);
	});
});

describe('resolveLayout', () => {
	it('sorts widgets by preset order when there are no overrides', () => {
		const resolved = resolveLayout(testPreset, testWidgets, undefined);

		expect(resolved.widgetsByZone.main.map((widget) => widget.widgetId)).toEqual([
			'home-priorities',
			'home-whats-new'
		]);
	});

	it('applies hidden, zone and size overrides', () => {
		const resolved = resolveLayout(testPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: ['home-priorities'],
			zoneOverrides: { 'home-whats-new': 'rail' },
			orderOverrides: {},
			gridOverrides: {
				'home-whats-new': { columns: 6, rows: 8, scrollMode: 'vertical' }
			}
		});

		expect(resolved.widgetsByZone.main).toEqual([]);
		expect(
			resolved.widgetsByZone.rail.map((widget) => [
				widget.widgetId,
				widget.columns,
				widget.rows,
				widget.minColumns,
				widget.minRows,
				widget.scrollMode
			])
		).toEqual([['home-whats-new', 6, 8, 2, 2, 'vertical']]);
		expect(resolved.hiddenByUser.map((widget) => widget.widgetId)).toEqual(['home-priorities']);
	});

	it('clamps illegal grid overrides to widget minimums and the 12-column / 24-row layout bounds', () => {
		const resolved = resolveLayout(testPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: [],
			zoneOverrides: {},
			orderOverrides: {},
			gridOverrides: {
				'home-whats-new': { columns: 99, rows: 99, scrollMode: 'vertical' },
				'home-priorities': { columns: 1, rows: 1, scrollMode: 'both' }
			}
		});

		const widgets = new Map(resolved.widgetsByZone.main.map((widget) => [widget.widgetId, widget]));
		expect(widgets.get('home-whats-new')).toMatchObject({
			columns: 12,
			rows: 24,
			scrollMode: 'vertical'
		});
		expect(widgets.get('home-priorities')).toMatchObject({
			columns: 2,
			rows: 2,
			scrollMode: 'both'
		});
	});

	it('falls back to definition scroll defaults when the instance stays automatic', () => {
		const resolved = resolveLayout(testPreset, testWidgets, undefined);

		const widgets = new Map(resolved.widgetsByZone.main.map((widget) => [widget.widgetId, widget]));
		expect(widgets.get('home-whats-new')?.scrollMode).toBe('vertical');
		expect(widgets.get('home-priorities')?.scrollMode).toBe('none');
	});

	it('applies order overrides before remaining widgets sorted by preset order', () => {
		const resolved = resolveLayout(orderedPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: [],
			zoneOverrides: {},
			orderOverrides: { main: ['home-whats-new', 'home-later'] },
			gridOverrides: {}
		});

		expect(resolved.widgetsByZone.main.map((widget) => widget.widgetId)).toEqual([
			'home-whats-new',
			'home-later',
			'home-priorities',
			'home-extra'
		]);
	});

	it('ignores illegal zone overrides', () => {
		const resolved = resolveLayout(testPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: [],
			zoneOverrides: { 'home-priorities': 'rail' },
			orderOverrides: {},
			gridOverrides: {}
		});

		expect(resolved.widgetsByZone.main.map((widget) => widget.widgetId)).toContain('home-priorities');
		expect(resolved.widgetsByZone.rail.map((widget) => widget.widgetId)).not.toContain(
			'home-priorities'
		);
	});

	it('reports preset widgets without matching definitions', () => {
		const resolved = resolveLayout(
			{
				...testPreset,
				widgets: [
					...testPreset.widgets,
					testInstance('home-missing', 'main', 3)
				]
			},
			testWidgets,
			undefined
		);

		expect(resolved.ignoredWidgetIds).toEqual(['home-missing']);
		expect(Object.values(resolved.widgetsByZone).flat().map((widget) => widget.widgetId)).not.toContain(
			'home-missing'
		);
	});

	it('does not mutate preset, definition or override inputs', () => {
		const override: ViewLayoutOverride = {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: ['home-priorities'],
			zoneOverrides: { 'home-whats-new': 'rail' },
			orderOverrides: { rail: ['home-whats-new'] },
			gridOverrides: { 'home-whats-new': { columns: 6, rows: 8, scrollMode: 'vertical' } }
		};
		const originalPreset = JSON.stringify(testPreset);
		const originalWidgets = JSON.stringify(testWidgets);
		const originalOverride = JSON.stringify(override);

		resolveLayout(testPreset, testWidgets, override);

		expect(JSON.stringify(testPreset)).toBe(originalPreset);
		expect(JSON.stringify(testWidgets)).toBe(originalWidgets);
		expect(JSON.stringify(override)).toBe(originalOverride);
	});
});
