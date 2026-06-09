export const LAYOUT_SCHEMA_VERSION = 2;
export const LEGACY_LAYOUT_SCHEMA_VERSION = 1;
export const LAYOUT_GRID_COLUMNS = 12;
export const LAYOUT_GRID_MAX_ROWS = 24;
export const LAYOUT_GRID_MIN_COLUMNS = 1;
export const LAYOUT_GRID_MIN_ROWS = 1;

export const layoutArchetypes = [
	'operational_board',
	'master_detail_workbench',
	'entity_workspace',
	'canvas_inspector'
] as const;

export type LayoutArchetype = (typeof layoutArchetypes)[number];

export const widgetSizeIntents = ['auto', 'compact', 'normal', 'wide', 'tall', 'large'] as const;

export type WidgetSizeIntent = (typeof widgetSizeIntents)[number];

export const widgetHighlightStates = ['none', 'border', 'pulse-once', 'pulse-continuous'] as const;

export type WidgetHighlightState = (typeof widgetHighlightStates)[number];

export const widgetScrollModes = ['none', 'vertical', 'horizontal', 'both'] as const;

export type WidgetScrollMode = (typeof widgetScrollModes)[number];

export type WidgetDataMode = 'static' | 'existing_state' | 'api_backed';

export type LayoutViewId =
	| 'home'
	| 'communications'
	| 'timeline'
	| 'persons'
	| 'projects'
	| 'tasks'
	| 'calendar'
	| 'documents'
	| 'notes'
	| 'knowledge-graph'
	| 'telegram'
	| 'whatsapp'
	| 'ai-agents'
	| 'organizations'
	| 'settings';

export type WidgetGridOverride = {
	columns?: number;
	rows?: number;
	scrollMode?: WidgetScrollMode;
};

export type WidgetDefinition = {
	id: string;
	title: string;
	viewScope: LayoutViewId[];
	defaultZone: string;
	allowedZones: string[];
	defaultColumns: number;
	defaultRows: number;
	minColumns: number;
	minRows: number;
	defaultScrollMode: WidgetScrollMode;
	defaultSizeIntent: WidgetSizeIntent;
	priority: number;
	canHide: boolean;
	canAdd: boolean;
	dataMode: WidgetDataMode;
};

export type LayoutZoneDefinition = {
	id: string;
	title: string;
	minWidth: number;
	minHeight: number;
};

export type LayoutWidgetInstance = {
	widgetId: string;
	zoneId: string;
	order: number;
	columns: number;
	rows: number;
	sizeIntent: WidgetSizeIntent;
	highlight: WidgetHighlightState;
	visible: boolean;
};

export type LayoutPreset = {
	id: string;
	version: number;
	viewId: LayoutViewId;
	archetype: LayoutArchetype;
	zones: LayoutZoneDefinition[];
	widgets: LayoutWidgetInstance[];
};

export type ViewLayoutOverride = {
	presetId: string;
	presetVersion: number;
	hiddenWidgetIds: string[];
	zoneOverrides: Record<string, string>;
	orderOverrides: Record<string, string[]>;
	gridOverrides: Record<string, WidgetGridOverride>;
	sizeIntentOverrides: Partial<Record<string, WidgetSizeIntent>>;
};

export type LayoutSettings = {
	schemaVersion: typeof LAYOUT_SCHEMA_VERSION;
	views: Partial<Record<LayoutViewId, ViewLayoutOverride>>;
};

export type ResolvedWidget = LayoutWidgetInstance & {
	definition: WidgetDefinition;
	columns: number;
	rows: number;
	minColumns: number;
	minRows: number;
	scrollMode: WidgetScrollMode;
	isHiddenByUser: boolean;
};

export type ResolvedLayout = {
	preset: LayoutPreset;
	zones: LayoutZoneDefinition[];
	widgetsByZone: Record<string, ResolvedWidget[]>;
	hiddenByUser: ResolvedWidget[];
	ignoredWidgetIds: string[];
};
