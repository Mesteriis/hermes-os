export type GraphicTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'info'

export interface DonutChartSegment {
	id: string
	label: string
	value: number
	tone?: GraphicTone
}

export interface RadarChartMetric {
	id: string
	label: string
	value: number
	max?: number
}

export interface CandlestickPoint {
	id: string
	label: string
	open: number
	high: number
	low: number
	close: number
}
