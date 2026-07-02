export type DataDisplayTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger' | 'accent'

export type DataTableCell = string | number | null | undefined

export interface DataTableColumn {
	key: string
	label: string
	align?: 'left' | 'center' | 'right'
}

export type DataTableRow = Record<string, DataTableCell>

export interface DataListItem {
	id: string
	label: string
	description?: string
	meta?: string
	icon?: string
	tone?: DataDisplayTone
}

export interface KeyValueItem {
	id: string
	label: string
	value: string | number
	description?: string
	tone?: DataDisplayTone
}

export interface TimelineEntry {
	id: string
	title: string
	description?: string
	time?: string
	icon?: string
	tone?: DataDisplayTone
}
