import { describe, expect, it } from 'vitest';
import { defaultLayoutSettings, parseLayoutSettings } from './settings';
import { LAYOUT_SCHEMA_VERSION } from './types';

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
