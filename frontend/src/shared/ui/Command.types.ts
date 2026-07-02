export interface CommandGroup {
	label: string
	items: CommandItem[]
}

export interface CommandItem {
	id: string
	label: string
	description?: string
	icon?: string
	keywords?: string[]
	onSelect?: () => void
}
