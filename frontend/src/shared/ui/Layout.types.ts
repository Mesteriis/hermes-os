export type LayoutGap = 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl'
export type LayoutAlign = 'start' | 'center' | 'end' | 'stretch' | 'baseline'
export type LayoutJustify = 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'
export type LayoutDirection = 'row' | 'column'
export type LayoutOrientation = 'horizontal' | 'vertical'
export type LayoutTone = 'default' | 'muted' | 'raised' | 'floating'

export interface LayoutStatusItem {
	id: string
	label: string
	value?: string | number
	tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'info'
}
