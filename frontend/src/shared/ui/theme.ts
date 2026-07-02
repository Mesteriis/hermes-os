export const uiThemeNames = ['light', 'dark', 'hermes'] as const
export type UiThemeName = (typeof uiThemeNames)[number]

export interface UiThemeOption {
	value: UiThemeName
	label: string
	description: string
}

export const uiThemeOptions: UiThemeOption[] = [
	{
		value: 'light',
		label: 'Light',
		description: 'Clean corporate light surface for daily work.'
	},
	{
		value: 'dark',
		label: 'Dark',
		description: 'Low-glare neutral dark surface without neon noise.'
	},
	{
		value: 'hermes',
		label: 'Hermes',
		description: 'Signature emerald intelligence theme for focused context work.'
	}
]

export function isUiThemeName(value: unknown): value is UiThemeName {
	return typeof value === 'string' && uiThemeNames.includes(value as UiThemeName)
}
