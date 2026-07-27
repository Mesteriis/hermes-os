export type MailMessageFlagStatus =
	| 'blocked'
	| 'error'
	| 'idle'
	| 'outcome-unknown'
	| 'pending'
	| 'rejected'
	| 'succeeded'

export type MailMessageFlagModel = {
	canMutate: boolean
	canQueryStatus: boolean
	hasSelection: boolean
	isRead: boolean
	isStarred: boolean
	busy: boolean
	status: MailMessageFlagStatus
	statusMessage: string
	operationId: string
}
