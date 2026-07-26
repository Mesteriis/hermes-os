export type ModuleSettingRowModel = {
	key: string
	label: string
	settingId: string
	value: string
	editable: boolean
	blocked: boolean
}

export type ModuleSettingsPanelModel = {
	title: string
	description: string
	icon: string
	tone: 'mail' | 'telegram' | 'whatsapp' | 'zulip'
	moduleId: string
	registered: boolean
	applyState: string
	revision: string
	reasonCode: string
	settings: readonly ModuleSettingRowModel[]
}
