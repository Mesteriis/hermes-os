export type CallTranscriptionPanelStatus =
	| 'unavailable'
	| 'waiting-source'
	| 'starting'
	| 'awaiting-recording'
	| 'awaiting-stt'
	| 'materializing'
	| 'ready'
	| 'rejected'
	| 'error'

export type CallTranscriptionPanelModel = {
	available: boolean
	busy: boolean
	canRetry: boolean
	status: CallTranscriptionPanelStatus
	statusMessage: string
	transcriptText: string
	detectedLanguage: string
	durationLabel: string
}
