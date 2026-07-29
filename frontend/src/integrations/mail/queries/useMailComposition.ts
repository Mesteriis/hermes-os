import { computed, ref } from 'vue'
import type { MailCompositionModel } from '../presentation/mailCompositionModel'
import type { MailAccountConnection } from './mailAccountConnections'
import { useMailDrafts } from './useMailDrafts'
import { useMailSignatures } from './useMailSignatures'
import { useMailTemplates } from './useMailTemplates'

export type MailDeliveryInput = {
	connectionId: string
	providerConversationId: string
	toRecipients: readonly string[]
	ccRecipients: readonly string[]
	bccRecipients: readonly string[]
	subject: string
	textBody: string
}

export function useMailComposition(input: {
	canMutate: () => boolean
	canQuery: () => boolean
	connections: () => readonly MailAccountConnection[]
}) {
	const status = ref<MailCompositionModel['status']>('blocked')
	const statusMessage = ref('')
	const selectedConnectionId = ref('')
	const refreshing = ref(false)
	let generation = 0

	const connections = computed(input.connections)
	const connectionId = () => selectedConnectionId.value
	const resourceInput = { canMutate: input.canMutate, connectionId }
	const drafts = useMailDrafts(resourceInput)
	const templates = useMailTemplates(resourceInput)
	const signatures = useMailSignatures(resourceInput)

	const model = computed<MailCompositionModel>(() => ({
		canMutate: input.canMutate(),
		canQuery: input.canQuery(),
		status: status.value,
		statusMessage: statusMessage.value,
		notice: drafts.notice.value || templates.notice.value || signatures.notice.value,
		busyAction: refreshing.value
			? 'refresh'
			: drafts.busy.value
				? 'draft'
				: templates.busy.value
					? templates.previewing.value ? 'preview' : 'template'
					: signatures.busy.value ? 'signature' : null,
		connections: connections.value.map((connection) => ({
			id: connection.connectionId,
			label: connection.connectionId,
			detail: connection.registrationId,
		})),
		selectedConnectionId: selectedConnectionId.value,
		drafts: drafts.options.value,
		templates: templates.options.value,
		signatures: signatures.options.value,
		draft: drafts.model.value,
		template: templates.model.value,
		signature: signatures.model.value,
	}))

	const deliveryInput = computed<MailDeliveryInput>(() => {
		const draft = drafts.deliveryInput.value
		const signature = signatures.records.value.find(
			(candidate) => candidate.signatureId === draft.signatureId,
		)
		return {
			connectionId: selectedConnectionId.value,
			providerConversationId: draft.providerConversationId,
			toRecipients: draft.toRecipients,
			ccRecipients: draft.ccRecipients,
			bccRecipients: draft.bccRecipients,
			subject: draft.subject,
			textBody: signature
				? `${draft.textBody.trimEnd()}\n\n${signature.textBody}`.trim()
				: draft.textBody,
		}
	})

	async function reconcile(): Promise<void> {
		const available = connections.value
		if (!input.canQuery()) {
			clear('Mail composition query capability is not admitted.')
			return
		}
		if (available.length === 0) {
			clear('No admitted Mail composition connection is available.')
			status.value = 'empty'
			return
		}
		if (!available.some((connection) => connection.connectionId === selectedConnectionId.value)) {
			selectedConnectionId.value = available[0]!.connectionId
		}
		await refresh()
	}

	async function refresh(): Promise<void> {
		if (!readyForQuery()) return
		const token = ++generation
		refreshing.value = true
		status.value = 'loading'
		statusMessage.value = 'Loading Mail composition workspace…'
		try {
			await Promise.all([drafts.load(), templates.load(), signatures.load()])
			if (token !== generation) return
			status.value = 'ready'
			statusMessage.value = ''
		} catch (error) {
			if (token !== generation) return
			status.value = 'error'
			statusMessage.value = error instanceof Error
				? error.message
				: 'Mail composition workspace is unavailable.'
		} finally {
			if (token === generation) refreshing.value = false
		}
	}

	async function selectConnection(connectionId: string): Promise<void> {
		if (!connections.value.some((connection) => connection.connectionId === connectionId)) return
		selectedConnectionId.value = connectionId
		resetResources()
		await refresh()
	}

	async function applyTemplate(): Promise<void> {
		const preview = await templates.preview()
		if (preview?.ready) drafts.applyTemplate(preview)
	}

	function readyForQuery(): boolean {
		if (!input.canQuery() || !selectedConnectionId.value) {
			status.value = 'blocked'
			statusMessage.value = 'Mail composition query is unavailable.'
			return false
		}
		return true
	}

	function clear(message: string): void {
		generation += 1
		refreshing.value = false
		status.value = 'blocked'
		statusMessage.value = message
		selectedConnectionId.value = ''
		resetResources()
	}

	function resetResources(): void {
		drafts.clear()
		templates.clear()
		signatures.clear()
	}

	return {
		model,
		deliveryInput,
		connectionId,
		reconcile,
		refresh,
		selectConnection,
		selectDraft: drafts.select,
		selectTemplate: templates.select,
		selectSignature: signatures.select,
		newDraft: drafts.startNew,
		newTemplate: templates.startNew,
		newSignature: signatures.startNew,
		saveDraft: drafts.save,
		removeDraft: drafts.remove,
		saveTemplate: templates.save,
		removeTemplate: templates.remove,
		applyTemplate,
		saveSignature: signatures.save,
		removeSignature: signatures.remove,
		useSignature: drafts.useSignature,
		updateDraft: drafts.update,
		updateTemplate: templates.update,
		updateSignature: signatures.update,
	}
}
