import type { TelegramMessageInspection } from '../api/telegramMessageInspectorGateway'

export type TelegramMessageInspectorRow = {
	id: string
	title: string
	detail: string
}

export type TelegramMessageInspectorModel = {
	selectedMessageId: string
	pending: boolean
	statusMessage: string
	canQuery: boolean
	overview: readonly string[]
	versions: readonly TelegramMessageInspectorRow[]
	tombstones: readonly TelegramMessageInspectorRow[]
	mutations: readonly TelegramMessageInspectorRow[]
	replyChain: readonly TelegramMessageInspectorRow[]
	forwardChain: readonly TelegramMessageInspectorRow[]
	reactions: readonly TelegramMessageInspectorRow[]
	commands: readonly TelegramMessageInspectorRow[]
}

export function buildTelegramMessageInspectionView(
	inspection: TelegramMessageInspection | null,
): Pick<
	TelegramMessageInspectorModel,
	'commands' | 'forwardChain' | 'mutations' | 'overview' | 'reactions' | 'replyChain' | 'tombstones' | 'versions'
> {
	if (!inspection) {
		return {
			overview: [],
			versions: [],
			tombstones: [],
			mutations: [],
			replyChain: [],
			forwardChain: [],
			reactions: [],
			commands: [],
		}
	}
	return {
		overview: [
			inspection.pinned ? 'pinned' : 'not pinned',
			inspection.references?.replyTo ? 'has reply reference' : '',
			inspection.references?.forwardOrigin ? 'has forward origin' : '',
			inspection.attachment ? `attachment ${inspection.attachment.state}` : '',
			inspection.file?.isDownloaded ? 'file downloaded' : '',
		].filter(Boolean),
		versions: inspection.versions.map((version) => ({
			id: version.versionId,
			title: `Version ${version.versionNumber} · ${version.source}`,
			detail: version.bodyText || 'No text body',
		})),
		tombstones: inspection.tombstones.map((tombstone) => ({
			id: tombstone.tombstoneId,
			title: tombstone.reason || 'Deleted',
			detail: [
				tombstone.isProviderDelete ? 'provider delete' : 'local delete',
				tombstone.isLocallyVisible ? 'visible' : 'hidden',
			].join(' · '),
		})),
		mutations: inspection.mutations.map((mutation, index) => mutationRow(mutation.mutation, index)),
		replyChain: inspection.replyChain.map((message) => messageRow(message.messageId, message)),
		forwardChain: inspection.forwardChain.map((message) => messageRow(message.messageId, message)),
		reactions: inspection.reactionSummary.map((reaction) => ({
			id: reaction.emoji,
			title: `${reaction.emoji} · ${reaction.count}`,
			detail: reaction.isActive ? 'active for current account' : 'observed',
		})),
		commands: inspection.commands.map((record) => ({
			id: record.operation?.operationId || 'unknown-operation',
			title: record.operation?.commandKind || record.command?.command.case || 'provider command',
			detail: record.operation?.state || 'unknown',
		})),
	}
}

function mutationRow(
	mutation: NonNullable<TelegramMessageInspection['mutations'][number]>['mutation'],
	index: number,
): TelegramMessageInspectorRow {
	switch (mutation.case) {
		case 'edit':
			return { id: `edit-${index}`, title: 'Edited', detail: mutation.value.text || 'No text body' }
		case 'delete':
			return {
				id: `delete-${index}`,
				title: 'Deleted',
				detail: mutation.value.isPermanent ? 'permanent' : 'recoverable',
			}
		case 'pin':
			return {
				id: `pin-${index}`,
				title: mutation.value.isPinned ? 'Pinned' : 'Unpinned',
				detail: 'provider mutation',
			}
		case 'reaction':
			return {
				id: `reaction-${index}`,
				title: mutation.value.isActive ? 'Reaction added' : 'Reaction removed',
				detail: mutation.value.emoji || 'reaction',
			}
		default:
			return { id: `unknown-${index}`, title: 'Unknown mutation', detail: 'unsupported revision' }
	}
}

function messageRow(
	id: string,
	message: TelegramMessageInspection['replyChain'][number],
): TelegramMessageInspectorRow {
	return {
		id,
		title: message.senderDisplayName || message.senderId || 'Unknown sender',
		detail: message.text || message.media?.caption || `[${message.media?.kind || 'message'}]`,
	}
}
