export interface TimelineMessage {
	message_id: string
	sender_display_name: string | null
	sender: string
	subject: string
	body_text_preview: string
	occurred_at: string | null
	projected_at: string
	channel_kind: string
}

export type TimelineFilterKind = 'Messages' | 'Documents' | 'Tasks' | 'Calendar' | 'Notes' | 'Decisions'

export interface TimelineFilters {
	Messages: boolean
	Documents: boolean
	Tasks: boolean
	Calendar: boolean
	Notes: boolean
	Decisions: boolean
}
