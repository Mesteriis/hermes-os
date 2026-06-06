import type { LayoutViewId, WidgetDefinition } from './types';

function widget(
	id: string,
	title: string,
	viewScope: LayoutViewId[],
	defaultZone: string,
	allowedZones: string[],
	dataMode: WidgetDefinition['dataMode'] = 'static'
): WidgetDefinition {
	return {
		id,
		title,
		viewScope,
		defaultZone,
		allowedZones,
		minSize: { width: 220, height: 120 },
		defaultSizeIntent: 'auto',
		priority: 100,
		canHide: true,
		canAdd: true,
		dataMode
	};
}

export const widgetRegistry: WidgetDefinition[] = [
	widget('home-metrics', 'Home Metrics', ['home'], 'metrics', ['metrics']),
	widget('home-focus-score', 'Focus Score', ['home'], 'metrics', ['metrics']),
	widget('home-whats-new', "What's New", ['home'], 'main', ['main', 'rail']),
	widget('home-priorities', "Today's Priorities", ['home'], 'main', ['main']),
	widget('home-upcoming', 'Upcoming', ['home'], 'main', ['main', 'rail']),
	widget('home-people-talked-to', 'People You Talked To', ['home'], 'rail', ['rail']),
	widget('home-system-status', 'System Status', ['home'], 'rail', ['rail']),
	widget('home-active-projects', 'Active Projects', ['home'], 'bottom', ['bottom', 'main']),
	widget('communications-conversation-list', 'Conversation List', ['communications'], 'list', ['list']),
	widget('communications-message-detail', 'Message Detail', ['communications'], 'detail', ['detail']),
	widget('communications-sender-profile', 'Sender Profile', ['communications'], 'rail', ['rail']),
	widget('communications-summary', 'Summary', ['communications'], 'rail', ['rail']),
	widget('communications-message-metadata', 'Message Metadata', ['communications'], 'rail', ['rail']),
	widget('communications-related-projects', 'Related Projects', ['communications'], 'rail', ['rail']),
	widget('communications-active-tasks', 'Active Tasks', ['communications'], 'rail', ['rail']),
	widget(
		'communications-ask-ai',
		'Ask AI',
		['communications'],
		'detail',
		['detail', 'rail'],
		'existing_state'
	),
	widget('timeline-stream', 'Timeline Stream', ['timeline'], 'canvas', ['canvas']),
	widget('timeline-filters', 'Timeline Filters', ['timeline'], 'toolbar', ['toolbar']),
	widget('timeline-period-summary', 'Period Summary', ['timeline'], 'inspector', ['inspector']),
	widget(
		'timeline-selected-event-context',
		'Selected Event Context',
		['timeline'],
		'inspector',
		['inspector']
	),
	widget('contacts-list', 'Contacts List', ['contacts'], 'list', ['list']),
	widget('contacts-hero', 'Contact Hero', ['contacts'], 'detail', ['detail']),
	widget('contacts-information', 'Contact Information', ['contacts'], 'detail', ['detail', 'rail']),
	widget('contacts-about', 'About', ['contacts'], 'detail', ['detail']),
	widget(
		'contacts-relationship-strength',
		'Relationship Strength',
		['contacts'],
		'detail',
		['detail', 'rail']
	),
	widget('contacts-recent-interactions', 'Recent Interactions', ['contacts'], 'detail', ['detail']),
	widget('contacts-active-projects', 'Active Projects', ['contacts'], 'detail', ['detail', 'rail']),
	widget('contacts-ai-summary', 'AI Summary', ['contacts'], 'rail', ['rail']),
	widget(
		'contacts-identity-review',
		'Contact Identity Review',
		['contacts'],
		'rail',
		['rail'],
		'api_backed'
	),
	widget('contacts-related-documents', 'Related Documents', ['contacts'], 'rail', ['rail']),
	widget('contacts-recent-notes', 'Recent Notes', ['contacts'], 'rail', ['rail']),
	widget('projects-hero', 'Project Hero', ['projects'], 'hero', ['hero']),
	widget('projects-metadata-strip', 'Metadata Strip', ['projects'], 'metadata', ['metadata']),
	widget('projects-switcher', 'Project Switcher', ['projects'], 'tabs', ['tabs']),
	widget('projects-section-tabs', 'Section Tabs', ['projects'], 'tabs', ['tabs']),
	widget('projects-summary', 'Project Summary', ['projects'], 'main', ['main']),
	widget('projects-graph-preview', 'Knowledge Graph', ['projects'], 'main', ['main']),
	widget('projects-timeline', 'Project Timeline', ['projects'], 'main', ['main', 'rail']),
	widget(
		'projects-recent-communications',
		'Recent Communications',
		['projects'],
		'main',
		['main', 'rail']
	),
	widget('projects-top-documents', 'Top Documents', ['projects'], 'main', ['main', 'rail']),
	widget('projects-source-evidence', 'Source Evidence', ['projects'], 'main', ['main']),
	widget('projects-open-promises', 'Open Promises', ['projects'], 'main', ['main']),
	widget('projects-health', 'Project Health', ['projects'], 'rail', ['rail']),
	widget('projects-key-people', 'Key People', ['projects'], 'rail', ['rail']),
	widget('projects-related-projects', 'Related Projects', ['projects'], 'rail', ['rail']),
	widget('tasks-metrics', 'Task Metrics', ['tasks'], 'header', ['header']),
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
	widget('tasks-context', 'Task Context', ['tasks'], 'rail', ['rail']),
	widget('tasks-deadlines-priority', 'Deadlines And Priority', ['tasks'], 'rail', ['rail']),
	widget('calendar-toolbar', 'Calendar Toolbar', ['calendar'], 'toolbar', ['toolbar']),
	widget('calendar-week-grid', 'Week Grid', ['calendar'], 'canvas', ['canvas']),
	widget('calendar-event-blocks', 'Event Blocks', ['calendar'], 'canvas', ['canvas']),
	widget('calendar-upcoming', 'Upcoming', ['calendar'], 'inspector', ['inspector']),
	widget('calendar-source-status', 'Source Status', ['calendar'], 'inspector', ['inspector']),
	widget('documents-source-cards', 'Source Cards', ['documents'], 'header', ['header']),
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
