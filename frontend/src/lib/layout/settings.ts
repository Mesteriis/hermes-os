import {
	LAYOUT_SCHEMA_VERSION,
	type LayoutSettings,
	type LayoutViewId,
	type ViewLayoutOverride,
	type WidgetSizeIntent,
	widgetSizeIntents
} from './types';

const layoutViewIds = [
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
	'settings'
] as const satisfies readonly LayoutViewId[];

const layoutViewIdSet = new Set<string>(layoutViewIds);
const widgetSizeIntentSet = new Set<string>(widgetSizeIntents);

export function defaultLayoutSettings(): LayoutSettings {
	return {
		schemaVersion: LAYOUT_SCHEMA_VERSION,
		views: {}
	};
}

export function parseLayoutSettings(value: unknown): LayoutSettings {
	if (!isRecord(value) || value.schemaVersion !== LAYOUT_SCHEMA_VERSION || !isRecord(value.views)) {
		return defaultLayoutSettings();
	}

	const views: LayoutSettings['views'] = {};
	for (const [viewId, viewOverride] of Object.entries(value.views)) {
		if (!isLayoutViewId(viewId)) {
			continue;
		}

		const parsedOverride = parseViewOverride(viewOverride);
		if (parsedOverride !== null) {
			views[viewId] = parsedOverride;
		}
	}

	return {
		schemaVersion: LAYOUT_SCHEMA_VERSION,
		views
	};
}

function parseViewOverride(value: unknown): ViewLayoutOverride | null {
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
		sizeIntentOverrides: parseWidgetSizeIntentRecord(value.sizeIntentOverrides)
	};
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isLayoutViewId(value: string): value is LayoutViewId {
	return layoutViewIdSet.has(value);
}

function isWidgetSizeIntent(value: string): value is WidgetSizeIntent {
	return widgetSizeIntentSet.has(value);
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

function parseWidgetSizeIntentRecord(value: unknown): Partial<Record<string, WidgetSizeIntent>> {
	if (!isRecord(value)) {
		return {};
	}

	return Object.fromEntries(
		Object.entries(value).filter(
			(entry): entry is [string, WidgetSizeIntent] =>
				typeof entry[1] === 'string' && isWidgetSizeIntent(entry[1])
		)
	);
}
