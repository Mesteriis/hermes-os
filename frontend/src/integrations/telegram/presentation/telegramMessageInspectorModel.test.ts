import { describe, expect, it } from 'vitest'

import { buildTelegramMessageInspectionView } from './telegramMessageInspectorModel'

describe('Telegram message inspector presentation model', () => {
	it('maps versions, lineage, mutations and command audit into bounded rows', () => {
		const model = buildTelegramMessageInspectionView({
			message: { messageId: 'message-1' } as never,
			versions: [{
				versionId: 'version-1',
				versionNumber: 2,
				source: 'provider_edit',
				bodyText: 'Updated',
			} as never],
			tombstones: [],
			mutations: [{
				mutation: { case: 'pin', value: { isPinned: true } },
			} as never],
			references: {
				replyTo: { providerMessageId: 'previous-1' },
			} as never,
			replyChain: [],
			forwardChain: [],
			attachment: { state: 'safe' } as never,
			file: undefined,
			reactions: [],
			reactionSummary: [{ emoji: '👍', count: 2, isActive: true } as never],
			commands: [{
				operation: {
					operationId: 'operation-1',
					commandKind: 'pin',
					state: 'completed',
				},
			} as never],
			pinned: true,
		})

		expect(model.overview).toEqual([
			'pinned',
			'has reply reference',
			'attachment safe',
		])
		expect(model.versions[0]).toMatchObject({ title: 'Version 2 · provider_edit' })
		expect(model.mutations[0]).toMatchObject({ title: 'Pinned' })
		expect(model.reactions[0]).toMatchObject({ title: '👍 · 2' })
		expect(model.commands[0]).toMatchObject({ id: 'operation-1', detail: 'completed' })
	})
})
