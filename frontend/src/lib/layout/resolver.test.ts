import { describe, expect, it } from 'vitest';
import { resolveLayout } from './resolver';
import { defaultLayoutSettings, parseLayoutSettings } from './settings';
import { LAYOUT_SCHEMA_VERSION } from './types';
import type { LayoutPreset, WidgetDefinition } from './types';

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

describe('layout domain exports', () => {
	it('uses schema version 1 for the first persisted layout setting', () => {
		expect(LAYOUT_SCHEMA_VERSION).toBe(1);
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

	it('applies hidden, zone, order and size overrides', () => {
		const resolved = resolveLayout(testPreset, testWidgets, {
			presetId: 'home-default',
			presetVersion: 1,
			hiddenWidgetIds: ['home-priorities'],
			zoneOverrides: { 'home-whats-new': 'rail' },
			orderOverrides: { rail: ['home-whats-new'] },
			sizeIntentOverrides: { 'home-whats-new': 'wide' }
		});

		expect(resolved.widgetsByZone.main).toEqual([]);
		expect(resolved.widgetsByZone.rail.map((widget) => [widget.widgetId, widget.sizeIntent])).toEqual([
			['home-whats-new', 'wide']
		]);
		expect(resolved.hiddenByUser.map((widget) => widget.widgetId)).toEqual(['home-priorities']);
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
});
