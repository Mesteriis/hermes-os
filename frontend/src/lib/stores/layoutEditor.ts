import { derived, get, writable } from 'svelte/store';
import {
	LAYOUT_GRID_COLUMNS,
	LAYOUT_GRID_MAX_ROWS,
	defaultLayoutSettings,
	findPresetForView,
	panelBlurValues,
	panelOpacityValues,
	parseLayoutSettings,
	resolveLayout,
	widgetRegistry,
	type LayoutSettings,
	type ResolvedLayout,
	type ResolvedWidget,
	type ViewLayoutOverride
} from '$lib/layout';
import { saveFrontendLayoutSetting } from '$lib/api';
import { activeWorkspaceView, isUserMenuOpen } from './navigation';
import { effectiveThemeSettings } from './theme';

export type WidgetGridDimension = 'columns' | 'rows';
export type WidgetPanelSurfaceSetting = 'panelOpacity' | 'panelBlur';

export const layoutSettings = writable<LayoutSettings>(defaultLayoutSettings());
export const layoutDraft = writable<LayoutSettings | null>(null);
export const isLayoutEditing = writable(false);
export const isWidgetDrawerOpen = writable(false);
export const selectedLayoutWidgetId = writable<string | null>(null);
export const layoutError = writable('');
export const isLayoutSettingsSaving = writable(false);
export const viewportHiddenWidgetTitles = writable<string[]>([]);

export const effectiveLayoutSettings = derived(
	[layoutSettings, layoutDraft],
	([$layoutSettings, $layoutDraft]) => $layoutDraft ?? $layoutSettings
);

export const activeLayout = derived(
	[activeWorkspaceView, effectiveLayoutSettings],
	([$activeWorkspaceView, $effectiveLayoutSettings]) => resolveActiveLayout($activeWorkspaceView, $effectiveLayoutSettings)
);

export const activeWidgetById = derived(activeLayout, ($activeLayout) => {
	const widgets = Object.values($activeLayout?.widgetsByZone ?? {}).flat();
	return new Map(widgets.map((widget) => [widget.widgetId, widget]));
});

export const selectedLayoutWidget = derived(
	[selectedLayoutWidgetId, activeWidgetById],
	([$selectedLayoutWidgetId, $activeWidgetById]) =>
		$selectedLayoutWidgetId ? $activeWidgetById.get($selectedLayoutWidgetId) ?? null : null
);

export const hiddenWidgetTitles = derived(activeLayout, ($activeLayout) =>
	($activeLayout?.hiddenByUser ?? []).map((widget) => widget.definition.title)
);

export const visibleWidgetIds = derived(activeLayout, ($activeLayout) => {
	const widgets = Object.values($activeLayout?.widgetsByZone ?? {}).flat();
	return new Set(widgets.map((widget) => widget.widgetId));
});

export const addableWidgetsForCurrentView = derived(activeLayout, ($activeLayout) => {
	if (!$activeLayout) {
		return [];
	}

	const hiddenWidgetIds = new Set(($activeLayout.hiddenByUser ?? []).map((widget) => widget.widgetId));

	return widgetRegistry
		.filter(
			(widget) =>
				widget.canAdd &&
				widget.viewScope.includes($activeLayout.preset.viewId) &&
				hiddenWidgetIds.has(widget.id)
		)
		.sort((left, right) => {
			return left.title.localeCompare(right.title);
		});
});

const gridClassNames = [
	...Array.from({ length: 12 }, (_, index) => `widget-cols-${index + 1}`),
	...Array.from({ length: 24 }, (_, index) => `widget-rows-${index + 1}`),
	...panelOpacityValues.map((value) => `widget-panel-opacity-${value}`),
	...panelBlurValues.map((value) => `widget-panel-blur-${value}`),
	'widget-scroll-none',
	'widget-scroll-y',
	'widget-scroll-x',
	'widget-scroll-both',
	'widget-fit-hidden'
];

const scrollClassByMode = {
	none: 'widget-scroll-none',
	vertical: 'widget-scroll-y',
	horizontal: 'widget-scroll-x',
	both: 'widget-scroll-both'
};

export function setLayoutSettings(settings: LayoutSettings): void {
	layoutSettings.set(parseLayoutSettings(settings));
	layoutDraft.set(null);
	layoutError.set('');
}

export function startLayoutEditing(): void {
	layoutDraft.set(cloneLayoutSettings(get(layoutSettings)));
	isLayoutEditing.set(true);
	isUserMenuOpen.set(false);
	layoutError.set('');
	selectedLayoutWidgetId.set(null);
}

export function cancelLayoutEditing(): void {
	layoutDraft.set(null);
	isLayoutEditing.set(false);
	isWidgetDrawerOpen.set(false);
	selectedLayoutWidgetId.set(null);
	layoutError.set('');
}

export async function saveLayoutSettings(): Promise<void> {
	const nextSettings = parseLayoutSettings(get(layoutDraft) ?? get(layoutSettings));
	isLayoutSettingsSaving.set(true);
	try {
		const updated = await saveFrontendLayoutSetting(nextSettings);
		layoutSettings.set(parseLayoutSettings(updated.value));
		layoutDraft.set(null);
		isLayoutEditing.set(false);
		isWidgetDrawerOpen.set(false);
		selectedLayoutWidgetId.set(null);
		layoutError.set('');
	} catch (error) {
		layoutError.set(error instanceof Error ? error.message : 'Unknown layout settings update error');
	} finally {
		isLayoutSettingsSaving.set(false);
	}
}

export function openAddWidgetDrawer(): void {
	selectedLayoutWidgetId.set(null);
	isWidgetDrawerOpen.set(true);
}

export function closeAddWidgetDrawer(): void {
	isWidgetDrawerOpen.set(false);
}

export function resetCurrentViewLayout(): void {
	const layout = get(activeLayout);
	const layoutViewId = layout?.preset.viewId;
	if (!layoutViewId) {
		return;
	}

	const draft = get(layoutDraft) ?? cloneLayoutSettings(get(layoutSettings));
	const views = { ...draft.views };
	delete views[layoutViewId];
	layoutDraft.set({ ...draft, views });
	selectedLayoutWidgetId.set(null);
	layoutError.set('');
}

export function hideWidget(widgetId: string): void {
	updateCurrentViewOverride((override) => {
		if (override.hiddenWidgetIds.includes(widgetId)) {
			return override;
		}

		return {
			...override,
			hiddenWidgetIds: [...override.hiddenWidgetIds, widgetId]
		};
	});
	if (get(selectedLayoutWidgetId) === widgetId) {
		selectedLayoutWidgetId.set(null);
	}
}

export function showWidget(widgetId: string): void {
	updateCurrentViewOverride((override) => ({
		...override,
		hiddenWidgetIds: override.hiddenWidgetIds.filter((id) => id !== widgetId)
	}));
	isWidgetDrawerOpen.set(false);
}

export function openWidgetSettingsDrawer(widgetId: string): void {
	selectedLayoutWidgetId.set(widgetId);
	isWidgetDrawerOpen.set(false);
}

export function closeWidgetSettingsDrawer(): void {
	selectedLayoutWidgetId.set(null);
}

export function widgetZoneTitle(zoneId: string): string {
	return get(activeLayout)?.zones.find((zone) => zone.id === zoneId)?.title ?? zoneId;
}

export function widgetGridValue(widgetId: string, dimension: WidgetGridDimension): number {
	return get(activeWidgetById).get(widgetId)?.[dimension] ?? 1;
}

export function widgetGridMin(widgetId: string, dimension: WidgetGridDimension): number {
	const widget = get(activeWidgetById).get(widgetId);
	if (!widget) {
		return 1;
	}

	return dimension === 'columns' ? widget.minColumns : widget.minRows;
}

export function widgetGridMax(dimension: WidgetGridDimension): number {
	return dimension === 'columns' ? LAYOUT_GRID_COLUMNS : LAYOUT_GRID_MAX_ROWS;
}

export function adjustWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, delta: -1 | 1): void {
	setWidgetGridValue(widgetId, dimension, widgetGridValue(widgetId, dimension) + delta);
}

export function handleWidgetGridInput(widgetId: string, dimension: WidgetGridDimension, event: Event): void {
	const input = event.currentTarget;
	if (!(input instanceof HTMLInputElement)) {
		return;
	}

	setWidgetGridValue(widgetId, dimension, input.valueAsNumber);
}

export function widgetPanelSurfaceValue(widgetId: string, setting: WidgetPanelSurfaceSetting): number {
	const widget = get(activeWidgetById).get(widgetId);
	return widget?.[setting] ?? get(effectiveThemeSettings)[setting];
}

export function widgetPanelSurfaceOverrideValue(widgetId: string, setting: WidgetPanelSurfaceSetting): number | null {
	return get(activeWidgetById).get(widgetId)?.[setting] ?? null;
}

export function handleWidgetPanelSurfaceInput(
	widgetId: string,
	setting: WidgetPanelSurfaceSetting,
	event: Event
): void {
	const input = event.currentTarget;
	if (!(input instanceof HTMLInputElement)) {
		return;
	}

	setWidgetPanelSurfaceValue(widgetId, setting, input.valueAsNumber);
}

export function resetWidgetPanelSurface(widgetId: string): void {
	updateCurrentViewOverride((override) => {
		const currentGridOverride = override.gridOverrides[widgetId];
		if (!currentGridOverride) {
			return override;
		}

		const restGridOverride = { ...currentGridOverride };
		delete restGridOverride.panelOpacity;
		delete restGridOverride.panelBlur;
		const nextGridOverrides = { ...override.gridOverrides };
		if (Object.keys(restGridOverride).length === 0) {
			delete nextGridOverrides[widgetId];
		} else {
			nextGridOverrides[widgetId] = restGridOverride;
		}

		return {
			...override,
			gridOverrides: nextGridOverrides
		};
	});
}

export function resetWidgetGrid(widgetId: string): void {
	updateCurrentViewOverride((override) => {
		const currentGridOverride = override.gridOverrides[widgetId];
		if (!currentGridOverride) {
			return override;
		}

		const restGridOverride = { ...currentGridOverride };
		delete restGridOverride.columns;
		delete restGridOverride.rows;
		const nextGridOverrides = { ...override.gridOverrides };
		if (Object.keys(restGridOverride).length === 0) {
			delete nextGridOverrides[widgetId];
		} else {
			nextGridOverrides[widgetId] = restGridOverride;
		}

		return {
			...override,
			gridOverrides: nextGridOverrides
		};
	});
}

export function moveWidgetInZone(widgetId: string, direction: -1 | 1): void {
	const layout = get(activeLayout);
	if (!layout) return;

	const widget = Object.values(layout.widgetsByZone)
		.flat()
		.find((item) => item.widgetId === widgetId);
	if (!widget) return;

	const zoneWidgets = layout.widgetsByZone[widget.zoneId] ?? [];
	const ids = zoneWidgets.map((item) => item.widgetId);
	const index = ids.indexOf(widgetId);
	const nextIndex = index + direction;
	if (index < 0 || nextIndex < 0 || nextIndex >= ids.length) return;

	const nextIds = [...ids];
	[nextIds[index], nextIds[nextIndex]] = [nextIds[nextIndex], nextIds[index]];

	updateCurrentViewOverride((override) => ({
		...override,
		orderOverrides: {
			...override.orderOverrides,
			[widget.zoneId]: nextIds
		}
	}));
}

export function isWidgetVisible(widgetId: string): boolean {
	const layout = get(activeLayout);
	if (!layout) return true;

	return Object.values(layout.widgetsByZone).some((widgets) =>
		widgets.some((widget) => widget.widgetId === widgetId)
	);
}

export function syncWidgetGridClasses(widgetsById = get(activeWidgetById)): void {
	if (typeof document === 'undefined') {
		return;
	}

	const layoutEditing = get(isLayoutEditing);
	if (window.scrollX !== 0 || window.scrollY !== 0) {
		window.scrollTo(0, 0);
	}

	const workspace = document.querySelector<HTMLElement>('.workspace');
	const workspaceWidth = workspace?.clientWidth ?? window.innerWidth;
	const workspaceRect = workspace?.getBoundingClientRect();
	const statusStrip = document.querySelector<HTMLElement>('.workspace-status-strip');
	const statusRect = statusStrip?.getBoundingClientRect();
	const minimumColumnWidth = 44;
	const availableColumns = Math.max(1, Math.min(12, Math.floor(workspaceWidth / minimumColumnWidth)));

	for (const element of document.querySelectorAll<HTMLElement>('.widget-frame[data-widget-id]')) {
		element.classList.remove(...gridClassNames);

		const widgetId = element.dataset.widgetId;
		if (!widgetId) continue;

		const widget = widgetsById.get(widgetId);
		if (!widget) continue;

		const scrollClass = scrollClassByMode[widget.scrollMode];
		element.classList.add(`widget-cols-${widget.columns}`, `widget-rows-${widget.rows}`, scrollClass);
		if (widget.panelOpacity !== undefined) {
			element.classList.add(`widget-panel-opacity-${widget.panelOpacity}`);
		}
		if (widget.panelBlur !== undefined) {
			element.classList.add(`widget-panel-blur-${widget.panelBlur}`);
		}
		element.dataset.widgetColumns = String(widget.columns);
		element.dataset.widgetRows = String(widget.rows);
		element.dataset.widgetMinColumns = String(widget.minColumns);
		element.dataset.widgetMinRows = String(widget.minRows);
		element.dataset.widgetScroll = widget.scrollMode;
		element.dataset.widgetPanelOpacity =
			widget.panelOpacity === undefined ? 'global' : String(widget.panelOpacity);
		element.dataset.widgetPanelBlur = widget.panelBlur === undefined ? 'global' : String(widget.panelBlur);
	}

	if (layoutEditing) {
		viewportHiddenWidgetTitles.set([]);
		return;
	}

	requestAnimationFrame(() => {
		if (window.scrollX !== 0 || window.scrollY !== 0) {
			window.scrollTo(0, 0);
		}

		const hiddenByViewport: string[] = [];
		const rightLimit = workspaceRect?.right ?? window.innerWidth;
		const bottomLimit = statusRect?.top ?? workspaceRect?.bottom ?? window.innerHeight;

		for (const element of document.querySelectorAll<HTMLElement>('.widget-frame[data-widget-id]')) {
			const widgetId = element.dataset.widgetId;
			const widget = widgetId ? widgetsById.get(widgetId) : null;
			if (!widget) continue;

			const rect = element.getBoundingClientRect();
			const isFitHidden =
				widget.minColumns > availableColumns ||
				rect.left < (workspaceRect?.left ?? 0) - 1 ||
				rect.right > rightLimit + 1 ||
				rect.bottom > bottomLimit - 1;

			if (isFitHidden) {
				element.classList.add('widget-fit-hidden');
				hiddenByViewport.push(widget.definition.title);
			} else {
				element.classList.remove('widget-fit-hidden');
			}
		}

		for (const element of document.querySelectorAll<HTMLElement>('.widget-frame[data-widget-hide-if-clipped-content][data-widget-id]')) {
			const widgetId = element.dataset.widgetId;
			const widget = widgetId ? widgetsById.get(widgetId) : null;
			const contentElements = Array.from(
				element.querySelectorAll<HTMLElement>('[data-widget-fit-content]')
			);
			if (!widget || contentElements.length === 0 || element.classList.contains('widget-fit-hidden')) {
				continue;
			}

			const rect = element.getBoundingClientRect();
			const isContentClipped = contentElements.some((content) => {
				const contentRect = content.getBoundingClientRect();
				const hasInternalOverflow =
					content.scrollHeight > content.clientHeight + 1 ||
					content.scrollWidth > content.clientWidth + 1;

				return (
					hasInternalOverflow ||
					contentRect.left < rect.left - 1 ||
					contentRect.right > rect.right + 1 ||
					contentRect.top < rect.top - 1 ||
					contentRect.bottom > rect.bottom + 1
				);
			});

			if (isContentClipped) {
				element.classList.add('widget-fit-hidden');
				hiddenByViewport.push(widget.definition.title);
			}
		}

		viewportHiddenWidgetTitles.set(hiddenByViewport);
	});
}

function setWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, value: number): void {
	const nextValue = normalizeWidgetGridValue(widgetId, dimension, value);
	updateCurrentViewOverride((override) => {
		const currentGridOverride = override.gridOverrides[widgetId] ?? {};

		return {
			...override,
			gridOverrides: {
				...override.gridOverrides,
				[widgetId]: {
					...currentGridOverride,
					[dimension]: nextValue
				}
			}
		};
	});
}

function setWidgetPanelSurfaceValue(widgetId: string, setting: WidgetPanelSurfaceSetting, value: number): void {
	const allowedValues: readonly number[] = setting === 'panelOpacity' ? panelOpacityValues : panelBlurValues;
	if (!allowedValues.includes(value)) {
		return;
	}

	updateCurrentViewOverride((override) => {
		const currentGridOverride = override.gridOverrides[widgetId] ?? {};
		return {
			...override,
			gridOverrides: {
				...override.gridOverrides,
				[widgetId]: {
					...currentGridOverride,
					[setting]: value
				}
			}
		};
	});
}

function normalizeWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, value: number): number {
	if (!Number.isFinite(value)) {
		return widgetGridValue(widgetId, dimension);
	}

	return Math.max(widgetGridMin(widgetId, dimension), Math.min(widgetGridMax(dimension), Math.trunc(value)));
}

function updateCurrentViewOverride(update: (override: ViewLayoutOverride) => ViewLayoutOverride): void {
	const override = ensureCurrentViewOverride();
	const layoutViewId = get(activeLayout)?.preset.viewId ?? findPresetForView(get(activeWorkspaceView))?.viewId;
	const draft = get(layoutDraft);
	if (!override || !draft || !layoutViewId) {
		return;
	}

	layoutDraft.set({
		...draft,
		views: {
			...draft.views,
			[layoutViewId]: update(override)
		}
	});
}

function ensureCurrentViewOverride(): ViewLayoutOverride | null {
	const preset = get(activeLayout)?.preset ?? findPresetForView(get(activeWorkspaceView));
	if (!preset) {
		return null;
	}

	const draft = get(layoutDraft) ?? cloneLayoutSettings(get(layoutSettings));
	const existingOverride = draft.views[preset.viewId];
	const override = existingOverride ?? {
		presetId: preset.id,
		presetVersion: preset.version,
		hiddenWidgetIds: [],
		zoneOverrides: {},
		orderOverrides: {},
		gridOverrides: {}
	};

	layoutDraft.set({
		...draft,
		views: {
			...draft.views,
			[preset.viewId]: override
		}
	});
	layoutError.set('');

	return override;
}

function resolveActiveLayout(viewId: string, settings: LayoutSettings): ResolvedLayout | null {
	const preset = findPresetForView(viewId);
	if (!preset) return null;
	return resolveLayout(preset, widgetRegistry, settings.views[preset.viewId]);
}

function cloneLayoutSettings(settings: LayoutSettings): LayoutSettings {
	return structuredClone(settings);
}
