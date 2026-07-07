export interface StepsItem {
	title?: string
	description?: string
	requirement?: string
}

export interface StepsSlotProps {
	step: number
	stepCount: number
	isFirst: boolean
	isLast: boolean
	next: () => void
	previous: () => void
	goToStep: (step: number) => void
	close: () => void
}
