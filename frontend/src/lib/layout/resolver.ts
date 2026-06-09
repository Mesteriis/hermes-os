import type { LayoutPreset, ResolvedLayout, ResolvedWidget, ViewLayoutOverride, WidgetDefinition } from './types';
import { LAYOUT_GRID_COLUMNS, LAYOUT_GRID_MAX_ROWS, LAYOUT_GRID_MIN_COLUMNS, LAYOUT_GRID_MIN_ROWS } from './types';

export function resolveLayout(
	preset: LayoutPreset,
	definitions: WidgetDefinition[],
	override: ViewLayoutOverride | undefined
): ResolvedLayout {
	const definitionsById = new Map(definitions.map((definition) => [definition.id, definition]));
	const zoneIds = new Set(preset.zones.map((zone) => zone.id));
	const widgetsByZone: Record<string, ResolvedWidget[]> = Object.fromEntries(
		preset.zones.map((zone) => [zone.id, []])
	);
	const hiddenByUser: ResolvedWidget[] = [];
	const ignoredWidgetIds: string[] = [];
	const hiddenWidgetIds = new Set(override?.hiddenWidgetIds ?? []);

	for (const instance of preset.widgets) {
		const definition = definitionsById.get(instance.widgetId);
		if (!definition) {
			ignoredWidgetIds.push(instance.widgetId);
			continue;
		}

		const requestedZoneId = override?.zoneOverrides[instance.widgetId];
		const zoneId =
			requestedZoneId &&
			zoneIds.has(requestedZoneId) &&
			definition.allowedZones.includes(requestedZoneId)
				? requestedZoneId
				: instance.zoneId;
		const gridOverride = override?.gridOverrides[instance.widgetId];
		const minColumns = clampGridNumber(
			definition.minColumns,
			LAYOUT_GRID_MIN_COLUMNS,
			LAYOUT_GRID_COLUMNS
		);
		const minRows = clampGridNumber(definition.minRows, LAYOUT_GRID_MIN_ROWS, LAYOUT_GRID_MAX_ROWS);
		const columns = clampGridNumber(
			gridOverride?.columns ?? instance.columns,
			minColumns,
			LAYOUT_GRID_COLUMNS
		);
		const rows = clampGridNumber(
			gridOverride?.rows ?? instance.rows,
			minRows,
			LAYOUT_GRID_MAX_ROWS
		);
		const scrollMode = gridOverride?.scrollMode ?? definition.defaultScrollMode;
		const panelOpacity = gridOverride?.panelOpacity;
		const panelBlur = gridOverride?.panelBlur;
		const resolvedWidget: ResolvedWidget = {
			...instance,
			zoneId,
			columns,
			rows,
			minColumns,
			minRows,
			scrollMode,
			panelOpacity,
			panelBlur,
			definition,
			isHiddenByUser: hiddenWidgetIds.has(instance.widgetId)
		};

		if (resolvedWidget.isHiddenByUser || !instance.visible) {
			hiddenByUser.push(resolvedWidget);
			continue;
		}

		widgetsByZone[zoneId]?.push(resolvedWidget);
	}

	for (const zone of preset.zones) {
		widgetsByZone[zone.id] = sortZoneWidgets(
			widgetsByZone[zone.id] ?? [],
			override?.orderOverrides[zone.id]
		);
	}

	return {
		preset,
		zones: preset.zones,
		widgetsByZone,
		hiddenByUser,
		ignoredWidgetIds
	};
}

function sortZoneWidgets(
	widgets: ResolvedWidget[],
	orderOverride: string[] | undefined
): ResolvedWidget[] {
	if (!orderOverride?.length) {
		return sortByPresetOrder(widgets);
	}

	const overrideOrderByWidgetId = new Map(
		orderOverride.map((widgetId, index) => [widgetId, index])
	);

	return [...widgets].sort((left, right) => {
		const leftOverrideOrder = overrideOrderByWidgetId.get(left.widgetId);
		const rightOverrideOrder = overrideOrderByWidgetId.get(right.widgetId);

		if (leftOverrideOrder !== undefined && rightOverrideOrder !== undefined) {
			return leftOverrideOrder - rightOverrideOrder;
		}

		if (leftOverrideOrder !== undefined) {
			return -1;
		}

		if (rightOverrideOrder !== undefined) {
			return 1;
		}

		return left.order - right.order;
	});
}

function sortByPresetOrder(widgets: ResolvedWidget[]): ResolvedWidget[] {
	return [...widgets].sort((left, right) => left.order - right.order);
}

function clampGridNumber(value: number, min: number, max: number): number {
	if (!Number.isInteger(value)) {
		return min;
	}

	return Math.min(max, Math.max(min, value));
}
