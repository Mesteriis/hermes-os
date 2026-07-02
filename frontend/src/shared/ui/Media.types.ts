export type MediaTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'info'

export interface MediaImageItem {
	id: string
	src: string
	alt: string
	title?: string
	description?: string
	meta?: string
}

export interface MediaAttachmentItem {
	id: string
	name: string
	mimeType?: string
	size?: string
	description?: string
	icon?: string
	tone?: MediaTone
}

export interface MediaTrack {
	src: string
	kind: 'subtitles' | 'captions' | 'descriptions' | 'chapters' | 'metadata'
	label?: string
	srclang?: string
	default?: boolean
}
