export type TelegramChatCommandModel = {
	folderId: string
	targetFolderIds: string
	pending: boolean
	statusMessage: string
	canCommand: boolean
	hasChat: boolean
}
