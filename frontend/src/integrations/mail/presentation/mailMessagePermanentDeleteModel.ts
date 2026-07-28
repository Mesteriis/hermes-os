export type MailMessagePermanentDeleteStatus =
	| 'blocked'
	| 'error'
	| 'idle'
	| 'outcome-unknown'
	| 'pending'
	| 'reauthorization-required'
	| 'rejected'
	| 'succeeded'
	| 'unsupported'

export type MailMessagePermanentDeleteModel = {
	canDelete: boolean
	canQueryStatus: boolean
	hasTrashSelection: boolean
	confirmed: boolean
	busy: boolean
	status: MailMessagePermanentDeleteStatus
	statusMessage: string
	operationId: string
}
