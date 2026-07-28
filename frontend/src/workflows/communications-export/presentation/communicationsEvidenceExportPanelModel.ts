export type CommunicationsEvidenceExportPanelStatus =
	| 'unavailable'
	| 'idle'
	| 'starting'
	| 'pending'
	| 'materializing'
	| 'ready'
	| 'rejected'
	| 'downloading'
	| 'error'

export type CommunicationsEvidenceExportPanelModel = {
	available: boolean
	busy: boolean
	canAddCandidate: boolean
	canDownload: boolean
	canRefresh: boolean
	selectedCount: number
	progressLabel: string
	status: CommunicationsEvidenceExportPanelStatus
	statusMessage: string
}
