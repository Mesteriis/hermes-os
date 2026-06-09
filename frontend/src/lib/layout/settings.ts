import {
	LEGACY_LAYOUT_SCHEMA_VERSION,
	LAYOUT_SCHEMA_VERSION,
	layoutViewIds,
	type WidgetGridOverride,
	type WidgetScrollMode,
	type LayoutSettings,
	type LayoutViewId,
	type ViewLayoutOverride,
	widgetScrollModes,
} from './types';

const layoutViewIdSet = new Set<string>(layoutViewIds);
const widgetScrollModeSet = new Set<string>(widgetScrollModes);

export function defaultLayoutSettings(): LayoutSettings {
	return {
		schemaVersion: LAYOUT_SCHEMA_VERSION,
		views: {}
	};
}

export function parseLayoutSettings(value: unknown): LayoutSettings {
	if (
		!isRecord(value) ||
		(value.schemaVersion !== LAYOUT_SCHEMA_VERSION &&
			value.schemaVersion !== LEGACY_LAYOUT_SCHEMA_VERSION) ||
		!isRecord(value.views)
	) {
		return defaultLayoutSettings();
	}

	const isLegacySchema = value.schemaVersion === LEGACY_LAYOUT_SCHEMA_VERSION;
	const views: LayoutSettings['views'] = {};
	for (const [viewId, viewOverride] of Object.entries(value.views)) {
		if (!isLayoutViewId(viewId)) {
			continue;
		}

		const parsedOverride = parseViewOverride(viewOverride, isLegacySchema);
		if (parsedOverride !== null) {
			views[viewId] = parsedOverride;
		}
	}

	return {
		schemaVersion: LAYOUT_SCHEMA_VERSION,
		views
	};
}

function parseViewOverride(value: unknown, isLegacySchema: boolean): ViewLayoutOverride | null {
	if (
		!isRecord(value) ||
		typeof value.presetId !== 'string' ||
		typeof value.presetVersion !== 'number' ||
		!Number.isInteger(value.presetVersion)
	) {
		return null;
	}

	return {
		presetId: value.presetId,
		presetVersion: value.presetVersion,
		hiddenWidgetIds: parseStringArray(value.hiddenWidgetIds),
		zoneOverrides: parseStringRecord(value.zoneOverrides),
		orderOverrides: parseStringArrayRecord(value.orderOverrides),
		gridOverrides: isLegacySchema ? {} : parseWidgetGridOverrideRecord(value.gridOverrides)
	};
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isLayoutViewId(value: string): value is LayoutViewId {
	return layoutViewIdSet.has(value);
}

function isWidgetScrollMode(value: string): value is WidgetScrollMode {
	return widgetScrollModeSet.has(value);
}

function parseStringArray(value: unknown): string[] {
	if (!Array.isArray(value)) {
		return [];
	}

	return value.filter((item): item is string => typeof item === 'string');
}

function parseStringRecord(value: unknown): Record<string, string> {
	if (!isRecord(value)) {
		return {};
	}

	return Object.fromEntries(
		Object.entries(value).filter((entry): entry is [string, string] => typeof entry[1] === 'string')
	);
}

function parseStringArrayRecord(value: unknown): Record<string, string[]> {
	if (!isRecord(value)) {
		return {};
	}

	return Object.fromEntries(
		Object.entries(value).map(([key, item]) => [key, parseStringArray(item)])
	);
}

function parseWidgetGridOverrideRecord(value: unknown): Record<string, WidgetGridOverride> {
	if (!isRecord(value)) {
		return {};
	}

	return Object.fromEntries(
		Object.entries(value)
			.map(([widgetId, item]) => [widgetId, parseWidgetGridOverride(item)] as const)
			.filter((entry): entry is [string, WidgetGridOverride] => Object.keys(entry[1]).length > 0)
	);
}

function parseWidgetGridOverride(value: unknown): WidgetGridOverride {
	if (!isRecord(value)) {
		return {};
	}

	const override: WidgetGridOverride = {};
	const columns = value.columns;
	const rows = value.rows;
	if (typeof columns === 'number' && Number.isInteger(columns)) {
		override.columns = columns;
	}
	if (typeof rows === 'number' && Number.isInteger(rows)) {
		override.rows = rows;
	}
	if (typeof value.scrollMode === 'string' && isWidgetScrollMode(value.scrollMode)) {
		override.scrollMode = value.scrollMode;
	}

	return override;
}
