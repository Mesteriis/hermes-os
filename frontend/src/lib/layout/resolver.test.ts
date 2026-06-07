import { describe, expect, it } from 'vitest';
import { findPresetForView, layoutPresets, layoutViewIdForAppView } from './presets';
import { widgetRegistry } from './registry';
import { resolveLayout } from './resolver';
import { defaultLayoutSettings, parseLayoutSettings } from './settings';
import { LAYOUT_SCHEMA_VERSION } from './types';
import type { LayoutViewId } from './types';
import type { LayoutPreset, ViewLayoutOverride, WidgetDefinition } from './types';

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

const testWidgets: WidgetDefinition[] = [
	{
		id: 'home-whats-new',
		title: "What's New",
		viewScope: ['home'],
		defaultZone: 'main',
		allowedZones: ['main', 'rail'],
		minSize: { width: 260, height: 160 },
		defaultSizeIntent: 'auto',
		priority: 10,
		canHide: true,
		canAdd: true,
		dataMode: 'static'
	},
	{
		id: 'home-priorities',
		title: "Today's Priorities",
		viewScope: ['home'],
		defaultZone: 'main',
		allowedZones: ['main'],
		minSize: { width: 260, height: 160 },
		defaultSizeIntent: 'auto',
		priority: 20,
		canHide: true,
		canAdd: true,
		dataMode: 'static'
	},
	{
		id: 'home-later',
		title: 'Later',
		viewScope: ['home'],
		defaultZone: 'main',
		allowedZones: ['main'],
		minSize: { width: 260, height: 160 },
		defaultSizeIntent: 'auto',
		priority: 30,
		canHide: true,
		canAdd: true,
		dataMode: 'static'
	},
	{
		id: 'home-extra',
		title: 'Extra',
		viewScope: ['home'],
		defaultZone: 'main',
		allowedZones: ['main'],
		minSize: { width: 260, height: 160 },
		defaultSizeIntent: 'auto',
		priority: 40,
		canHide: true,
		canAdd: true,
		dataMode: 'static'
	}
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
	widgets: [
		{
			widgetId: 'home-whats-new',
			zoneId: 'main',
			order: 2,
			sizeIntent: 'auto',
			highlight: 'none',
			visible: true
		},
		{
			widgetId: 'home-priorities',
			zoneId: 'main',
			order: 1,
			sizeIntent: 'auto',
			highlight: 'none',
			visible: true
		}
	]
};

const orderedPreset: LayoutPreset = {
	...testPreset,
	widgets: [
		{
			widgetId: 'home-whats-new',
			zoneId: 'main',
			order: 3,
			sizeIntent: 'auto',
			highlight: 'none',
			visible: true
		},
		{
			widgetId: 'home-priorities',
			zoneId: 'main',
			order: 1,
			sizeIntent: 'auto',
			highlight: 'none',
			visible: true
		},
		{
			widgetId: 'home-later',
			zoneId: 'main',
			order: 2,
			sizeIntent: 'auto',
			highlight: 'none',
			visible: true
		},
		{
			widgetId: 'home-extra',
			zoneId: 'main',
			order: 4,
			sizeIntent: 'auto',
			highlight: 'none',
			visible: true
		}
	]
};

describe('layout domain exports', () => {
	it('uses schema version 1 for the first persisted layout setting', () => {
		expect(LAYOUT_SCHEMA_VERSION).toBe(1);
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
			schemaVersion: 1,
			views: {
				home: {
					presetId: 'home-default',
					presetVersion: 1,
					hiddenWidgetIds: ['home-system-status'],
					zoneOverrides: { 'home-whats-new': 'rail' },
					orderOverrides: { main: ['home-priorities', 'home-whats-new'] },
					sizeIntentOverrides: { 'home-whats-new': 'wide' }
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
		expect(parsed.views.home?.sizeIntentOverrides).toEqual({ 'home-whats-new': 'wide' });
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
			sizeIntentOverrides: { 'home-whats-new': 'wide' }
		});

		expect(resolved.widgetsByZone.main).toEqual([]);
		expect(resolved.widgetsByZone.rail.map((widget) => [widget.widgetId, widget.sizeIntent])).toEqual([
			['home-whats-new', 'wide']
		]);
		expect(resolved.hiddenByUser.map((widget) => widget.widgetId)).toEqual(['home-priorities']);
	});

	it('applies order overrides before remaining widgets sorted by preset order', () => {
		const resolved = resolveLayout(orderedPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: [],
			zoneOverrides: {},
			orderOverrides: { main: ['home-whats-new', 'home-later'] },
			sizeIntentOverrides: {}
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
			sizeIntentOverrides: {}
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
					{
						widgetId: 'home-missing',
						zoneId: 'main',
						order: 3,
						sizeIntent: 'auto',
						highlight: 'none',
						visible: true
					}
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
			sizeIntentOverrides: { 'home-whats-new': 'wide' }
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
