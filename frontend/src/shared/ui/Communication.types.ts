export type CommunicationTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'info'

export type MessageDirection = 'inbound' | 'outbound' | 'system'

export type MessageDeliveryState = 'queued' | 'sent' | 'delivered' | 'read' | 'failed'

export interface ComposerToolbarAction {
	id: string
	label: string
	icon?: string
	active?: boolean
	disabled?: boolean
	tone?: CommunicationTone
}

export interface ReadReceiptItem {
	id: string
	label: string
	initials?: string
	src?: string
	timestamp?: string
}
