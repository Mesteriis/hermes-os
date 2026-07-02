export type UtilityTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'info'

export interface LocaleSwitcherOption {
	value: string
	label: string
	description?: string
}

export type ProviderIconKind = 'mail' | 'telegram' | 'whatsapp' | 'calendar' | 'documents' | 'generic'

export type StatusIconKind = 'idle' | 'active' | 'success' | 'warning' | 'danger' | 'offline' | 'syncing'

export type EntityIconKind =
	| 'person'
	| 'organization'
	| 'project'
	| 'task'
	| 'document'
	| 'decision'
	| 'obligation'
	| 'knowledge'
	| 'event'
	| 'generic'

export type FileIconKind = 'image' | 'audio' | 'video' | 'pdf' | 'code' | 'archive' | 'spreadsheet' | 'text' | 'generic'
