export type TelegramOperationRetryModel = {
	operationId: string
	pending: boolean
	statusMessage: string
	canRetry: boolean
}
