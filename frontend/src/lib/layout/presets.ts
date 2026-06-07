import type { LayoutPreset, LayoutViewId, LayoutWidgetInstance } from './types';

export type AppShellLayoutViewId = LayoutViewId | 'knowledge' | 'agents';

const layoutViewIds = [
	'organizations',
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

function instance(widgetId: string, zoneId: string, order: number): LayoutWidgetInstance {
	return {
		widgetId,
		zoneId,
		order,
		sizeIntent: 'auto',
		highlight: 'none',
		visible: true
	};
}

const workbenchZones: LayoutPreset['zones'] = [
	{ id: 'header', title: 'Header', minWidth: 560, minHeight: 72 },
	{ id: 'filters', title: 'Filters', minWidth: 560, minHeight: 48 },
	{ id: 'list', title: 'List', minWidth: 220, minHeight: 320 },
	{ id: 'detail', title: 'Detail', minWidth: 320, minHeight: 320 },
	{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 }
];

const boardZones: LayoutPreset['zones'] = [
	{ id: 'hero', title: 'Hero', minWidth: 560, minHeight: 72 },
	{ id: 'metrics', title: 'Metrics', minWidth: 560, minHeight: 84 },
	{ id: 'main', title: 'Main', minWidth: 320, minHeight: 320 },
	{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 },
	{ id: 'bottom', title: 'Bottom', minWidth: 560, minHeight: 120 }
];

const entityZones: LayoutPreset['zones'] = [
	{ id: 'hero', title: 'Hero', minWidth: 560, minHeight: 96 },
	{ id: 'metadata', title: 'Metadata', minWidth: 560, minHeight: 72 },
	{ id: 'tabs', title: 'Tabs', minWidth: 560, minHeight: 48 },
	{ id: 'main', title: 'Main', minWidth: 320, minHeight: 320 },
	{ id: 'rail', title: 'Rail', minWidth: 220, minHeight: 240 },
	{ id: 'bottom', title: 'Bottom', minWidth: 560, minHeight: 120 }
];

const canvasZones: LayoutPreset['zones'] = [
	{ id: 'toolbar', title: 'Toolbar', minWidth: 560, minHeight: 56 },
	{ id: 'canvas', title: 'Canvas', minWidth: 360, minHeight: 360 },
	{ id: 'inspector', title: 'Inspector', minWidth: 220, minHeight: 240 },
	{ id: 'bottom', title: 'Bottom', minWidth: 560, minHeight: 120 }
];

function preset(
	viewId: LayoutViewId,
	archetype: LayoutPreset['archetype'],
	zones: LayoutPreset['zones'],
	widgets: LayoutWidgetInstance[]
): LayoutPreset {
	return {
		id: `${viewId}-default`,
		version: 1,
		viewId,
		archetype,
		zones,
		widgets
	};
}

export const layoutPresets: LayoutPreset[] = [
	preset('home', 'operational_board', boardZones, [
		instance('home-metrics', 'metrics', 10),
		instance('home-focus-score', 'metrics', 20),
		instance('home-whats-new', 'main', 10),
		instance('home-priorities', 'main', 20),
		instance('home-upcoming', 'main', 30),
		instance('home-people-talked-to', 'rail', 10),
		instance('home-system-status', 'rail', 20),
		instance('home-active-projects', 'bottom', 10)
	]),
	preset('communications', 'master_detail_workbench', workbenchZones, [
		instance('communications-conversation-list', 'list', 10),
		instance('communications-message-detail', 'detail', 10),
		instance('communications-ask-ai', 'detail', 20),
		instance('communications-sender-profile', 'rail', 10),
		instance('communications-summary', 'rail', 20),
		instance('communications-message-metadata', 'rail', 30),
		instance('communications-related-projects', 'rail', 40),
		instance('communications-active-tasks', 'rail', 50)
	]),
	preset('timeline', 'canvas_inspector', canvasZones, [
		instance('timeline-filters', 'toolbar', 10),
		instance('timeline-stream', 'canvas', 10),
		instance('timeline-period-summary', 'inspector', 10),
		instance('timeline-selected-event-context', 'inspector', 20)
	]),
	preset('persons', 'master_detail_workbench', workbenchZones, [
		instance('persons-list', 'list', 10),
		instance('persons-hero', 'detail', 10),
		instance('persons-information', 'detail', 20),
		instance('persons-about', 'detail', 30),
		instance('persons-relationship-strength', 'detail', 40),
		instance('persons-recent-interactions', 'detail', 50),
		instance('persons-active-projects', 'detail', 60),
		instance('persons-ai-summary', 'rail', 10),
		instance('persons-identity-review', 'rail', 20),
		instance('persons-related-documents', 'rail', 30),
		instance('persons-recent-notes', 'rail', 40)
	]),
	preset('projects', 'entity_workspace', entityZones, [
		instance('projects-hero', 'hero', 10),
		instance('projects-metadata-strip', 'metadata', 10),
		instance('projects-switcher', 'tabs', 10),
		instance('projects-section-tabs', 'tabs', 20),
		instance('projects-summary', 'main', 10),
		instance('projects-graph-preview', 'main', 20),
		instance('projects-timeline', 'main', 30),
		instance('projects-recent-communications', 'main', 40),
		instance('projects-top-documents', 'main', 50),
		instance('projects-source-evidence', 'main', 60),
		instance('projects-open-promises', 'main', 70),
		instance('projects-health', 'rail', 10),
		instance('projects-key-people', 'rail', 20),
		instance('projects-related-projects', 'rail', 30)
	]),
	preset('tasks', 'master_detail_workbench', workbenchZones, [
		instance('tasks-metrics', 'header', 10),
		instance('tasks-candidate-review', 'list', 10),
		instance('tasks-active-list', 'detail', 10),
		instance('tasks-ai-refresh-status', 'rail', 10),
		instance('tasks-context', 'rail', 20),
		instance('tasks-deadlines-priority', 'rail', 30)
	]),
	preset('calendar', 'canvas_inspector', canvasZones, [
		instance('calendar-toolbar', 'toolbar', 10),
		instance('calendar-week-grid', 'canvas', 10),
		instance('calendar-event-blocks', 'canvas', 20),
		instance('calendar-upcoming', 'inspector', 10),
		instance('calendar-source-status', 'inspector', 20)
	]),
	preset('documents', 'master_detail_workbench', workbenchZones, [
		instance('documents-source-cards', 'header', 10),
		instance('documents-navigation', 'list', 10),
		instance('documents-list', 'list', 20),
		instance('documents-detail-preview', 'detail', 10),
		instance('documents-processing-jobs', 'rail', 10),
		instance('documents-failed-retry-status', 'rail', 20),
		instance('documents-related-context', 'rail', 30)
	]),
	preset('notes', 'master_detail_workbench', workbenchZones, [
		instance('notes-source-filters', 'header', 10),
		instance('notes-list', 'list', 10),
		instance('notes-detail', 'detail', 10),
		instance('notes-metadata', 'rail', 10),
		instance('notes-related-projects-documents', 'rail', 20)
	]),
	preset('knowledge-graph', 'canvas_inspector', canvasZones, [
		instance('knowledge-toolbar', 'toolbar', 10),
		instance('knowledge-graph-canvas', 'canvas', 10),
		instance('knowledge-node-inspector', 'inspector', 10),
		instance('knowledge-connections', 'inspector', 20),
		instance('knowledge-graph-summary', 'inspector', 30),
		instance('knowledge-search-results', 'inspector', 40),
		instance('knowledge-evidence-context', 'inspector', 50)
	]),
	preset('telegram', 'master_detail_workbench', workbenchZones, [
		instance('telegram-chat-list', 'list', 10),
		instance('telegram-message-thread', 'detail', 10),
		instance('telegram-account-status', 'rail', 10),
		instance('telegram-sync-controls', 'rail', 20),
		instance('telegram-selected-chat-metadata', 'rail', 30)
	]),
	preset('whatsapp', 'master_detail_workbench', workbenchZones, [
		instance('whatsapp-session-status', 'header', 10),
		instance('whatsapp-chat-message-surface', 'detail', 10),
		instance('whatsapp-sync-controls', 'rail', 10),
		instance('whatsapp-account-session-metadata', 'rail', 20)
	]),
	preset('ai-agents', 'operational_board', boardZones, [
		instance('ai-runtime-metrics', 'metrics', 10),
		instance('ai-agent-list', 'main', 10),
		instance('ai-selected-agent-detail', 'main', 20),
		instance('ai-runtime-status', 'rail', 10),
		instance('ai-run-history', 'rail', 20),
		instance('ai-citations', 'rail', 30)
	]),
	preset('organizations', 'master_detail_workbench', workbenchZones, [
		instance('organizations-list', 'list', 10),
		instance('organizations-detail', 'detail', 10),
		instance('organizations-health', 'rail', 10),
		instance('organizations-key-people', 'rail', 20),
		instance('organizations-related-projects', 'rail', 30)
	]),
	preset('settings', 'operational_board', boardZones, [
		instance('settings-metrics', 'metrics', 10),
		instance('settings-application-list-editor', 'main', 10),
		instance('settings-accounts-list', 'main', 20),
		instance('settings-account-setup-cards', 'rail', 10),
		instance('settings-account-detail-status', 'rail', 20),
		instance('settings-security-runtime-status', 'rail', 30)
	])
];

export function layoutViewIdForAppView(viewId: string): LayoutViewId | null {
	if (viewId === 'knowledge') {
		return 'knowledge-graph';
	}

	if (viewId === 'agents') {
		return 'ai-agents';
	}

	if (isLayoutViewId(viewId)) {
		return viewId;
	}

	return null;
}

export function findPresetForView(viewId: AppShellLayoutViewId | string): LayoutPreset | null {
	const layoutViewId = layoutViewIdForAppView(viewId);
	if (layoutViewId === null) {
		return null;
	}

	return layoutPresets.find((preset) => preset.viewId === layoutViewId) ?? null;
}

function isLayoutViewId(viewId: string): viewId is LayoutViewId {
	return layoutViewIdSet.has(viewId);
}
