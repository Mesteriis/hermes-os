export type MailMessageLocationStatus =
	| 'blocked'
	| 'error'
	| 'idle'
	| 'outcome-unknown'
	| 'pending'
	| 'rejected'
	| 'succeeded'
	| 'unsupported'

export type MailMessageLocationFolderOption = {
	id: string
	label: string
}

export type MailMessageLocationModel = {
	canMutate: boolean
	canQueryStatus: boolean
	hasSelection: boolean
	isTrashed: boolean
	busy: boolean
	status: MailMessageLocationStatus
	statusMessage: string
	operationId: string
	targetFolderId: string
	targetFolders: readonly MailMessageLocationFolderOption[]
}
