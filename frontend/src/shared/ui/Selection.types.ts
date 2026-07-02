export type SelectionTone = 'default' | 'muted' | 'warning' | 'danger' | 'success'

export interface SelectOption {
	value: string
	label: string
	description?: string
	disabled?: boolean
	icon?: string
	tone?: SelectionTone
}

export interface SelectGroup {
	id: string
	label: string
	options: SelectOption[]
}

export interface TreeSelectOption extends SelectOption {
	children?: TreeSelectOption[]
}

export interface SelectionSearchState {
	query: string
}
