import type { ComposeFormModel, CommunicationAccountOption, MailSyncStatus } from '../types/communications'
import type { CommunicationConversationModel } from '../components/communicationDomainElements'
import type { MailInspectorModel } from '../components/mail/mailInspector'
import type { MailListItemModel } from '../components/mail/mailElements'
import type {
	MessengerAttachmentModel,
	MessengerConversationModel,
	MessengerInspectorModel,
	MessengerListItemModel,
} from '../components/messengers/messengerElements'
import type { TelegramChat, TelegramMessage } from '@/shared/communications/types/telegram'
import type {
	TelegramConversationRuntimeAction,
	TelegramConversationRuntimeActionRunner,
} from '@/shared/communications/types/telegramRuntimeActions'

export type CommunicationsPageModel =
	| {
		channel: 'mail'
		items: readonly MailListItemModel[]
		conversation: CommunicationConversationModel
		inspector: MailInspectorModel
		hasMoreItems: boolean
		isImporting: boolean
		composeError: string
		composeAccountOptions: readonly CommunicationAccountOption[]
		composeForm: ComposeFormModel
		composeOpen: boolean
		composeStatus: string
		isActionRunning: boolean
		isLoadingMore: boolean
		isSending: boolean
		searchQuery: string
		syncStatus: MailSyncStatus | null
	}
	| {
		channel: 'telegram'
		items: readonly MessengerListItemModel[]
		conversation: MessengerConversationModel
		inspector: MessengerInspectorModel
		isActionRunning: boolean
		isListLoading: boolean
		isListRefreshing: boolean
		isLoadingOlder: boolean
		listError: string
		selectedMessageId?: string
		telegramChat: TelegramChat | null
		telegramMessage: TelegramMessage | null
		runtimeActionRunner?: TelegramConversationRuntimeActionRunner
	}
	| {
		channel: 'whatsapp'
		items: readonly MessengerListItemModel[]
		conversation: MessengerConversationModel
		inspector: MessengerInspectorModel
	}

export type CommunicationsPageActions = {
	closeCompose(): void
	importMailFile(file: File): void | Promise<void>
	attachComposeFiles(files: File[]): void
	loadMoreMail(): void | Promise<void>
	newMailMessage(): void
	refreshMail(): void | Promise<void>
	removeComposeAttachment(attachmentId: string): void
	saveCompose(): void | Promise<void>
	selectMailAction(actionId: string): void | Promise<void>
	selectMailMessage(item: MailListItemModel): void
	sendCompose(): void | Promise<void>
	updateMailSearch(query: string): void
	updateCompose(partial: Partial<ComposeFormModel>): void
	setVisibleMailItemIds(itemIds: string[]): void
	runTelegramAction(action: TelegramConversationRuntimeAction, file?: File, caption?: string): void | Promise<void>
	refreshTelegram(): void | Promise<void>
	selectTelegramConversation(item: MessengerListItemModel): void
	selectTelegramMessage(messageId: string): void
	submitTelegram(value: string): void | Promise<void>
	downloadTelegramAttachment(attachment: MessengerAttachmentModel): void | Promise<void>
	loadOlderTelegram(): void | Promise<void>
	markTelegramMessagesVisible(): void | Promise<void>
	selectWhatsappConversation(item: MessengerListItemModel): void
}
