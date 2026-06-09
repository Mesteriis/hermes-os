import { defaultWidgetGrid } from './grid-defaults';
import type { LayoutViewId, WidgetDefinition } from './types';

const scrollableWidgetIds = new Set<string>([
	'communications-conversation-list',
	'communications-message-detail',
	'communications-ask-ai',
	'timeline-stream',
	'timeline-filters',
	'persons-list',
	'persons-hero',
	'persons-information',
	'persons-about',
	'persons-recent-interactions',
	'persons-active-projects',
	'projects-summary',
	'projects-graph-preview',
	'projects-timeline',
	'projects-recent-communications',
	'projects-top-documents',
	'projects-source-evidence',
	'projects-open-promises',
	'tasks-active-list',
	'tasks-candidate-review',
	'calendar-week-grid',
	'calendar-upcoming',
	'documents-navigation',
	'documents-list',
	'documents-detail-preview',
	'notes-list',
	'notes-detail',
	'knowledge-graph-canvas',
	'knowledge-search-results',
	'telegram-chat-list',
	'telegram-message-thread',
	'telegram-sync-controls',
	'telegram-selected-chat-metadata',
	'whatsapp-account-session-metadata',
	'whatsapp-chat-message-surface',
	'whatsapp-sync-controls',
	'ai-agent-list',
	'ai-selected-agent-detail',
	'ai-run-history',
	'ai-answer-form',
	'organizations-list',
	'organizations-detail',
	'settings-application-list-editor',
	'settings-accounts-list',
	'settings-account-setup-cards',
	'settings-account-detail-status',
	'settings-security-runtime-status'
]);

function widget(
	id: string,
	title: string,
	viewScope: LayoutViewId[],
	defaultZone: string,
	allowedZones: string[],
	dataMode: WidgetDefinition['dataMode'] = 'static'
): WidgetDefinition {
	const grid = defaultWidgetGrid(id, defaultZone);

	return {
		id,
		title,
		viewScope,
		defaultZone,
		allowedZones,
		minColumns: Math.min(grid.columns, defaultZone === 'rail' || defaultZone === 'inspector' ? 2 : 1),
		minRows: Math.min(grid.rows, defaultZone === 'toolbar' || defaultZone === 'tabs' ? 1 : 2),
		defaultScrollMode: scrollableWidgetIds.has(id) ? 'vertical' : 'none',
		priority: 100,
		canHide: true,
		canAdd: true,
		dataMode
	};
}

export const widgetRegistry: WidgetDefinition[] = [
	widget('home-metrics', 'Home Metrics', ['home'], 'metrics', ['metrics'], 'api_backed'),
	widget('home-focus-score', 'Focus Score', ['home'], 'metrics', ['metrics'], 'api_backed'),
	widget('home-whats-new', "What's New", ['home'], 'main', ['main', 'rail'], 'api_backed'),
	widget('home-priorities', "Today's Priorities", ['home'], 'main', ['main'], 'api_backed'),
	widget('home-upcoming', 'Upcoming', ['home'], 'main', ['main', 'rail'], 'api_backed'),
	widget('home-people-talked-to', 'People You Talked To', ['home'], 'rail', ['rail'], 'api_backed'),
	widget('home-system-status', 'System Status', ['home'], 'rail', ['rail'], 'api_backed'),
	widget('home-active-projects', 'Active Projects', ['home'], 'bottom', ['bottom', 'main'], 'api_backed'),
	widget('communications-conversation-list', 'Conversation List', ['communications'], 'list', ['list'], 'api_backed'),
	widget('communications-message-detail', 'Message Detail', ['communications'], 'detail', ['detail'], 'api_backed'),
	widget('communications-sender-profile', 'Sender Profile', ['communications'], 'rail', ['rail'], 'api_backed'),
	widget('communications-summary', 'Summary', ['communications'], 'rail', ['rail'], 'api_backed'),
	widget('communications-message-metadata', 'Message Metadata', ['communications'], 'rail', ['rail'], 'api_backed'),
	widget('communications-related-projects', 'Related Projects', ['communications'], 'rail', ['rail'], 'api_backed'),
	widget('communications-active-tasks', 'Active Tasks', ['communications'], 'rail', ['rail'], 'api_backed'),
	widget(
		'communications-ask-ai',
		'Ask AI',
		['communications'],
		'detail',
		['detail', 'rail'],
		'existing_state'
	),
	widget('timeline-stream', 'Timeline Stream', ['timeline'], 'canvas', ['canvas'], 'api_backed'),
	widget('timeline-filters', 'Timeline Filters', ['timeline'], 'toolbar', ['toolbar'], 'api_backed'),
	widget('timeline-period-summary', 'Period Summary', ['timeline'], 'inspector', ['inspector'], 'api_backed'),
	widget(
		'timeline-selected-event-context',
		'Selected Event Context',
		['timeline'],
		'inspector',
		['inspector'],
		'api_backed'
	),
	widget('persons-list', 'Persons List', ['persons'], 'list', ['list'], 'api_backed'),
	widget('persons-hero', 'Person Hero', ['persons'], 'detail', ['detail'], 'api_backed'),
	widget('persons-information', 'Person Information', ['persons'], 'detail', ['detail', 'rail'], 'api_backed'),
	widget('persons-about', 'About', ['persons'], 'detail', ['detail'], 'api_backed'),
	widget(
		'persons-relationship-strength',
		'Relationship Strength',
		['persons'],
		'detail',
		['detail', 'rail']
	),
	widget('persons-recent-interactions', 'Recent Interactions', ['persons'], 'detail', ['detail'], 'api_backed'),
	widget('persons-active-projects', 'Active Projects', ['persons'], 'detail', ['detail', 'rail'], 'api_backed'),
	widget('persons-ai-summary', 'AI Summary', ['persons'], 'rail', ['rail'], 'api_backed'),
	widget(
		'persons-identity-review',
		'Person Identity Review',
		['persons'],
		'rail',
		['rail'],
		'api_backed'
	),
	widget('persons-related-documents', 'Related Documents', ['persons'], 'rail', ['rail']),
	widget('persons-recent-notes', 'Recent Notes', ['persons'], 'rail', ['rail']),
	widget('projects-hero', 'Project Hero', ['projects'], 'hero', ['hero'], 'api_backed'),
	widget('projects-metadata-strip', 'Metadata Strip', ['projects'], 'metadata', ['metadata'], 'api_backed'),
	widget('projects-switcher', 'Project Switcher', ['projects'], 'tabs', ['tabs'], 'api_backed'),
	widget('projects-section-tabs', 'Section Tabs', ['projects'], 'tabs', ['tabs'], 'api_backed'),
	widget('projects-summary', 'Project Summary', ['projects'], 'main', ['main'], 'api_backed'),
	widget('projects-graph-preview', 'Knowledge Graph', ['projects'], 'main', ['main'], 'api_backed'),
	widget('projects-timeline', 'Project Timeline', ['projects'], 'main', ['main', 'rail'], 'api_backed'),
	widget(
		'projects-recent-communications',
		'Recent Communications',
		['projects'],
		'main',
		['main', 'rail']
	),
	widget('projects-top-documents', 'Top Documents', ['projects'], 'main', ['main', 'rail'], 'api_backed'),
	widget('projects-source-evidence', 'Source Evidence', ['projects'], 'main', ['main'], 'api_backed'),
	widget('projects-open-promises', 'Open Promises', ['projects'], 'main', ['main'], 'api_backed'),
	widget('projects-health', 'Project Health', ['projects'], 'rail', ['rail'], 'api_backed'),
	widget('projects-key-people', 'Key People', ['projects'], 'rail', ['rail'], 'api_backed'),
	widget('projects-related-projects', 'Related Projects', ['projects'], 'rail', ['rail'], 'api_backed'),
	widget('tasks-metrics', 'Task Metrics', ['tasks'], 'header', ['header'], 'api_backed'),
	widget(
		'tasks-candidate-review',
		'Candidate Review Queue',
		['tasks'],
		'list',
		['list'],
		'api_backed'
	),
	widget('tasks-active-list', 'Active Tasks', ['tasks'], 'detail', ['detail'], 'api_backed'),
	widget('tasks-ai-refresh-status', 'AI Refresh Status', ['tasks'], 'rail', ['rail'], 'api_backed'),
	widget('tasks-context', 'Task Context', ['tasks'], 'rail', ['rail'], 'api_backed'),
	widget('tasks-deadlines-priority', 'Deadlines And Priority', ['tasks'], 'rail', ['rail'], 'api_backed'),
	widget('calendar-toolbar', 'Calendar Toolbar', ['calendar'], 'toolbar', ['toolbar']),
	widget('calendar-week-grid', 'Week Grid', ['calendar'], 'canvas', ['canvas']),
	widget('calendar-event-blocks', 'Event Blocks', ['calendar'], 'canvas', ['canvas']),
	widget('calendar-upcoming', 'Upcoming', ['calendar'], 'inspector', ['inspector']),
	widget('calendar-source-status', 'Source Status', ['calendar'], 'inspector', ['inspector']),
	widget('documents-source-cards', 'Source Cards', ['documents'], 'header', ['header'], 'api_backed'),
	widget('documents-navigation', 'Document Navigation', ['documents'], 'list', ['list', 'rail']),
	widget('documents-list', 'Documents List', ['documents'], 'list', ['list']),
	widget('documents-detail-preview', 'Document Detail', ['documents'], 'detail', ['detail']),
	widget(
		'documents-processing-jobs',
		'Processing Jobs',
		['documents'],
		'rail',
		['rail'],
		'api_backed'
	),
	widget(
		'documents-failed-retry-status',
		'Failed Job Retry Status',
		['documents'],
		'rail',
		['rail'],
		'api_backed'
	),
	widget('documents-related-context', 'Related Context', ['documents'], 'rail', ['rail']),
	widget('notes-list', 'Notes List', ['notes'], 'list', ['list']),
	widget('notes-detail', 'Note Detail', ['notes'], 'detail', ['detail']),
	widget('notes-metadata', 'Note Metadata', ['notes'], 'rail', ['rail']),
	widget('notes-source-filters', 'Source Filters', ['notes'], 'header', ['header']),
	widget(
		'notes-related-projects-documents',
		'Related Projects And Documents',
		['notes'],
		'rail',
		['rail']
	),
	widget('knowledge-toolbar', 'Graph Toolbar', ['knowledge-graph'], 'toolbar', ['toolbar']),
	widget(
		'knowledge-graph-canvas',
		'Graph Canvas',
		['knowledge-graph'],
		'canvas',
		['canvas'],
		'api_backed'
	),
	widget(
		'knowledge-node-inspector',
		'Node Inspector',
		['knowledge-graph'],
		'inspector',
		['inspector'],
		'api_backed'
	),
	widget(
		'knowledge-connections',
		'Connections',
		['knowledge-graph'],
		'inspector',
		['inspector'],
		'api_backed'
	),
	widget(
		'knowledge-graph-summary',
		'Graph Summary',
		['knowledge-graph'],
		'inspector',
		['inspector'],
		'api_backed'
	),
	widget(
		'knowledge-search-results',
		'Search Results',
		['knowledge-graph'],
		'inspector',
		['inspector'],
		'api_backed'
	),
	widget(
		'knowledge-evidence-context',
		'Evidence',
		['knowledge-graph'],
		'inspector',
		['inspector'],
		'api_backed'
	),
	widget('telegram-chat-list', 'Telegram Chats', ['telegram'], 'list', ['list'], 'api_backed'),
	widget(
		'telegram-message-thread',
		'Message Thread',
		['telegram'],
		'detail',
		['detail'],
		'api_backed'
	),
	widget('telegram-account-status', 'Account Status', ['telegram'], 'rail', ['rail'], 'api_backed'),
	widget('telegram-sync-controls', 'Sync Controls', ['telegram'], 'rail', ['rail'], 'api_backed'),
	widget(
		'telegram-selected-chat-metadata',
		'Selected Chat Metadata',
		['telegram'],
		'rail',
		['rail']
	),
	widget(
		'whatsapp-session-status',
		'Session Status',
		['whatsapp'],
		'header',
		['header'],
		'api_backed'
	),
	widget(
		'whatsapp-chat-message-surface',
		'Chat Message Surface',
		['whatsapp'],
		'detail',
		['detail'],
		'api_backed'
	),
	widget('whatsapp-sync-controls', 'Sync Controls', ['whatsapp'], 'rail', ['rail'], 'api_backed'),
	widget(
		'whatsapp-account-session-metadata',
		'Account Session Metadata',
		['whatsapp'],
		'rail',
		['rail']
	),
	widget('ai-runtime-metrics', 'Runtime Metrics', ['ai-agents'], 'metrics', ['metrics'], 'api_backed'),
	widget('ai-runtime-status', 'Runtime Status', ['ai-agents'], 'rail', ['rail'], 'api_backed'),
	widget('ai-agent-list', 'Agent List', ['ai-agents'], 'main', ['main'], 'api_backed'),
	widget(
		'ai-selected-agent-detail',
		'Selected Agent Detail',
		['ai-agents'],
		'main',
		['main'],
		'api_backed'
	),
	widget('ai-run-history', 'Run History', ['ai-agents'], 'rail', ['rail'], 'api_backed'),
	widget('ai-answer-form', 'Answer Form', ['ai-agents'], 'main', ['main'], 'api_backed'),
	widget(
		'ai-workflow-panels',
		'Meeting Prep And Task Extraction',
		['ai-agents'],
		'rail',
		['rail'],
		'api_backed'
	),
	widget('ai-citations', 'Citations', ['ai-agents'], 'rail', ['rail'], 'api_backed'),
	widget('organizations-list', 'Company List', ['organizations'], 'list', ['list'], 'api_backed'),
	widget('organizations-detail', 'Company Detail', ['organizations'], 'detail', ['detail'], 'api_backed'),
	widget('organizations-health', 'Company Health', ['organizations'], 'rail', ['rail'], 'api_backed'),
	widget('organizations-key-people', 'Key People', ['organizations'], 'rail', ['rail'], 'api_backed'),
	widget('organizations-related-projects', 'Related Projects', ['organizations'], 'rail', ['rail'], 'api_backed'),
	widget('settings-metrics', 'Settings Metrics', ['settings'], 'metrics', ['metrics'], 'api_backed'),
	widget(
		'settings-application-list-editor',
		'Application Settings',
		['settings'],
		'main',
		['main'],
		'api_backed'
	),
	widget('settings-accounts-list', 'Accounts List', ['settings'], 'main', ['main'], 'api_backed'),
	widget(
		'settings-account-setup-cards',
		'Account Setup',
		['settings'],
		'rail',
		['rail'],
		'api_backed'
	),
	widget(
		'settings-account-detail-status',
		'Account Detail Status',
		['settings'],
		'rail',
		['rail'],
		'api_backed'
	),
	widget(
		'settings-security-runtime-status',
		'Security And Runtime Status',
		['settings'],
		'rail',
		['rail'],
		'api_backed'
	)
];
