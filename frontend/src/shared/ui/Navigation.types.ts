export interface NavigationItem {
	id: string
	label: string
	icon?: string
	href?: string
	disabled?: boolean
	current?: boolean
	children?: NavigationItem[]
}

export interface TreeItemData {
	id: string
	label: string
	icon?: string
	disabled?: boolean
	children?: TreeItemData[]
}
