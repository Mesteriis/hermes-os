import { computed, ref } from 'vue'

import {
	downloadTelegramFile,
	sendTelegramMedia,
} from '../api/telegramMediaCommandGateway'
import { useTelegramCommandFeedback } from './useTelegramCommandFeedback'

export type TelegramMediaCommandModel = {
	mediaKind: string
	blobRef: string
	referenceIdHex: string
	declaredSize: string
	backupClass: string
	caption: string
	filename: string
	providerFileId: string
	pending: boolean
	statusMessage: string
	canCommand: boolean
	hasChat: boolean
}

export function useTelegramMediaCommands(input: {
	accountId: () => string
	canCommand: () => boolean
	providerChatId: () => string
}) {
	const mediaKind = ref('document')
	const blobRef = ref('')
	const referenceIdHex = ref('')
	const declaredSize = ref('')
	const backupClass = ref('')
	const caption = ref('')
	const filename = ref('')
	const providerFileId = ref('')
	const feedback = useTelegramCommandFeedback(input.canCommand)

	const model = computed<TelegramMediaCommandModel>(() => ({
		mediaKind: mediaKind.value,
		blobRef: blobRef.value,
		referenceIdHex: referenceIdHex.value,
		declaredSize: declaredSize.value,
		backupClass: backupClass.value,
		caption: caption.value,
		filename: filename.value,
		providerFileId: providerFileId.value,
		pending: feedback.pending.value,
		statusMessage: feedback.statusMessage.value,
		canCommand: input.canCommand(),
		hasChat: Boolean(input.providerChatId()),
	}))

	async function sendMedia(): Promise<void> {
		await feedback.run(() => sendTelegramMedia({
			accountId: input.accountId(),
			providerChatId: input.providerChatId(),
			operationId: crypto.randomUUID(),
			mediaKind: mediaKind.value,
			blobRef: blobRef.value,
			referenceIdHex: referenceIdHex.value,
			declaredSize: parseDeclaredSize(),
			backupClass: parseBackupClass(),
			caption: caption.value,
			filename: filename.value,
		}))
	}

	async function downloadFile(): Promise<void> {
		await feedback.run(() => downloadTelegramFile({
			accountId: input.accountId(),
			providerFileId: providerFileId.value,
			operationId: crypto.randomUUID(),
			priority: 1,
		}))
	}

	function updateMediaKind(value: string): void {
		mediaKind.value = value
	}

	function updateBlobRef(value: string): void {
		blobRef.value = value
	}

	function updateReferenceIdHex(value: string): void {
		referenceIdHex.value = value
	}

	function updateDeclaredSize(value: string): void {
		declaredSize.value = value
	}

	function updateBackupClass(value: string): void {
		backupClass.value = value
	}

	function updateCaption(value: string): void {
		caption.value = value
	}

	function updateFilename(value: string): void {
		filename.value = value
	}

	function updateProviderFileId(value: string): void {
		providerFileId.value = value
	}

	function parseDeclaredSize(): bigint {
		if (!/^[1-9]\d*$/.test(declaredSize.value.trim())) {
			throw new RangeError('Telegram media size must be a positive integer')
		}
		return BigInt(declaredSize.value.trim())
	}

	function parseBackupClass(): number {
		const normalized = backupClass.value.trim()
		if (!/^\d+$/.test(normalized)) {
			throw new RangeError('Telegram media backup class must be a non-negative integer')
		}
		const parsed = Number(normalized)
		if (!Number.isSafeInteger(parsed) || parsed > 4_294_967_295) {
			throw new RangeError('Telegram media backup class exceeds uint32')
		}
		return parsed
	}

	return {
		model,
		sendMedia,
		downloadFile,
		updateMediaKind,
		updateBlobRef,
		updateReferenceIdHex,
		updateDeclaredSize,
		updateBackupClass,
		updateCaption,
		updateFilename,
		updateProviderFileId,
	}
}
